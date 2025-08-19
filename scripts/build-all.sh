#!/bin/bash

# EPSX - Smart Build All Containers
# Automatically detects environment and routes to appropriate build script

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Script directory and root detection
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}🚀 EPSX Smart Build System${NC}"

# Load environment detection
if [[ -f "$SCRIPT_DIR/env-manager.sh" ]]; then
    source "$SCRIPT_DIR/env-manager.sh"
else
    echo -e "${RED}❌ env-manager.sh not found. Using fallback environment detection.${NC}"
fi

# Detect current environment
DETECTED_ENV=$(detect_environment 2>/dev/null || echo "development")

# Check for explicit build target override
BUILD_TARGET="${BUILD_TARGET:-$DETECTED_ENV}"

echo -e "${PURPLE}Environment Detection:${NC}"
echo -e "  Detected Environment: ${GREEN}$DETECTED_ENV${NC}"
echo -e "  Build Target: ${GREEN}$BUILD_TARGET${NC}"
echo

# Route to appropriate build script
case "$BUILD_TARGET" in
    development|dev)
        echo -e "${BLUE}🔧 Routing to development build...${NC}"
        exec "$SCRIPT_DIR/build-dev.sh" "$@"
        ;;
    staging)
        echo -e "${BLUE}🏗️  Routing to staging build...${NC}"
        exec "$SCRIPT_DIR/build-staging.sh" "$@"
        ;;
    production|prod)
        echo -e "${BLUE}🏭 Routing to production build...${NC}"
        exec "$SCRIPT_DIR/build-prod.sh" "$@"
        ;;
    cloud-run)
        echo -e "${BLUE}☁️  Routing to Cloud Run optimized build...${NC}"
        # For Cloud Run, use production build with cloud-run environment
        BUILD_TARGET=cloud-run exec "$SCRIPT_DIR/build-prod.sh" "$@"
        ;;
    legacy|fallback)
        echo -e "${YELLOW}⚠️  Using legacy build method...${NC}"
        # Fall back to original build logic below
        ;;
    *)
        echo -e "${RED}❌ Unknown build target: $BUILD_TARGET${NC}"
        echo -e "${YELLOW}Available targets: development, staging, production, cloud-run${NC}"
        echo -e "${YELLOW}Override with: BUILD_TARGET=<target> ./scripts/build-all.sh${NC}"
        exit 1
        ;;
esac

# Legacy build logic (fallback)
echo -e "${YELLOW}Using legacy build method...${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="${BUILD_VERSION:-latest}"

# Image names
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${YELLOW}Legacy Build Configuration:${NC}"
echo -e "  Project: $PROJECT_ID${NC}"
echo -e "  Version: $VERSION${NC}"
echo

# Use Docker engine
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    echo -e "${BLUE}🐳 Using Docker engine${NC}"
    CONTAINER_CMD="docker"
else
    echo -e "${RED}❌ Docker not available. Please install Docker.${NC}"
    exit 1
fi

# Function to build container with Docker
build_container() {
    local app_name=$1
    local dockerfile_path=$2
    local image_name=$3
    local context_path=$4
    
    echo -e "${BLUE}📦 Building $app_name with Docker...${NC}"
    
    docker build \
        --platform linux/amd64 \
        --tag "$image_name" \
        --file "$dockerfile_path" \
        "$context_path"
    
    echo -e "${GREEN}✅ $app_name completed${NC}"
}

# Start builds in parallel
echo -e "${BLUE}🏗️  Building containers...${NC}"

# Frontend build (use root context for pnpm-lock.yaml access)
(
    build_container "Frontend" "apps/frontend/Dockerfile" "$FRONTEND_IMAGE" "."
) &
FRONTEND_PID=$!

# Admin build (use root context for pnpm-lock.yaml access)
(
    build_container "Admin" "apps/admin-frontend/Dockerfile" "$ADMIN_IMAGE" "."
) &
ADMIN_PID=$!

# Backend build
(
    build_container "Backend" "apps/backend/Dockerfile" "$BACKEND_IMAGE" "apps/backend"
) &
BACKEND_PID=$!

# Wait for all builds to complete
echo -e "${YELLOW}⏳ Waiting for builds...${NC}"

wait $FRONTEND_PID
wait $ADMIN_PID  
wait $BACKEND_PID

echo
echo -e "${GREEN}🎉 All containers built successfully!${NC}"
echo
echo -e "${BLUE}Images built:${NC}"
echo -e "  Frontend: $FRONTEND_IMAGE"
echo -e "  Admin:    $ADMIN_IMAGE"
echo -e "  Backend:  $BACKEND_IMAGE"
echo
echo -e "${YELLOW}Next: ./scripts/push-all.sh${NC}"