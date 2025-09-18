#!/bin/bash
set -e

# EPSX Cloud Run Deployment Script
# Deploy services to Google Cloud Run using YAML configurations
#
# Usage:
#   ./scripts/deploy/deploy.sh [environment] [service]
#
# Examples:
#   ./scripts/deploy/deploy.sh production            # Deploy all services to production
#   ./scripts/deploy/deploy.sh staging backend      # Deploy only backend to staging
#   ./scripts/deploy/deploy.sh development frontend # Deploy only frontend to development

ENVIRONMENT="${1:-production}"
SERVICE="${2:-all}"

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

# Validate service
if [[ "$SERVICE" != "all" && "$SERVICE" != "backend" && "$SERVICE" != "frontend" && "$SERVICE" != "admin" ]]; then
    log_error "Invalid service: $SERVICE"
    echo "Valid services: backend, frontend, admin, all"
    exit 1
fi

log_header "EPSX Cloud Run Deployment"
echo "Environment: $ENVIRONMENT"
echo "Service: $SERVICE"
echo "Project: $PROJECT_ID"
echo "Region: $REGION"
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

# Set project
log_info "Setting gcloud project to $PROJECT_ID..."
gcloud config set project "$PROJECT_ID"

# Deployment function - delegate to deploy-service.sh
deploy_service() {
    local svc="$1"
    
    log_info "Deploying $svc using deploy-service.sh..."
    
    # Use the individual service deployment script
    ./scripts/deploy/deploy-service.sh "$svc" "$ENVIRONMENT"
}

# Health check function
health_check() {
    local svc="$1"
    local url=""
    
    case "$svc" in
        "backend")
            url="$BACKEND_URL/health"
            ;;
        "frontend")
            url="$FRONTEND_URL"
            ;;
        "admin")
            url="$ADMIN_URL"
            ;;
    esac
    
    if [[ -n "$url" ]]; then
        log_info "Health checking $svc: $url"
        if curl --retry=5 --retry-delay=10 --retry-max-time=60 -f -s "$url" > /dev/null; then
            log_success "$svc is healthy"
        else
            log_warning "$svc health check failed"
        fi
    fi
}

# Deploy services
if [[ "$SERVICE" == "all" ]]; then
    log_info "Deploying all services..."
    
    # Deploy in order: backend first, then frontend services
    deploy_service "backend"
    deploy_service "frontend"
    deploy_service "admin"
    
    log_success "All services deployed"
    
    # Health checks
    log_info "Performing health checks..."
    health_check "backend"
    health_check "frontend"
    health_check "admin"
    
else
    deploy_service "$SERVICE"
    health_check "$SERVICE"
fi

log_header "Deployment Summary"
echo "✅ Environment: $ENVIRONMENT"
echo "✅ Service: $SERVICE"
echo "✅ Project: $PROJECT_ID"
echo "✅ Backend: $BACKEND_URL"
echo "✅ Frontend: $FRONTEND_URL"
echo "✅ Admin: $ADMIN_URL"
echo ""
log_success "Deployment completed successfully!"

# Show useful commands
echo ""
echo "📋 Useful commands:"
echo "  View logs: gcloud logging read \"resource.type=cloud_run_revision\" --project=$PROJECT_ID --limit=20"
echo "  List services: gcloud run services list --project=$PROJECT_ID --region=$REGION"
if [[ "$SERVICE" != "all" ]]; then
    echo "  Service details: gcloud run services describe $SERVICE --project=$PROJECT_ID --region=$REGION"
fi