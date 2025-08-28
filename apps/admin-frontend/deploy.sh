#!/bin/bash

# Deploy Admin Frontend to Google Cloud Run
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Deploying Admin Frontend to Google Cloud Run${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
SERVICE_NAME="epsx-admin"

# Use latest image
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:latest"

echo -e "${YELLOW}Deploy Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Service: $SERVICE_NAME"
echo -e "  Image: $ADMIN_IMAGE"
echo

# Deploy to Cloud Run
echo -e "${BLUE}Deploying Admin Frontend...${NC}"
if gcloud run deploy "$SERVICE_NAME" \
    --image="$ADMIN_IMAGE" \
    --platform=managed \
    --region="$REGION" \
    --allow-unauthenticated \
    --port=3000 \
    --memory=1Gi \
    --cpu=1 \
    --min-instances=0 \
    --max-instances=5 \
    --timeout=300s \
    --concurrency=50 \
    --execution-environment=gen2 \
    --set-env-vars="NODE_ENV=production,NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum,NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io,ADMIN_URL=https://admin.epsx.io,NEXTAUTH_URL=https://admin.epsx.io" \
    --quiet; then
    
    # Get service URL
    SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --region="$REGION" --format="value(status.url)" 2>/dev/null || echo "")
    
    echo -e "${GREEN}✅ Admin Frontend deployed successfully!${NC}"
    echo -e "${GREEN}   Service URL: $SERVICE_URL${NC}"
    echo -e "${GREEN}   Custom Domain: https://admin.epsx.io${NC}"
    
    exit 0
else
    echo -e "${RED}❌ Admin Frontend deployment failed${NC}"
    exit 1
fi