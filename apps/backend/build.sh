#!/bin/bash

# Build Backend from apps/backend directory
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔨 Building Backend from Latest Source${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Image tags
IMAGE_TAG="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$TIMESTAMP"
LATEST_TAG="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:latest"

echo -e "${YELLOW}Build Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Repository: $REPOSITORY"
echo -e "  Timestamp: $TIMESTAMP"
echo -e "  Image: $IMAGE_TAG"
echo

# Navigate to backend directory
cd "$(dirname "${BASH_SOURCE[0]}")"
echo -e "${YELLOW}Building from: $(pwd)${NC}"

# Create cloudbuild config for custom dockerfile
cat > cloudbuild.yaml << EOF
steps:
- name: 'gcr.io/cloud-builders/docker'
  args: ['build', '-f', 'Dockerfile.standalone', '-t', '$IMAGE_TAG', '.']
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
        echo -e "${GREEN}✅ Backend built successfully!${NC}"
        echo -e "${GREEN}   Image: $IMAGE_TAG${NC}"
        echo -e "${GREEN}   Latest: $LATEST_TAG${NC}"
        
        # Store the image reference
        echo "$IMAGE_TAG" > .backend-image
        echo "$LATEST_TAG" > .backend-latest
        
        # Cleanup
        rm -f cloudbuild.yaml
        exit 0
    else
        echo -e "${RED}❌ Failed to tag backend as latest${NC}"
        rm -f cloudbuild.yaml
        exit 1
    fi
else
    echo -e "${RED}❌ Backend build failed${NC}"
    rm -f cloudbuild.yaml
    exit 1
fi