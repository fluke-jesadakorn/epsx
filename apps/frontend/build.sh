#!/bin/bash

# Build Frontend from apps/frontend directory
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔨 Building Frontend from Latest Source${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Image tags
IMAGE_TAG="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$TIMESTAMP"
LATEST_TAG="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:latest"

echo -e "${YELLOW}Build Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Repository: $REPOSITORY"
echo -e "  Timestamp: $TIMESTAMP"
echo -e "  Image: $IMAGE_TAG"
echo

# Navigate to frontend directory
cd "$(dirname "${BASH_SOURCE[0]}")"
echo -e "${YELLOW}Building from: $(pwd)${NC}"

# Ensure pnpm-lock.yaml exists (copy from root if needed)
if [ ! -f "pnpm-lock.yaml" ]; then
    echo -e "${YELLOW}Copying pnpm-lock.yaml from project root...${NC}"
    cp ../../pnpm-lock.yaml .
fi

# Create cloudbuild config for custom dockerfile
cat > cloudbuild.yaml << EOF
steps:
- name: 'gcr.io/cloud-builders/docker'
  args: 
    - 'build'
    - '-f'
    - 'Dockerfile.standalone'
    - '-t'
    - '$IMAGE_TAG'
    - '--build-arg'
    - 'APP_URL=https://epsx.io'
    - '--build-arg'
    - 'BACKEND_URL=https://api.epsx.io'
    - '--build-arg'
    - 'NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io'
    - '--build-arg'
    - 'NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum'
    - '--build-arg'
    - 'OIDC_CLIENT_ID=epsx-frontend'
    - '--build-arg'
    - 'OIDC_CLIENT_SECRET=epsx-frontend-secret-2024'
    - '.'
  env: ['DOCKER_BUILDKIT=0']
images: ['$IMAGE_TAG']
options:
  machineType: 'E2_HIGHCPU_8'
timeout: '1800s'
EOF

# Build using gcloud builds submit
echo -e "${BLUE}Starting build...${NC}"
if gcloud builds submit . \
    --config=cloudbuild.yaml \
    --timeout=1800s \
    --quiet; then
    
    # Tag as latest
    if gcloud container images add-tag "$IMAGE_TAG" "$LATEST_TAG" --quiet; then
        echo -e "${GREEN}✅ Frontend built successfully!${NC}"
        echo -e "${GREEN}   Image: $IMAGE_TAG${NC}"
        echo -e "${GREEN}   Latest: $LATEST_TAG${NC}"
        
        # Store the image reference
        echo "$IMAGE_TAG" > .frontend-image
        echo "$LATEST_TAG" > .frontend-latest
        
        # Cleanup
        rm -f cloudbuild.yaml
        exit 0
    else
        echo -e "${RED}❌ Failed to tag frontend as latest${NC}"
        rm -f cloudbuild.yaml
        exit 1
    fi
else
    echo -e "${RED}❌ Frontend build failed${NC}"
    rm -f cloudbuild.yaml
    exit 1
fi