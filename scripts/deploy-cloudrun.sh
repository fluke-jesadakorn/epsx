#!/bin/bash

# EPSX - Deploy to Google Cloud Run
# Deploys frontend, admin-frontend, and backend services

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

# Service names
FRONTEND_SERVICE="epsx-frontend"
ADMIN_SERVICE="epsx-admin"
BACKEND_SERVICE="epsx-backend"

# Image names
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

echo -e "${BLUE}🚀 Deploying to Google Cloud Run...${NC}"
echo -e "${YELLOW}Project: $PROJECT_ID${NC}"
echo

# Set project
gcloud config set project $PROJECT_ID

# Load environment variables from .env.shared
if [ -f ".env.shared" ]; then
    echo -e "${BLUE}📝 Loading environment variables...${NC}"
    set -a
    source .env.shared
    set +a
fi

# Deploy Backend Service
echo -e "${BLUE}🔧 Deploying Backend...${NC}"
gcloud run deploy $BACKEND_SERVICE \
    --image=$BACKEND_IMAGE \
    --platform=managed \
    --region=$REGION \
    --allow-unauthenticated \
    --port=8080 \
    --memory=1Gi \
    --cpu=1 \
    --min-instances=1 \
    --max-instances=10 \
    --timeout=300s \
    --set-env-vars="RUST_LOG=info"

BACKEND_URL=$(gcloud run services describe $BACKEND_SERVICE --region=$REGION --format="value(status.url)")
echo -e "${GREEN}✅ Backend deployed: $BACKEND_URL${NC}"

# Deploy Frontend Service  
echo -e "${BLUE}🌐 Deploying Frontend...${NC}"
gcloud run deploy $FRONTEND_SERVICE \
    --image=$FRONTEND_IMAGE \
    --platform=managed \
    --region=$REGION \
    --allow-unauthenticated \
    --port=3000 \
    --memory=512Mi \
    --cpu=1 \
    --min-instances=1 \
    --max-instances=10 \
    --timeout=300s \
    --set-env-vars="NODE_ENV=production,NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL"

FRONTEND_URL=$(gcloud run services describe $FRONTEND_SERVICE --region=$REGION --format="value(status.url)")
echo -e "${GREEN}✅ Frontend deployed: $FRONTEND_URL${NC}"

# Deploy Admin Service
echo -e "${BLUE}⚙️ Deploying Admin...${NC}"
gcloud run deploy $ADMIN_SERVICE \
    --image=$ADMIN_IMAGE \
    --platform=managed \
    --region=$REGION \
    --allow-unauthenticated \
    --port=3000 \
    --memory=512Mi \
    --cpu=1 \
    --min-instances=0 \
    --max-instances=5 \
    --timeout=300s \
    --set-env-vars="NODE_ENV=production,NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL"

ADMIN_URL=$(gcloud run services describe $ADMIN_SERVICE --region=$REGION --format="value(status.url)")
echo -e "${GREEN}✅ Admin deployed: $ADMIN_URL${NC}"

echo
echo -e "${GREEN}🎉 All services deployed successfully!${NC}"
echo
echo -e "${BLUE}Service URLs:${NC}"
echo -e "  Frontend: $FRONTEND_URL"
echo -e "  Admin:    $ADMIN_URL"
echo -e "  Backend:  $BACKEND_URL"
echo
echo -e "${YELLOW}🔍 Test the services:${NC}"
echo -e "  curl $FRONTEND_URL"
echo -e "  curl $BACKEND_URL/health"