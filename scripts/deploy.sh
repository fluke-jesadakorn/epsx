#!/bin/bash

# EPSX Unified Deployment Script
# Supports both Cloud Run and Kubernetes deployment
# Usage: ./scripts/deploy.sh [platform] [environment] [options]
# Examples:
#   ./scripts/deploy.sh cloudrun production
#   ./scripts/deploy.sh k8s staging
#   ./scripts/deploy.sh both production --build

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
PLATFORM=${1:-cloudrun}
ENVIRONMENT=${2:-production}
BUILD_IMAGES=false
VERBOSE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1"
}

# Parse command line arguments
shift 2
while [[ $# -gt 0 ]]; do
    case $1 in
        --build)
            BUILD_IMAGES=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [platform] [environment] [options]"
            echo ""
            echo "Arguments:"
            echo "  platform       Target platform (cloudrun, k8s, both)"
            echo "  environment    Target environment (development, staging, production)"
            echo ""
            echo "Options:"
            echo "  --build        Build Docker images before deployment"
            echo "  --verbose      Verbose output"
            echo "  --help         Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 cloudrun production     # Deploy to Cloud Run"
            echo "  $0 k8s staging              # Deploy to Kubernetes"
            echo "  $0 both production --build  # Deploy to both platforms with build"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate platform
if [[ ! "$PLATFORM" =~ ^(cloudrun|k8s|both)$ ]]; then
    error "Invalid platform: $PLATFORM. Must be one of: cloudrun, k8s, both"
    exit 1
fi

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(development|staging|production)$ ]]; then
    error "Invalid environment: $ENVIRONMENT. Must be one of: development, staging, production"
    exit 1
fi

# Function to deploy to Cloud Run
deploy_cloudrun() {
    local env=$1
    log "Deploying to Google Cloud Run - Environment: $env"

    if [[ ! -f "$SCRIPT_DIR/deploy-cloudrun.sh" ]]; then
        error "Cloud Run deployment script not found: $SCRIPT_DIR/deploy-cloudrun.sh"
        return 1
    fi

    # Build images if requested
    if [[ "$BUILD_IMAGES" == "true" ]]; then
        log "Building images for Cloud Run..."
        "$SCRIPT_DIR/../build/local-all.sh"
    fi

    # Deploy to Cloud Run
    local deploy_args=("$env")
    if [[ "$VERBOSE" == "true" ]]; then
        deploy_args+=("--verbose")
    fi

    "$SCRIPT_DIR/deploy-cloudrun.sh" "${deploy_args[@]}"
    success "Cloud Run deployment completed"
}

# Function to deploy to Kubernetes
deploy_kubernetes() {
    local env=$1
    log "Deploying to Kubernetes - Environment: $env"

    if [[ ! -f "$SCRIPT_DIR/k8s/deploy-k8s.sh" ]]; then
        error "Kubernetes deployment script not found: $SCRIPT_DIR/k8s/deploy-k8s.sh"
        return 1
    fi

    # Deploy to Kubernetes
    local deploy_args=("$env")
    if [[ "$BUILD_IMAGES" == "true" ]]; then
        deploy_args+=("--build")
    fi
    if [[ "$VERBOSE" == "true" ]]; then
        deploy_args+=("--verbose")
    fi

    "$SCRIPT_DIR/k8s/deploy-k8s.sh" "${deploy_args[@]}"
    success "Kubernetes deployment completed"
}

# Main deployment logic
log "Starting EPSX deployment"
log "Platform: $PLATFORM"
log "Environment: $ENVIRONMENT"
log "Build images: $BUILD_IMAGES"

case "$PLATFORM" in
    "cloudrun")
        deploy_cloudrun "$ENVIRONMENT"
        ;;
    "k8s")
        deploy_kubernetes "$ENVIRONMENT"
        ;;
    "both")
        log "Deploying to both platforms..."
        deploy_cloudrun "$ENVIRONMENT"
        echo ""
        deploy_kubernetes "$ENVIRONMENT"
        ;;
esac

success "EPSX deployment completed successfully!"
echo ""
log "Deployment summary:"
echo "  Platform: $PLATFORM"
echo "  Environment: $ENVIRONMENT"
echo "  Build images: $BUILD_IMAGES"

if [[ "$PLATFORM" == "both" ]]; then
    echo ""
    log "Both Cloud Run and Kubernetes deployments completed"
fi