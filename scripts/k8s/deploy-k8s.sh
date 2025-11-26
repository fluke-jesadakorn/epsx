#!/bin/bash

# EPSX Kubernetes Deployment Script
# Usage: ./scripts/k8s/deploy-k8s.sh [environment] [options]
# Examples:
#   ./scripts/k8s/deploy-k8s.sh production
#   ./scripts/k8s/deploy-k8s.sh staging --dry-run
#   ./scripts/k8s/deploy-k8s.sh development --build

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
K8S_DIR="$PROJECT_ROOT/k8s"

# Default values
ENVIRONMENT=${1:-production}
DRY_RUN=false
BUILD_IMAGES=false
SKIP_SECRETS=false
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
shift
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --build)
            BUILD_IMAGES=true
            shift
            ;;
        --skip-secrets)
            SKIP_SECRETS=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [environment] [options]"
            echo ""
            echo "Arguments:"
            echo "  environment    Target environment (development, staging, production)"
            echo ""
            echo "Options:"
            echo "  --dry-run      Show what would be deployed without making changes"
            echo "  --build        Build Docker images before deployment"
            echo "  --skip-secrets Skip secrets management (use existing secrets)"
            echo "  --verbose      Verbose output"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(development|staging|production)$ ]]; then
    error "Invalid environment: $ENVIRONMENT. Must be one of: development, staging, production"
    exit 1
fi

# Set namespace
NAMESPACE="epsx-$ENVIRONMENT"
if [[ "$ENVIRONMENT" == "production" ]]; then
    NAMESPACE="epsx"
fi

log "Starting EPSX Kubernetes deployment"
log "Environment: $ENVIRONMENT"
log "Namespace: $NAMESPACE"
log "Project root: $PROJECT_ROOT"

# Check prerequisites
command -v kubectl >/dev/null 2>&1 || { error "kubectl is required but not installed"; exit 1; }
command -v kustomize >/dev/null 2>&1 || { error "kustomize is required but not installed"; exit 1; }

# Verify cluster connectivity
if ! kubectl cluster-info >/dev/null 2>&1; then
    error "Cannot connect to Kubernetes cluster"
    exit 1
fi

# Build Docker images if requested
if [[ "$BUILD_IMAGES" == "true" ]]; then
    log "Building Docker images..."
    cd "$PROJECT_ROOT"

    # Build backend
    log "Building backend image..."
    docker build -t epsx-backend:latest ./apps/backend/

    # Build frontend
    log "Building frontend image..."
    docker build -t epsx-frontend:latest ./apps/frontend/

    # Build admin frontend
    log "Building admin frontend image..."
    docker build -t epsx-admin-frontend:latest ./apps/admin-frontend/

    success "Docker images built successfully"
fi

# Create namespace if it doesn't exist
log "Ensuring namespace exists: $NAMESPACE"
if [[ "$DRY_RUN" == "false" ]]; then
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
fi

# Manage secrets if not skipped
if [[ "$SKIP_SECRETS" == "false" ]]; then
    log "Managing secrets for environment: $ENVIRONMENT"

    if [[ "$DRY_RUN" == "false" ]]; then
        # Create secrets from environment files
        if [[ -f "$PROJECT_ROOT/environments/.secrets.$ENVIRONMENT" ]]; then
            # Create secret from environment file
            kubectl create secret generic epsx-secrets \
                --namespace="$NAMESPACE" \
                --from-env-file="$PROJECT_ROOT/environments/.secrets.$ENVIRONMENT" \
                --dry-run=client -o yaml | kubectl apply -f -
            success "Secrets applied successfully"
        else
            warn "Secrets file not found: $PROJECT_ROOT/environments/.secrets.$ENVIRONMENT"
            warn "Please create it or use --skip-secrets to use existing secrets"
        fi
    fi
fi

# Deploy with Kustomize
log "Deploying Kubernetes manifests for environment: $ENVIRONMENT"
log "Kustomize directory: $K8S_DIR/overlays/$ENVIRONMENT"

# Check if overlay exists
if [[ ! -d "$K8S_DIR/overlays/$ENVIRONMENT" ]]; then
    warn "Environment overlay not found, using base configuration"
    KUSTOMIZE_DIR="$K8S_DIR/base"
else
    KUSTOMIZE_DIR="$K8S_DIR/overlays/$ENVIRONMENT"
fi

# Build and apply manifests
if [[ "$VERBOSE" == "true" ]]; then
    log "Kustomize build output:"
    kustomize build "$KUSTOMIZE_DIR"
    echo ""
fi

if [[ "$DRY_RUN" == "true" ]]; then
    log "DRY RUN: Would deploy the following manifests:"
    kustomize build "$KUSTOMIZE_DIR"
else
    log "Applying manifests..."
    kustomize build "$KUSTOMIZE_DIR" | kubectl apply --namespace="$NAMESPACE" -f -

    # Wait for deployments to be ready
    log "Waiting for deployments to be ready..."

    # List of deployments to wait for
    DEPLOYMENTS=("backend" "frontend" "admin-frontend")

    for deployment in "${DEPLOYMENTS[@]}"; do
        log "Waiting for $deployment deployment..."
        kubectl rollout status deployment/$deployment --namespace="$NAMESPACE" --timeout=300s
        success "$deployment deployment is ready"
    done

    # Wait for statefulsets
    STATEFULSETS=("postgres" "redis")
    for statefulset in "${STATEFULSETS[@]}"; do
        log "Waiting for $statefulset statefulset..."
        kubectl rollout status statefulset/$statefulset --namespace="$NAMESPACE" --timeout=300s
        success "$statefulset statefulset is ready"
    done
fi

# Show deployment status
log "Deployment status:"
if [[ "$DRY_RUN" == "false" ]]; then
    kubectl get pods --namespace="$NAMESPACE" -l app.kubernetes.io/name=epsx
    kubectl get services --namespace="$NAMESPACE" -l app.kubernetes.io/name=epsx
    kubectl get ingress --namespace="$NAMESPACE"

    # Show URLs if ingress is ready
    INGRESS_IP=$(kubectl get ingress epsx-ingress --namespace="$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
    INGRESS_HOSTNAME=$(kubectl get ingress epsx-ingress --namespace="$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "")

    if [[ -n "$INGRESS_IP" ]] || [[ -n "$INGRESS_HOSTNAME" ]]; then
        HOST=${INGRESS_HOSTNAME:-$INGRESS_IP}
        success "Deployment completed successfully!"
        echo ""
        log "Application URLs:"
        echo "  Frontend: https://epsx.io"
        echo "  API: https://api.epsx.io"
        echo "  Admin: https://admin.epsx.io"
        if [[ -n "$INGRESS_IP" ]]; then
            echo "  Ingress IP: $INGRESS_IP"
        fi
    else
        warn "Ingress IP not yet available. Check status with: kubectl get ingress epsx-ingress --namespace=$NAMESPACE"
    fi
else
    warn "DRY RUN completed. No changes were made."
fi

log "EPSX Kubernetes deployment completed successfully!"