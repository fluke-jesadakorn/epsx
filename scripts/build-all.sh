#!/bin/bash

# EPSX - Build All Containers with Apple Container Engine (15x Performance)
# Builds frontend, admin-frontend, and backend containers in parallel using Apple's optimized container engine

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="${BUILD_VERSION:-latest}"

# Image names
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${BLUE}🚀 Building EPSX containers locally...${NC}"
echo -e "${YELLOW}Project: $PROJECT_ID${NC}"
echo -e "${YELLOW}Version: $VERSION${NC}"
echo

# Use Apple Container Engine exclusively for 15x performance improvement
if command -v container >/dev/null 2>&1 && container system status >/dev/null 2>&1; then
    echo -e "${BLUE}🚀 Using Apple container engine (optimized for Apple Silicon)${NC}"
    CONTAINER_CMD="container"
else
    echo -e "${RED}❌ Apple container engine not available. Please install container tooling.${NC}"
    echo -e "${YELLOW}Install: https://github.com/containers/podman or use OrbStack${NC}"
    exit 1
fi

# Function to build container with Apple container engine
build_container() {
    local app_name=$1
    local dockerfile_path=$2
    local image_name=$3
    local context_path=$4
    
    echo -e "${BLUE}📦 Building $app_name with Apple container engine...${NC}"
    
    container build \
        --arch amd64 \
        --os linux \
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