#!/bin/bash

# EPSX - Direct Backend Deployment to Cloud Run
# Deploy pre-built backend image directly to Cloud Run

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying EPSX Backend to Cloud Run${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
SERVICE_NAME="epsx-backend"
# Use latest fixed x86_64 image
VERSION="${1:-latest}"  # Allow version override via command line

# Image configuration
if [[ "$VERSION" == sha256:* ]]; then
    BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend@$VERSION"
else
    BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"
fi

echo -e "${YELLOW}Deploy Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Service: $SERVICE_NAME"
echo -e "  Image: $BACKEND_IMAGE"
echo

# Production environment variables
DATABASE_URL="postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require"
NEXTAUTH_SECRET="prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum"
FIREBASE_PROJECT_ID="epsx-449804"

# Deploy to Cloud Run
echo -e "${PURPLE}=== Deploying to Cloud Run ===${NC}"
if gcloud run deploy "$SERVICE_NAME" \
    --image="$BACKEND_IMAGE" \
    --platform=managed \
    --region="$REGION" \
    --allow-unauthenticated \
    --port=8080 \
    --memory=4Gi \
    --cpu=4 \
    --timeout=3600s \
    --set-env-vars="DATABASE_URL=$DATABASE_URL" \
    --set-env-vars="NEXTAUTH_SECRET=$NEXTAUTH_SECRET" \
    --set-env-vars="FIREBASE_PROJECT_ID=$FIREBASE_PROJECT_ID" \
    --set-env-vars="RUST_ENV=production" \
    --set-env-vars="ENV=production" \
    --set-env-vars="DATABASE_MAX_CONNECTIONS=4" \
    --set-env-vars="DATABASE_MIN_CONNECTIONS=1" \
    --set-env-vars="DATABASE_ACQUIRE_TIMEOUT=30" \
    --set-env-vars="DATABASE_IDLE_TIMEOUT=600" \
    --set-env-vars="RUST_LOG=info" \
    --set-env-vars="HOST=0.0.0.0" \
    --min-instances=0 \
    --max-instances=10 \
    --execution-environment=gen2; then
    echo -e "${GREEN}✅ Deployment completed successfully${NC}"
else
    echo -e "${RED}❌ Deployment failed${NC}"
    exit 1
fi

# Get service URL
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
    --region="$REGION" \
    --format="value(status.url)")

if [ -n "$SERVICE_URL" ]; then
    echo -e "${GREEN}✅ Backend deployed successfully!${NC}"
    echo -e "${YELLOW}Service URL: $SERVICE_URL${NC}"
    
    # Test health endpoint
    echo -e "${BLUE}🏥 Testing health endpoint...${NC}"
    if curl -f -s "$SERVICE_URL/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Health check passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Health check failed - checking logs...${NC}"
        gcloud logs read "resource.type=cloud_run_revision AND resource.labels.service_name=$SERVICE_NAME" \
            --limit=5 \
            --format="table(timestamp,severity,textPayload)" \
            --region="$REGION" || echo "No logs available yet"
    fi
else
    echo -e "${RED}❌ Failed to get service URL${NC}"
    exit 1
fi

echo -e "\n${GREEN}🎉 Backend deployment completed!${NC}"
echo -e "\n${YELLOW}Service Details:${NC}"
echo -e "  ✅ URL: $SERVICE_URL"
echo -e "  ✅ Built with native x86_64 in Google Cloud Build"
echo -e "  ✅ Running on Cloud Run with 4GB memory, 4 CPUs"

echo -e "\n${BLUE}📊 Service Status:${NC}"
gcloud run services describe "$SERVICE_NAME" \
    --region="$REGION" \
    --format="table(metadata.name,status.url,status.conditions[0].type,status.conditions[0].status)"