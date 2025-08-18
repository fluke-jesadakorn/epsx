#!/bin/bash

# EPSX - Push Container Images to Google Artifact Registry (Apple Container Engine)
# Pushes frontend, admin-frontend, and backend images using Apple container engine

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-449804}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="${BUILD_VERSION:-latest}"

# Image names
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${BLUE}📤 Pushing containers to Google Artifact Registry...${NC}"
echo -e "${YELLOW}Project: $PROJECT_ID${NC}"
echo

# Configure Apple Container for Artifact Registry
echo -e "${BLUE}🔧 Configuring container registry authentication...${NC}"
gcloud auth configure-docker $REGION-docker.pkg.dev --quiet
# Apple container should inherit the authentication from gcloud

# Create repository if it doesn't exist
echo -e "${BLUE}📦 Setting up repository...${NC}"
gcloud artifacts repositories create $REPOSITORY \
    --repository-format=docker \
    --location=$REGION \
    --description="EPSX containers" \
    --quiet 2>/dev/null || true

# Push images using Apple container engine
echo -e "${BLUE}📤 Pushing images with Apple container...${NC}"

container images push $FRONTEND_IMAGE
echo -e "${GREEN}✅ Frontend pushed${NC}"

container images push $ADMIN_IMAGE
echo -e "${GREEN}✅ Admin pushed${NC}"

container images push $BACKEND_IMAGE
echo -e "${GREEN}✅ Backend pushed${NC}"

echo
echo -e "${GREEN}🎉 All images pushed successfully!${NC}"
echo
echo -e "${BLUE}Pushed:${NC}"
echo -e "  Frontend: $FRONTEND_IMAGE"
echo -e "  Admin:    $ADMIN_IMAGE"
echo -e "  Backend:  $BACKEND_IMAGE"
echo
echo -e "${YELLOW}Next: ./scripts/deploy-cloudrun.sh${NC}"