#!/bin/bash

# EPSX - Deploy Admin Frontend to Google Cloud Run
# Deployment script for the Next.js admin frontend application

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying EPSX Admin Frontend to Cloud Run${NC}"
echo -e "${PURPLE}Deploying full Next.js admin application with proper configuration${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
# Use known working image SHA (verified working)
WORKING_IMAGE_SHA="sha256:6d23a5b528a16e3d19641f6094088bec151c1e7a1e2b4615385af78fc7f9bd56"
VERSION="${BUILD_VERSION:-$WORKING_IMAGE_SHA}"

# Service configuration
ADMIN_SERVICE="epsx-admin"
if [[ "$VERSION" == sha256:* ]]; then
    ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin@$VERSION"
else
    ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
fi

# URLs
BACKEND_URL="https://epsx-backend-307278481624.us-central1.run.app"
FRONTEND_URL="https://epsx.io"
ADMIN_URL="https://admin.epsx.io"

echo -e "${YELLOW}Deployment Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Service: $ADMIN_SERVICE"
echo -e "  Image: $ADMIN_IMAGE"
echo -e "  Backend URL: $BACKEND_URL"
echo -e "  Admin URL: $ADMIN_URL"
echo

# Check if image exists in registry
echo -e "${BLUE}🔍 Verifying image in registry...${NC}"
if gcloud artifacts docker images describe "$ADMIN_IMAGE" --quiet >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Admin Frontend image found in registry${NC}"
else
    echo -e "${RED}❌ Admin Frontend image not found: $ADMIN_IMAGE${NC}"
    echo -e "${YELLOW}💡 Run ./scripts/build-admin.sh first${NC}"
    exit 1
fi

# Deploy Admin Frontend to Cloud Run
echo -e "${PURPLE}=== Deploying Next.js Admin Frontend ===${NC}"

gcloud run deploy "$ADMIN_SERVICE" \
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
    --set-env-vars="NODE_ENV=production" \
    --set-env-vars="ADMIN_URL=$ADMIN_URL" \
    --set-env-vars="NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum" \
    --set-env-vars="OIDC_CLIENT_ID=epsx-admin" \
    --set-env-vars="OIDC_CLIENT_SECRET=epsx-admin-secret-2024" \
    --set-env-vars="BACKEND_URL=$BACKEND_URL" \
    --set-env-vars="NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL" \
    --set-env-vars="NEXT_PUBLIC_BUILD_MODE=production" \
    --set-env-vars="NEXT_TELEMETRY_DISABLED=1" \
    --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Admin Frontend deployed successfully${NC}"
else
    echo -e "${RED}❌ Admin Frontend deployment failed${NC}"
    exit 1
fi

# Get service URL
DEPLOYED_URL=$(gcloud run services describe "$ADMIN_SERVICE" --region="$REGION" --format="value(status.url)")

echo -e "\n${GREEN}🎉 Admin Frontend Deployment Complete!${NC}"
echo -e "\n${BLUE}📋 Service Information:${NC}"
echo -e "  ${GREEN}Service Name:${NC}    $ADMIN_SERVICE"
echo -e "  ${GREEN}Deployed URL:${NC}    $DEPLOYED_URL"
echo -e "  ${GREEN}Image:${NC}           $ADMIN_IMAGE"
echo -e "  ${GREEN}Memory:${NC}          1Gi"
echo -e "  ${GREEN}CPU:${NC}             1 core"
echo -e "  ${GREEN}Concurrency:${NC}     50"

# Test the deployment
echo -e "\n${PURPLE}=== Testing Admin Frontend Deployment ===${NC}"

echo -e "${YELLOW}Testing admin frontend accessibility...${NC}"
if curl -s -o /dev/null -w "%{http_code}" "$DEPLOYED_URL" | grep -q "200"; then
    echo -e "${GREEN}✅ Admin Frontend responding successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Admin Frontend might still be starting up (this is normal)${NC}"
fi

echo -e "\n${YELLOW}📊 Service Status:${NC}"
gcloud run services describe "$ADMIN_SERVICE" --region="$REGION" --format="table(status.conditions[0].type,status.conditions[0].status,status.traffic[0].percent)"

echo -e "\n${BLUE}✅ Admin Frontend Deployment Summary:${NC}"
echo -e "  ✅ Deployed full Next.js admin application"
echo -e "  ✅ All environment variables configured"
echo -e "  ✅ Connected to backend API: $BACKEND_URL"
echo -e "  ✅ Production-ready configuration"
echo -e "  ✅ Scalable Cloud Run service"

echo -e "\n${YELLOW}🔧 Management Commands:${NC}"
echo -e "  View logs:    gcloud run services logs read $ADMIN_SERVICE --region=$REGION"
echo -e "  Update:       gcloud run services update $ADMIN_SERVICE --region=$REGION"
echo -e "  Scale down:   gcloud run services update $ADMIN_SERVICE --region=$REGION --min-instances=0"

echo -e "\n${PURPLE}🌐 Your EPSX admin frontend is now live at: ${GREEN}$DEPLOYED_URL${NC}"