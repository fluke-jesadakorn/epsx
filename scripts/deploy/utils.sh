#!/bin/bash
# EPSX Deployment Utilities
# Unified environment validation and deployment helpers

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${GREEN}✅${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ️${NC} $1"
}

# Load environment file and validate using unified schema
load_and_validate_env() {
    local ENV_NAME=$1
    # Get absolute path to project root from script location
    local SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
    local PROJECT_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
    local ENV_FILE=""
    
    echo "🔍 Loading environment configuration for: $ENV_NAME"
    
    # Determine environment file path
    case $ENV_NAME in
        "production")
            ENV_FILE="$PROJECT_ROOT/production/deployment/environments/production.env"
            if [ ! -f "$ENV_FILE" ]; then
                print_error "Production environment file not found: $ENV_FILE"
                print_info "Please copy production.env.example to production.env and configure it"
                exit 1
            fi
            ;;
        "staging")
            ENV_FILE="$PROJECT_ROOT/production/deployment/environments/staging.env"
            if [ ! -f "$ENV_FILE" ]; then
                print_error "Staging environment file not found: $ENV_FILE"
                print_info "Please copy staging.env.example to staging.env and configure it"
                exit 1
            fi
            ;;
        "development")
            ENV_FILE="$PROJECT_ROOT/production/deployment/environments/development.env"
            if [ ! -f "$ENV_FILE" ]; then
                print_error "Development environment file not found: $ENV_FILE"
                print_info "Please ensure development.env exists in production/deployment/environments/"
                exit 1
            fi
            ;;
        *)
            print_error "Invalid environment: $ENV_NAME. Must be development, staging, or production"
            exit 1
            ;;
    esac
    
    print_info "Loading environment from: $ENV_FILE"
    
    # Load environment file
    if [ -f "$ENV_FILE" ]; then
        set -a  # Export all variables
        source "$ENV_FILE"
        set +a  # Stop exporting
        print_status "Environment file loaded successfully"
    else
        print_error "Environment file not found: $ENV_FILE"
        exit 1
    fi
    
    # Validate using unified schema (if available)
    if command -v node &> /dev/null && [ -f "$PROJECT_ROOT/scripts/utils/validate-env.js" ]; then
        print_info "Validating environment using unified schema..."
        cd "$PROJECT_ROOT"
        if node scripts/utils/validate-env.js; then
            print_status "Environment validation passed"
        else
            print_error "Environment validation failed"
            exit 1
        fi
    else
        print_warning "Node.js or validation script not available, skipping schema validation"
        
        # Basic validation for critical variables
        validate_required_vars
    fi
}

# Basic validation for required variables
validate_required_vars() {
    local REQUIRED_VARS=(
        "DATABASE_URL"
        "NEXTAUTH_SECRET"
        "OIDC_CLIENT_SECRET"
        "OIDC_ADMIN_CLIENT_SECRET"
        "FIREBASE_PROJECT_ID"
        "FIREBASE_PRIVATE_KEY"
        "FIREBASE_CLIENT_EMAIL"
    )
    
    print_info "Performing basic validation of required variables..."
    
    local missing_vars=()
    for var in "${REQUIRED_VARS[@]}"; do
        if [ -z "${!var}" ]; then
            missing_vars+=("$var")
        fi
    done
    
    if [ ${#missing_vars[@]} -gt 0 ]; then
        print_error "Missing required environment variables:"
        for var in "${missing_vars[@]}"; do
            echo "  - $var"
        done
        exit 1
    fi
    
    print_status "Basic validation passed - all required variables are set"
}

# Get all environment variables needed for Cloud Run deployment
get_backend_env_vars() {
    local ENV_NAME=$1
    
    # Core environment variables
    local ENV_VARS="ENV=$ENV_NAME,NODE_ENV=$ENV_NAME,RUST_ENV=$ENV_NAME"
    
    # Set log level based on environment
    if [ "$ENV_NAME" = "production" ]; then
        ENV_VARS+=",RUST_LOG=info"
    else
        ENV_VARS+=",RUST_LOG=debug"
    fi
    
    # Required variables
    ENV_VARS+=",DATABASE_URL=${DATABASE_URL}"
    ENV_VARS+=",NEXTAUTH_SECRET=${NEXTAUTH_SECRET}"
    ENV_VARS+=",FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}"
    ENV_VARS+=",FIREBASE_PRIVATE_KEY=${FIREBASE_PRIVATE_KEY}"
    ENV_VARS+=",FIREBASE_CLIENT_EMAIL=${FIREBASE_CLIENT_EMAIL}"
    ENV_VARS+=",OIDC_CLIENT_SECRET=${OIDC_CLIENT_SECRET}"
    ENV_VARS+=",OIDC_ADMIN_CLIENT_SECRET=${OIDC_ADMIN_CLIENT_SECRET}"
    
    # URL configuration
    ENV_VARS+=",BACKEND_URL=${BACKEND_URL}"
    ENV_VARS+=",FRONTEND_URL=${FRONTEND_URL}"
    ENV_VARS+=",ADMIN_FRONTEND_URL=${ADMIN_FRONTEND_URL}"
    
    # Optional variables (only set if they exist)
    [ -n "$MUSEPAY_PARTNER_ID" ] && ENV_VARS+=",MUSEPAY_PARTNER_ID=${MUSEPAY_PARTNER_ID}"
    [ -n "$MUSEPAY_PRIVATE_KEY" ] && ENV_VARS+=",MUSEPAY_PRIVATE_KEY=${MUSEPAY_PRIVATE_KEY}"
    [ -n "$REDIS_URL" ] && ENV_VARS+=",REDIS_URL=${REDIS_URL}"
    
    # Environment-specific optimizations
    if [ "$ENV_NAME" = "production" ]; then
        ENV_VARS+=",DATABASE_ACQUIRE_TIMEOUT=90"
        ENV_VARS+=",DATABASE_MAX_CONNECTIONS=2"
        ENV_VARS+=",DATABASE_MIN_CONNECTIONS=1"
        ENV_VARS+=",SKIP_DB_TEST=true"
    elif [ "$ENV_NAME" = "development" ]; then
        ENV_VARS+=",DATABASE_ACQUIRE_TIMEOUT=30"
        ENV_VARS+=",SKIP_DB_TEST=true"
    fi
    
    echo "$ENV_VARS"
}

# Get all environment variables needed for Frontend/Admin deployment
get_frontend_env_vars() {
    local ENV_NAME=$1
    
    # Core environment variables
    local ENV_VARS="NODE_ENV=$ENV_NAME"
    
    # Client-accessible URLs
    ENV_VARS+=",NEXT_PUBLIC_BACKEND_URL=${NEXT_PUBLIC_BACKEND_URL}"
    ENV_VARS+=",NEXT_PUBLIC_APP_URL=${NEXT_PUBLIC_APP_URL}"
    ENV_VARS+=",NEXT_PUBLIC_ADMIN_URL=${NEXT_PUBLIC_ADMIN_URL}"
    ENV_VARS+=",NEXT_PUBLIC_OAUTH_CLIENT_ID=${NEXT_PUBLIC_OAUTH_CLIENT_ID}"
    
    # Firebase client configuration (optional)
    [ -n "$NEXT_PUBLIC_FIREBASE_API_KEY" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_API_KEY=${NEXT_PUBLIC_FIREBASE_API_KEY}"
    [ -n "$NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=${NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN}"
    [ -n "$NEXT_PUBLIC_FIREBASE_PROJECT_ID" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_PROJECT_ID=${NEXT_PUBLIC_FIREBASE_PROJECT_ID}"
    [ -n "$NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=${NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET}"
    [ -n "$NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=${NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID}"
    [ -n "$NEXT_PUBLIC_FIREBASE_APP_ID" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_APP_ID=${NEXT_PUBLIC_FIREBASE_APP_ID}"
    [ -n "$NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID" ] && ENV_VARS+=",NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID=${NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID}"
    
    echo "$ENV_VARS"
}

# Get Docker build args for frontend/admin builds
get_docker_build_args() {
    local ENV_NAME=$1
    
    local BUILD_ARGS="--build-arg NODE_ENV=$ENV_NAME"
    BUILD_ARGS+=" --build-arg NEXT_PUBLIC_BACKEND_URL=${NEXT_PUBLIC_BACKEND_URL}"
    BUILD_ARGS+=" --build-arg NEXT_PUBLIC_APP_URL=${NEXT_PUBLIC_APP_URL}"
    BUILD_ARGS+=" --build-arg NEXT_PUBLIC_ADMIN_URL=${NEXT_PUBLIC_ADMIN_URL}"
    BUILD_ARGS+=" --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=${NEXT_PUBLIC_OAUTH_CLIENT_ID}"
    
    # Firebase build args (optional)
    [ -n "$NEXT_PUBLIC_FIREBASE_API_KEY" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_API_KEY=${NEXT_PUBLIC_FIREBASE_API_KEY}"
    [ -n "$NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=${NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN}"
    [ -n "$NEXT_PUBLIC_FIREBASE_PROJECT_ID" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_PROJECT_ID=${NEXT_PUBLIC_FIREBASE_PROJECT_ID}"
    [ -n "$NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=${NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET}"
    [ -n "$NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=${NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID}"
    [ -n "$NEXT_PUBLIC_FIREBASE_APP_ID" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_APP_ID=${NEXT_PUBLIC_FIREBASE_APP_ID}"
    [ -n "$NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID" ] && BUILD_ARGS+=" --build-arg NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID=${NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID}"
    
    echo "$BUILD_ARGS"
}

# Check if required tools are installed
check_deployment_prerequisites() {
    print_info "Checking deployment prerequisites..."
    
    # Check Google Cloud CLI
    if ! command -v gcloud &> /dev/null; then
        print_error "Google Cloud CLI is required but not installed"
        print_info "Install from: https://cloud.google.com/sdk/docs/install"
        exit 1
    fi
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is required but not installed"
        print_info "Install from: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    # Check authentication
    if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q "@"; then
        print_error "Please authenticate with Google Cloud first:"
        echo "   gcloud auth login"
        echo "   gcloud auth application-default login"
        exit 1
    fi
    
    print_status "All prerequisites satisfied"
}

# Get environment-specific configuration
get_env_config() {
    local ENV_NAME=$1
    
    case $ENV_NAME in
        "development")
            export PROJECT_ID="epsx-469400"  # Use production project for development services
            export URL_SUFFIX="dev-"
            export RESOURCES_BACKEND="--memory=1Gi --cpu=1 --min-instances=0 --max-instances=3"
            export RESOURCES_FRONTEND="--memory=1Gi --cpu=1 --min-instances=0 --max-instances=3"
            export RESOURCES_ADMIN="--memory=1Gi --cpu=1 --min-instances=0 --max-instances=3"
            ;;
        "staging")
            export PROJECT_ID="epsx-staging"
            export URL_SUFFIX="staging-"
            export RESOURCES_BACKEND="--memory=2Gi --cpu=2 --min-instances=1 --max-instances=5"
            export RESOURCES_FRONTEND="--memory=1Gi --cpu=1 --min-instances=1 --max-instances=5"
            export RESOURCES_ADMIN="--memory=1Gi --cpu=1 --min-instances=1 --max-instances=3"
            ;;
        "production")
            export PROJECT_ID="epsx-469400"  # Your existing production project
            export URL_SUFFIX=""
            export RESOURCES_BACKEND="--memory=4Gi --cpu=4 --min-instances=1 --max-instances=10"
            export RESOURCES_FRONTEND="--memory=2Gi --cpu=2 --min-instances=1 --max-instances=10"
            export RESOURCES_ADMIN="--memory=2Gi --cpu=2 --min-instances=1 --max-instances=10"
            ;;
        *)
            print_error "Invalid environment: $ENV_NAME"
            exit 1
            ;;
    esac
    
    export REGION="us-central1"
    export REGISTRY="us-central1-docker.pkg.dev"
    
    print_status "Configuration loaded for $ENV_NAME environment"
    print_info "Project ID: $PROJECT_ID"
    print_info "Region: $REGION"
    print_info "Registry: $REGISTRY"
}

# Export functions for use in other scripts
export -f print_status print_error print_warning print_info
export -f load_and_validate_env validate_required_vars
export -f get_backend_env_vars get_frontend_env_vars get_docker_build_args
export -f check_deployment_prerequisites get_env_config