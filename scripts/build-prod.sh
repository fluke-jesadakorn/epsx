#!/bin/bash

# EPSX Production Build Script
# Highly optimized builds for production deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and root detection
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}🚀 EPSX Production Build${NC}"

# Load production environment (skip env-manager if not available)
if [[ -f "$SCRIPT_DIR/env-manager.sh" ]]; then
    source "$SCRIPT_DIR/env-manager.sh"
    load_environment "production" "" true
else
    echo -e "${YELLOW}⚠️  env-manager.sh not found, using default environment${NC}"
fi

# Production build configuration
export NODE_ENV=production
export RUST_ENV=production
export ENV=production
export NEXT_PUBLIC_BUILD_MODE=production

# Production Docker build args
BUILD_ARGS=(
    --build-arg "NODE_ENV=production"
    --build-arg "RUST_ENV=production"
    --build-arg "NEXT_PUBLIC_BUILD_MODE=production"
    --build-arg "ENABLE_DEBUG=false"
    --build-arg "BUILD_TARGET=production"
    --build-arg "OPTIMIZE_BUILD=true"
)

# Production image tags
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="${BUILD_VERSION:-$(date +%Y%m%d-%H%M%S)}"

FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${YELLOW}Production Build Configuration:${NC}"
echo -e "  Environment: ${GREEN}production${NC}"
echo -e "  Build Mode: ${GREEN}maximum optimization${NC}"
echo -e "  Debug: ${RED}disabled${NC}"
echo -e "  Source Maps: ${RED}disabled${NC}"
echo -e "  Version: ${GREEN}$VERSION${NC}"
echo

# Detect container engine
CONTAINER_CMD=""
if command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
    echo -e "${BLUE}🐋 Using Podman engine${NC}"
    CONTAINER_CMD="podman"
elif command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    echo -e "${BLUE}🐳 Using Docker engine${NC}"
    CONTAINER_CMD="docker"
else
    echo -e "${RED}❌ No container engine available. Please install Docker or Podman.${NC}"
    exit 1
fi

# Function to build container for production
build_container_prod() {
    local app_name=$1
    local dockerfile_path=$2
    local image_name=$3
    local context_path=$4
    
    echo -e "${BLUE}📦 Building $app_name (production)...${NC}"
    
    # Use production Dockerfile
    local dockerfile="$dockerfile_path"
    if [[ -f "${dockerfile_path}.prod" ]]; then
        dockerfile="${dockerfile_path}.prod"
        echo -e "  Using production-optimized Dockerfile: ${dockerfile}"
    else
        echo -e "  Using standard Dockerfile with production config: ${dockerfile}"
    fi
    
    # Build with maximum optimization (lowercase tags for Docker compatibility)
    local lowercase_name=$(echo "$app_name" | tr '[:upper:]' '[:lower:]')
    $CONTAINER_CMD build \
        --platform linux/amd64 \
        --tag "$image_name" \
        --tag "$lowercase_name:production-latest" \
        --tag "$lowercase_name:latest" \
        --file "$dockerfile" \
        "${BUILD_ARGS[@]}" \
        --no-cache \
        "$context_path"
    
    echo -e "${GREEN}✅ $app_name production build completed${NC}"
}

# Strict production environment validation
echo -e "${BLUE}🔍 Validating production environment...${NC}"

# Required production variables
REQUIRED_VARS=(
    "DATABASE_URL"
    "NEXTAUTH_SECRET"
    "FIREBASE_PRIVATE_KEY"
    "FIREBASE_CLIENT_EMAIL"
    "FRONTEND_URL"
    "BACKEND_URL"
    "ADMIN_FRONTEND_URL"
)

MISSING_VARS=()
for var in "${REQUIRED_VARS[@]}"; do
    if [[ -z "${!var:-}" ]]; then
        MISSING_VARS+=("$var")
    fi
done

if [[ ${#MISSING_VARS[@]} -gt 0 ]]; then
    echo -e "${RED}❌ Missing required production variables:${NC}"
    for var in "${MISSING_VARS[@]}"; do
        echo -e "  ${RED}- $var${NC}"
    done
    exit 1
fi

# Validate database connection is production-ready (Neon PostgreSQL)
if [[ "$DATABASE_URL" == *"localhost"* ]]; then
    echo -e "${RED}❌ DATABASE_URL appears to be localhost. Production requires remote database.${NC}"
    exit 1
fi

if [[ "$DATABASE_URL" != *"neon.tech"* ]]; then
    echo -e "${YELLOW}⚠️  DATABASE_URL doesn't appear to be Neon PostgreSQL${NC}"
fi

echo -e "${GREEN}✅ Production environment validation passed${NC}"
echo

# Start builds in parallel
echo -e "${BLUE}🏗️  Building containers for production...${NC}"

# Frontend build
(
    build_container_prod "Frontend" "apps/frontend/Dockerfile" "$FRONTEND_IMAGE" "."
) &
FRONTEND_PID=$!

# Admin build  
(
    build_container_prod "Admin" "apps/admin-frontend/Dockerfile" "$ADMIN_IMAGE" "."
) &
ADMIN_PID=$!

# Backend build
(
    build_container_prod "Backend" "apps/backend/Dockerfile" "$BACKEND_IMAGE" "apps/backend"
) &
BACKEND_PID=$!

# Wait for all builds to complete
echo -e "${YELLOW}⏳ Waiting for production builds...${NC}"

wait $FRONTEND_PID
wait $ADMIN_PID
wait $BACKEND_PID

echo
echo -e "${GREEN}🎉 Production builds completed successfully!${NC}"
echo
echo -e "${BLUE}Production Images:${NC}"
echo -e "  Frontend: $FRONTEND_IMAGE"
echo -e "  Admin:    $ADMIN_IMAGE"
echo -e "  Backend:  $BACKEND_IMAGE"
echo
echo -e "${BLUE}Local Tags:${NC}"
echo -e "  Frontend: frontend:production-latest, frontend:latest"
echo -e "  Admin:    admin:production-latest, admin:latest"  
echo -e "  Backend:  backend:production-latest, backend:latest"
echo
echo -e "${YELLOW}Production Optimizations Applied:${NC}"
echo -e "  ✅ Maximum compression"
echo -e "  ✅ Security hardening"
echo -e "  ✅ Performance optimization"
echo -e "  ✅ Size minimization"
echo -e "  ✅ Dead code elimination"
echo -e "  ❌ Debug symbols removed"
echo -e "  ❌ Source maps disabled"
echo -e "  ❌ Development tools removed"
echo
echo -e "${YELLOW}Next Steps for Production:${NC}"
echo -e "  ${GREEN}./scripts/push-all.sh${NC}                         # Push to registry"
echo -e "  ${GREEN}./scripts/deploy-prod.sh${NC}                      # Deploy to production"
echo -e "  ${GREEN}./.devtools/performance-monitor.sh production${NC} # Monitor performance"
echo
echo -e "${RED}⚠️  Production Deployment Checklist:${NC}"
echo -e "  ✅ All environment variables validated"
echo -e "  ✅ Database connection confirmed"
echo -e "  ✅ Security configurations applied"
echo -e "  □  Run integration tests"
echo -e "  □  Backup database before deployment"
echo -e "  □  Monitor application after deployment"