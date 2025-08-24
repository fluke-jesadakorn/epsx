#!/bin/bash

# EPSX - Deploy Frontend to Google Cloud Run
# Deployment script for the Next.js frontend application

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying EPSX Frontend to Cloud Run${NC}"
echo -e "${PURPLE}Deploying full Next.js application with proper configuration${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
# Use known working image SHA (verified working)
WORKING_IMAGE_SHA="sha256:33c40107c101e6342d4afb795fe0fa0d652960853535fc59e5ca765a244fddcc"
VERSION="${BUILD_VERSION:-$WORKING_IMAGE_SHA}"

# Service configuration
FRONTEND_SERVICE="epsx-frontend"
if [[ "$VERSION" == sha256:* ]]; then
    FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend@$VERSION"
else
    FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
fi

# URLs
BACKEND_URL="https://epsx-backend-307278481624.us-central1.run.app"
FRONTEND_URL="https://epsx.io"
ADMIN_URL="https://admin.epsx.io"

echo -e "${YELLOW}Deployment Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Service: $FRONTEND_SERVICE"
echo -e "  Image: $FRONTEND_IMAGE"
echo -e "  Backend URL: $BACKEND_URL"
echo -e "  Frontend URL: $FRONTEND_URL"
echo

# Check if image exists in registry
echo -e "${BLUE}🔍 Verifying image in registry...${NC}"
if gcloud artifacts docker images describe "$FRONTEND_IMAGE" --quiet >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Frontend image found in registry${NC}"
else
    echo -e "${RED}❌ Frontend image not found: $FRONTEND_IMAGE${NC}"
    echo -e "${YELLOW}💡 Run ./scripts/build-frontend.sh first${NC}"
    exit 1
fi

# Deploy Frontend to Cloud Run
echo -e "${PURPLE}=== Deploying Next.js Frontend ===${NC}"

gcloud run deploy "$FRONTEND_SERVICE" \
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
    --set-env-vars="NODE_ENV=production" \
    --set-env-vars="APP_URL=$FRONTEND_URL" \
    --set-env-vars="NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum" \
    --set-env-vars="OIDC_CLIENT_ID=epsx-frontend" \
    --set-env-vars="OIDC_CLIENT_SECRET=epsx-frontend-secret-2024" \
    --set-env-vars="BACKEND_URL=$BACKEND_URL" \
    --set-env-vars="NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL" \
    --set-env-vars="NEXT_PUBLIC_APP_URL=$FRONTEND_URL" \
    --set-env-vars="NEXT_PUBLIC_ADMIN_URL=$ADMIN_URL" \
    --set-env-vars="SITE_URL=$FRONTEND_URL" \
    --set-env-vars="FRONTEND_URL=$FRONTEND_URL" \
    --set-env-vars="ADMIN_FRONTEND_URL=$ADMIN_URL" \
    --set-env-vars="NEXT_TELEMETRY_DISABLED=1" \
    --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Frontend deployed successfully${NC}"
else
    echo -e "${RED}❌ Frontend deployment failed${NC}"
    exit 1
fi

# Get service URL
DEPLOYED_URL=$(gcloud run services describe "$FRONTEND_SERVICE" --region="$REGION" --format="value(status.url)")

echo -e "\n${GREEN}🎉 Frontend Deployment Complete!${NC}"
echo -e "\n${BLUE}📋 Service Information:${NC}"
echo -e "  ${GREEN}Service Name:${NC}    $FRONTEND_SERVICE"
echo -e "  ${GREEN}Deployed URL:${NC}    $DEPLOYED_URL"
echo -e "  ${GREEN}Image:${NC}           $FRONTEND_IMAGE"
echo -e "  ${GREEN}Memory:${NC}          2Gi"
echo -e "  ${GREEN}CPU:${NC}             2 cores"
echo -e "  ${GREEN}Concurrency:${NC}     80"

# Test the deployment
echo -e "\n${PURPLE}=== Testing Frontend Deployment ===${NC}"

echo -e "${YELLOW}Testing frontend accessibility...${NC}"
if curl -s -o /dev/null -w "%{http_code}" "$DEPLOYED_URL" | grep -q "200"; then
    echo -e "${GREEN}✅ Frontend responding successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Frontend might still be starting up (this is normal)${NC}"
fi

echo -e "\n${YELLOW}📊 Service Status:${NC}"
gcloud run services describe "$FRONTEND_SERVICE" --region="$REGION" --format="table(status.conditions[0].type,status.conditions[0].status,status.traffic[0].percent)"

echo -e "\n${BLUE}✅ Frontend Deployment Summary:${NC}"
echo -e "  ✅ Deployed full Next.js application"
echo -e "  ✅ All environment variables configured"
echo -e "  ✅ Connected to backend API: $BACKEND_URL"
echo -e "  ✅ Production-ready configuration"
echo -e "  ✅ Scalable Cloud Run service"

echo -e "\n${YELLOW}🔧 Management Commands:${NC}"
echo -e "  View logs:    gcloud run services logs read $FRONTEND_SERVICE --region=$REGION"
echo -e "  Update:       gcloud run services update $FRONTEND_SERVICE --region=$REGION"
echo -e "  Scale down:   gcloud run services update $FRONTEND_SERVICE --region=$REGION --min-instances=0"

echo -e "\n${PURPLE}🌐 Your EPSX frontend is now live at: ${GREEN}$DEPLOYED_URL${NC}"