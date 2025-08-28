#!/bin/bash

# Deploy Frontend to Google Cloud Run
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Deploying Frontend to Google Cloud Run${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
SERVICE_NAME="epsx-frontend"

# Use latest image
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:latest"

echo -e "${YELLOW}Deploy Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Service: $SERVICE_NAME"
echo -e "  Image: $FRONTEND_IMAGE"
echo

# Deploy to Cloud Run
echo -e "${BLUE}Deploying Frontend...${NC}"
if gcloud run deploy "$SERVICE_NAME" \
    --image="$FRONTEND_IMAGE" \
    --platform=managed \
    --region="$REGION" \
    --allow-unauthenticated \
    --port=3000 \
    --memory=2Gi \
    --cpu=2 \
    --min-instances=0 \
    --max-instances=10 \
    --timeout=300s \
    --concurrency=80 \
    --execution-environment=gen2 \
    --set-env-vars="NODE_ENV=production,NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum,NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io,APP_URL=https://epsx.io" \
    --quiet; then
    
    # Get service URL
    SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --region="$REGION" --format="value(status.url)" 2>/dev/null || echo "")
    
    echo -e "${GREEN}✅ Frontend deployed successfully!${NC}"
    echo -e "${GREEN}   Service URL: $SERVICE_URL${NC}"
    echo -e "${GREEN}   Custom Domain: https://epsx.io${NC}"
    
    exit 0
else
    echo -e "${RED}❌ Frontend deployment failed${NC}"
    exit 1
fi