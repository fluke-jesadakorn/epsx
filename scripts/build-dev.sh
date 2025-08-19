#!/bin/bash

# EPSX Development Build Script
# Fast, development-optimized builds with debug symbols

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

echo -e "${BLUE}🚀 EPSX Development Build${NC}"

# Load development environment
source "$SCRIPT_DIR/env-manager.sh"
load_environment "development" "" true

# Development build configuration
export NODE_ENV=development
export RUST_ENV=development
export ENV=development
export NEXT_PUBLIC_BUILD_MODE=development

# Development Docker build args
BUILD_ARGS=(
    --build-arg "NODE_ENV=development"
    --build-arg "RUST_ENV=development"
    --build-arg "NEXT_PUBLIC_BUILD_MODE=development"
    --build-arg "ENABLE_DEBUG=true"
    --build-arg "BUILD_TARGET=development"
)

# Development image tags
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="dev-$(date +%Y%m%d-%H%M%S)"

FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${YELLOW}Development Build Configuration:${NC}"
echo -e "  Environment: ${GREEN}development${NC}"
echo -e "  Build Mode: ${GREEN}fast development${NC}"
echo -e "  Debug: ${GREEN}enabled${NC}"
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

# Function to build container for development
build_container_dev() {
    local app_name=$1
    local dockerfile_path=$2
    local image_name=$3
    local context_path=$4
    
    echo -e "${BLUE}📦 Building $app_name (development)...${NC}"
    
    # Use development Dockerfile if available, otherwise use production with dev args
    local dockerfile="$dockerfile_path"
    if [[ -f "${dockerfile_path}.dev" ]]; then
        dockerfile="${dockerfile_path}.dev"
        echo -e "  Using development-specific Dockerfile: ${dockerfile}"
    else
        echo -e "  Using production Dockerfile with development args: ${dockerfile}"
    fi
    
    $CONTAINER_CMD build \
        --platform linux/amd64 \
        --tag "$image_name" \
        --tag "$app_name:dev-latest" \
        --file "$dockerfile" \
        "${BUILD_ARGS[@]}" \
        "$context_path"
    
    echo -e "${GREEN}✅ $app_name development build completed${NC}"
}

# Start builds in parallel for faster development iteration
echo -e "${BLUE}🏗️  Building containers for development...${NC}"

# Frontend build
(
    build_container_dev "Frontend" "apps/frontend/Dockerfile" "$FRONTEND_IMAGE" "."
) &
FRONTEND_PID=$!

# Admin build  
(
    build_container_dev "Admin" "apps/admin-frontend/Dockerfile" "$ADMIN_IMAGE" "."
) &
ADMIN_PID=$!

# Backend build
(
    build_container_dev "Backend" "apps/backend/Dockerfile" "$BACKEND_IMAGE" "apps/backend"
) &
BACKEND_PID=$!

# Wait for all builds to complete
echo -e "${YELLOW}⏳ Waiting for development builds...${NC}"

wait $FRONTEND_PID
wait $ADMIN_PID
wait $BACKEND_PID

echo
echo -e "${GREEN}🎉 Development builds completed successfully!${NC}"
echo
echo -e "${BLUE}Development Images:${NC}"
echo -e "  Frontend: $FRONTEND_IMAGE"
echo -e "  Admin:    $ADMIN_IMAGE"
echo -e "  Backend:  $BACKEND_IMAGE"
echo
echo -e "${BLUE}Local Tags:${NC}"
echo -e "  Frontend: frontend:dev-latest"
echo -e "  Admin:    admin:dev-latest"  
echo -e "  Backend:  backend:dev-latest"
echo
echo -e "${YELLOW}Development Features Enabled:${NC}"
echo -e "  ✅ Debug symbols"
echo -e "  ✅ Source maps"
echo -e "  ✅ Development tools"
echo -e "  ✅ Hot reload support"
echo -e "  ✅ Verbose logging"
echo
echo -e "${YELLOW}Next Steps for Development:${NC}"
echo -e "  ${GREEN}docker-compose -f docker-compose.dev.yml up${NC}   # Start development stack"
echo -e "  ${GREEN}pnpm dev${NC}                                      # Start development servers"
echo -e "  ${GREEN}./.devtools/debug-env.sh${NC}                      # Debug environment"