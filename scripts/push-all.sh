#!/bin/bash

# EPSX - Push Container Images to Google Artifact Registry
# Pushes frontend, admin-frontend, and backend images using Docker

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

# Verify configuration
echo -e "${BLUE}📋 Configuration:${NC}"
echo -e "  Project ID: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Repository: $REPOSITORY"
echo -e "  Version: $VERSION"
echo

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

# Check repository exists (handle permission errors gracefully)
echo -e "${BLUE}📦 Checking repository status...${NC}"
if ! gcloud artifacts repositories describe $REPOSITORY --location=$REGION >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Repository '$REPOSITORY' not found or no access${NC}"
    echo -e "${YELLOW}📋 Manual setup required - see below${NC}"
    echo -e "${RED}❌ Cannot proceed without repository access${NC}"
    echo
    echo -e "${BLUE}🔧 Manual Configuration Required:${NC}"
    echo -e "1. Go to Google Cloud Console > Artifact Registry"
    echo -e "2. Create repository: $REPOSITORY (Docker, $REGION)"
    echo -e "3. Grant IAM permissions to info@epsx.io:"
    echo -e "   - Artifact Registry Repository Administrator"
    echo -e "   - Artifact Registry Writer"
    echo -e "4. Re-run this script"
    echo
    exit 1
else
    echo -e "${GREEN}✅ Repository '$REPOSITORY' accessible${NC}"
fi

# Verify images exist locally before pushing
echo -e "${BLUE}🔍 Checking local images...${NC}"
for image in "$FRONTEND_IMAGE" "$ADMIN_IMAGE" "$BACKEND_IMAGE"; do
    if ! docker image inspect "$image" >/dev/null 2>&1; then
        echo -e "${RED}❌ Image not found: $image${NC}"
        echo -e "${YELLOW}💡 Run ./scripts/build-all.sh first${NC}"
        exit 1
    fi
done
echo -e "${GREEN}✅ All images found locally${NC}"

# Push images using Docker
echo -e "${BLUE}📤 Pushing images...${NC}"

# Function to push with Docker
push_image() {
    local image=$1
    local name=$2
    
    echo -e "${BLUE}📤 Pushing $name...${NC}"
    
    if docker push "$image"; then
        echo -e "${GREEN}✅ $name pushed${NC}"
    else
        echo -e "${RED}❌ $name push failed${NC}"
        return 1
    fi
}

push_image "$FRONTEND_IMAGE" "Frontend"
push_image "$ADMIN_IMAGE" "Admin"
push_image "$BACKEND_IMAGE" "Backend"

echo
echo -e "${GREEN}🎉 All images pushed successfully!${NC}"
echo
echo -e "${BLUE}Pushed:${NC}"
echo -e "  Frontend: $FRONTEND_IMAGE"
echo -e "  Admin:    $ADMIN_IMAGE"
echo -e "  Backend:  $BACKEND_IMAGE"
echo
echo -e "${YELLOW}Next: ./scripts/deploy-cloudrun.sh${NC}"