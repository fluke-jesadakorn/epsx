#!/bin/bash
set -e

# EPSX Individual Service Deployment
# Deploy a single service to Google Cloud Run
#
# Usage:
#   ./scripts/deploy/deploy-service.sh [service] [environment]
#
# Examples:
#   ./scripts/deploy/deploy-service.sh backend production
#   ./scripts/deploy/deploy-service.sh frontend staging
#   ./scripts/deploy/deploy-service.sh admin development

SERVICE="${1:-backend}"
ENVIRONMENT="${2:-production}"

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_header() { echo -e "${PURPLE}🚀 $1${NC}"; }

# Validate service
if [[ "$SERVICE" != "backend" && "$SERVICE" != "frontend" && "$SERVICE" != "admin" ]]; then
    log_error "Invalid service: $SERVICE"
    echo "Valid services: backend, frontend, admin"
    exit 1
fi

# Load environment configuration from .env files
ENV_FILE=""
case "$ENVIRONMENT" in
    "development")
        ENV_FILE=".env.development"
        PROJECT_ID="epsx-development"
        ;;
    "staging")
        ENV_FILE=".env.staging"
        PROJECT_ID="epsx-staging"
        ;;
    "production")
        ENV_FILE="production/deployment/environments/production.env"
        PROJECT_ID="epsx-469400"
        ;;
    *)
        log_error "Invalid environment: $ENVIRONMENT"
        echo "Valid environments: development, staging, production"
        exit 1
        ;;
esac

# Load environment variables
if [[ -f "$ENV_FILE" ]]; then
    log_info "Loading environment from $ENV_FILE..."
    set -a  # Export all variables
    source "$ENV_FILE"
    set +a  # Stop exporting
else
    log_warning "Environment file $ENV_FILE not found, using defaults"
fi

# Set default values if not provided
REGION="${REGION:-us-central1}"
BACKEND_URL="${BACKEND_URL}"
FRONTEND_URL="${FRONTEND_URL}"
ADMIN_URL="${ADMIN_FRONTEND_URL}"

log_header "EPSX Service Deployment"
echo "Service: $SERVICE"
echo "Environment: $ENVIRONMENT"
echo "Project: $PROJECT_ID"
echo ""

# Validate prerequisites
if ! command -v gcloud &> /dev/null; then
    log_error "gcloud CLI is not installed"
    exit 1
fi

if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    log_error "Not authenticated with gcloud. Run: gcloud auth login"
    exit 1
fi

# Check YAML template exists
YAML_TEMPLATE="scripts/deploy/services/$ENVIRONMENT/$SERVICE.yaml"
if [[ ! -f "$YAML_TEMPLATE" ]]; then
    log_error "Service YAML template not found: $YAML_TEMPLATE"
    exit 1
fi

# Create temporary processed YAML with environment variables substituted
YAML_FILE="/tmp/epsx-${SERVICE}-${ENVIRONMENT}-$(date +%s).yaml"
log_info "Processing YAML template with environment variables..."
envsubst < "$YAML_TEMPLATE" > "$YAML_FILE"

# Set project
log_info "Setting gcloud project to $PROJECT_ID..."
gcloud config set project "$PROJECT_ID"

# Deploy service
log_info "Deploying $SERVICE from $YAML_FILE..."

gcloud run services replace "$YAML_FILE" \
    --region="$REGION" \
    --project="$PROJECT_ID" \
    --quiet

log_success "$SERVICE deployed successfully"

# Cleanup temporary YAML file
rm -f "$YAML_FILE"

# Health check
log_info "Performing health check..."

case "$SERVICE" in
    "backend")
        HEALTH_URL="$BACKEND_URL/health"
        ;;
    "frontend")
        HEALTH_URL="$FRONTEND_URL"
        ;;
    "admin")
        HEALTH_URL="$ADMIN_URL"
        ;;
esac

log_info "Testing $SERVICE: $HEALTH_URL"

if curl --retry=5 --retry-delay=10 --retry-max-time=60 -f -s "$HEALTH_URL" > /dev/null; then
    log_success "$SERVICE is healthy"
else
    log_warning "$SERVICE health check failed"
fi

log_header "Service Deployment Summary"
echo "✅ Service: $SERVICE"
echo "✅ Environment: $ENVIRONMENT"
echo "✅ Project: $PROJECT_ID"
echo "✅ URL: $HEALTH_URL"
echo ""
log_success "$SERVICE deployment completed!"

# Show service-specific commands
echo ""
echo "📋 Service commands:"
echo "  View logs: gcloud logging read \"resource.type=cloud_run_revision resource.labels.service_name=$SERVICE\" --project=$PROJECT_ID --limit=10"
echo "  Service status: gcloud run services describe $SERVICE --project=$PROJECT_ID --region=$REGION"
echo "  Update traffic: gcloud run services update-traffic $SERVICE --project=$PROJECT_ID --region=$REGION"