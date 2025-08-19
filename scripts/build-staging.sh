#!/bin/bash

# EPSX Staging Build Script
# Production-like builds optimized for staging deployment

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

echo -e "${BLUE}🚀 EPSX Staging Build${NC}"

# Load staging environment
source "$SCRIPT_DIR/env-manager.sh"
load_environment "staging" "" true

# Staging build configuration
export NODE_ENV=production
export RUST_ENV=staging
export ENV=staging
export NEXT_PUBLIC_BUILD_MODE=staging

# Staging Docker build args
BUILD_ARGS=(
    --build-arg "NODE_ENV=production"
    --build-arg "RUST_ENV=staging"
    --build-arg "NEXT_PUBLIC_BUILD_MODE=staging"
    --build-arg "ENABLE_DEBUG=false"
    --build-arg "BUILD_TARGET=staging"
)

# Staging image tags
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="staging-$(date +%Y%m%d-%H%M%S)"

FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${YELLOW}Staging Build Configuration:${NC}"
echo -e "  Environment: ${GREEN}staging${NC}"
echo -e "  Build Mode: ${GREEN}production-optimized${NC}"
echo -e "  Debug: ${YELLOW}limited${NC}"
echo -e "  Source Maps: ${GREEN}enabled${NC}"
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

# Function to build container for staging
build_container_staging() {
    local app_name=$1
    local dockerfile_path=$2
    local image_name=$3
    local context_path=$4
    
    echo -e "${BLUE}📦 Building $app_name (staging)...${NC}"
    
    # Use production Dockerfile for staging builds
    local dockerfile="$dockerfile_path"
    echo -e "  Using production Dockerfile with staging config: ${dockerfile}"
    
    $CONTAINER_CMD build \
        --platform linux/amd64 \
        --tag "$image_name" \
        --tag "$app_name:staging-latest" \
        --file "$dockerfile" \
        "${BUILD_ARGS[@]}" \
        "$context_path"
    
    echo -e "${GREEN}✅ $app_name staging build completed${NC}"
}

# Validate staging environment before building
echo -e "${BLUE}🔍 Validating staging environment...${NC}"
if [[ -z "${DATABASE_URL:-}" ]]; then
    echo -e "${RED}❌ DATABASE_URL not set for staging${NC}"
    exit 1
fi

if [[ -z "${NEXTAUTH_SECRET:-}" ]]; then
    echo -e "${RED}❌ NEXTAUTH_SECRET not set for staging${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Staging environment validation passed${NC}"
echo

# Start builds in parallel
echo -e "${BLUE}🏗️  Building containers for staging...${NC}"

# Frontend build
(
    build_container_staging "Frontend" "apps/frontend/Dockerfile" "$FRONTEND_IMAGE" "."
) &
FRONTEND_PID=$!

# Admin build  
(
    build_container_staging "Admin" "apps/admin-frontend/Dockerfile" "$ADMIN_IMAGE" "."
) &
ADMIN_PID=$!

# Backend build
(
    build_container_staging "Backend" "apps/backend/Dockerfile" "$BACKEND_IMAGE" "apps/backend"
) &
BACKEND_PID=$!

# Wait for all builds to complete
echo -e "${YELLOW}⏳ Waiting for staging builds...${NC}"

wait $FRONTEND_PID
wait $ADMIN_PID
wait $BACKEND_PID

echo
echo -e "${GREEN}🎉 Staging builds completed successfully!${NC}"
echo
echo -e "${BLUE}Staging Images:${NC}"
echo -e "  Frontend: $FRONTEND_IMAGE"
echo -e "  Admin:    $ADMIN_IMAGE"
echo -e "  Backend:  $BACKEND_IMAGE"
echo
echo -e "${BLUE}Local Tags:${NC}"
echo -e "  Frontend: frontend:staging-latest"
echo -e "  Admin:    admin:staging-latest"  
echo -e "  Backend:  backend:staging-latest"
echo
echo -e "${YELLOW}Staging Features:${NC}"
echo -e "  ✅ Production optimizations"
echo -e "  ✅ Source maps for debugging"
echo -e "  ✅ Security hardening"
echo -e "  ✅ Performance monitoring"
echo -e "  ❌ Debug symbols removed"
echo
echo -e "${YELLOW}Next Steps for Staging:${NC}"
echo -e "  ${GREEN}./scripts/push-all.sh${NC}                         # Push to registry"
echo -e "  ${GREEN}./scripts/deploy-staging.sh${NC}                   # Deploy to staging"
echo -e "  ${GREEN}./.devtools/performance-monitor.sh staging${NC}    # Monitor performance"