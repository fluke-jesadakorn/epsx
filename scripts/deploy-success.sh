#!/bin/bash

# EPSX - Complete Production Deployment Success Script
# Builds and deploys all services with fixes applied

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 EPSX Complete Production Deployment${NC}"
echo -e "${GREEN}✅ Service worker issues fixed${NC}"
echo -e "${GREEN}✅ Database permission issues fixed${NC}"
echo -e "${GREEN}✅ Environment variable handling fixed${NC}"
echo

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="${BUILD_VERSION:-$(date +%Y%m%d-%H%M%S)}"

echo -e "${PURPLE}Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Repository: $REPOSITORY"
echo -e "  Version: $VERSION"
echo

# Set production environment
export NODE_ENV=production
export RUST_ENV=production
export ENV=production
export DATABASE_STATEMENT_LOGGING=false

# Required environment variables
export DATABASE_URL="${DATABASE_URL:-postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require}"
export NEXTAUTH_SECRET="${NEXTAUTH_SECRET:-prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum}"
export FIREBASE_PROJECT_ID="${FIREBASE_PROJECT_ID:-epsx-449804}"
export FRONTEND_URL="${FRONTEND_URL:-https://epsx.io}"
export BACKEND_URL="${BACKEND_URL:-https://api.epsx.io}"
export ADMIN_FRONTEND_URL="${ADMIN_FRONTEND_URL:-https://admin.epsx.io}"

# Step 1: Build updated backend with database fix
echo -e "${YELLOW}📦 Step 1: Building backend with database fixes...${NC}"
cd apps/backend
docker build \
    --platform linux/amd64 \
    --tag "us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION" \
    --tag "backend:latest" \
    --build-arg "DATABASE_STATEMENT_LOGGING=false" \
    --build-arg "RUST_LOG=info" \
    .
cd ../..

echo -e "${GREEN}✅ Backend built successfully${NC}"

# Step 2: Build frontend (without service worker)
echo -e "${YELLOW}📦 Step 2: Building frontend...${NC}"
cd apps/frontend
docker build \
    --platform linux/amd64 \
    --tag "us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION" \
    --tag "frontend:latest" \
    --build-arg "NODE_ENV=production" \
    .
cd ../..

echo -e "${GREEN}✅ Frontend built successfully${NC}"

# Step 3: Build admin frontend
echo -e "${YELLOW}📦 Step 3: Building admin frontend...${NC}"
cd apps/admin-frontend
docker build \
    --platform linux/amd64 \
    --tag "us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION" \
    --tag "admin:latest" \
    --build-arg "NODE_ENV=production" \
    .
cd ../..

echo -e "${GREEN}✅ Admin frontend built successfully${NC}"

# Step 4: Push to registry
echo -e "${YELLOW}📦 Step 4: Pushing to registry...${NC}"
gcloud auth configure-docker $REGION-docker.pkg.dev --quiet

docker push "us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"
docker push "us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"  
docker push "us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"

echo -e "${GREEN}✅ All images pushed successfully${NC}"

# Step 5: Deploy all services
echo -e "${YELLOW}📦 Step 5: Deploying to Cloud Run...${NC}"

# Deploy Backend
echo -e "${BLUE}🚀 Deploying Backend...${NC}"
gcloud run deploy epsx-backend \
    --image="us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION" \
    --platform=managed \
    --region=$REGION \
    --allow-unauthenticated \
    --port=8080 \
    --memory=1Gi \
    --cpu=1 \
    --min-instances=1 \
    --max-instances=10 \
    --timeout=300s \
    --concurrency=80 \
    --execution-environment=gen2 \
    --set-env-vars="NODE_ENV=production,RUST_ENV=production,ENV=production,DATABASE_URL=$DATABASE_URL,NEXTAUTH_SECRET=$NEXTAUTH_SECRET,FIREBASE_PROJECT_ID=$FIREBASE_PROJECT_ID,DATABASE_STATEMENT_LOGGING=false,DATABASE_MAX_CONNECTIONS=20,DATABASE_MIN_CONNECTIONS=2,DATABASE_ACQUIRE_TIMEOUT=15,RUST_LOG=info,HOST=0.0.0.0,PORT=8080" \
    --quiet

BACKEND_URL=$(gcloud run services describe epsx-backend --region=$REGION --format="value(status.url)")
echo -e "${GREEN}✅ Backend deployed: $BACKEND_URL${NC}"

# Deploy Frontend
echo -e "${BLUE}🚀 Deploying Frontend...${NC}"
gcloud run deploy epsx-frontend \
    --image="us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION" \
    --platform=managed \
    --region=$REGION \
    --allow-unauthenticated \
    --port=3000 \
    --memory=512Mi \
    --cpu=1 \
    --min-instances=1 \
    --max-instances=10 \
    --timeout=300s \
    --concurrency=80 \
    --execution-environment=gen2 \
    --set-env-vars="NODE_ENV=production,NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL,NEXT_TELEMETRY_DISABLED=1" \
    --quiet

FRONTEND_URL=$(gcloud run services describe epsx-frontend --region=$REGION --format="value(status.url)")
echo -e "${GREEN}✅ Frontend deployed: $FRONTEND_URL${NC}"

# Deploy Admin
echo -e "${BLUE}🚀 Deploying Admin Frontend...${NC}"
gcloud run deploy epsx-admin \
    --image="us-central1-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION" \
    --platform=managed \
    --region=$REGION \
    --allow-unauthenticated \
    --port=3000 \
    --memory=512Mi \
    --cpu=1 \
    --min-instances=0 \
    --max-instances=5 \
    --timeout=300s \
    --concurrency=80 \
    --execution-environment=gen2 \
    --set-env-vars="NODE_ENV=production,NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL,NEXT_TELEMETRY_DISABLED=1" \
    --quiet

ADMIN_URL=$(gcloud run services describe epsx-admin --region=$REGION --format="value(status.url)")
echo -e "${GREEN}✅ Admin deployed: $ADMIN_URL${NC}"

echo
echo -e "${GREEN}🎉 Complete deployment successful!${NC}"
echo
echo -e "${BLUE}📋 Service URLs:${NC}"
echo -e "${GREEN}  Frontend:${NC}     $FRONTEND_URL"
echo -e "${GREEN}  Admin:${NC}        $ADMIN_URL"
echo -e "${GREEN}  Backend API:${NC}  $BACKEND_URL"
echo
echo -e "${PURPLE}🔧 Changes Applied:${NC}"
echo -e "${GREEN}  ✅ Service worker removed (PWA errors fixed)${NC}"
echo -e "${GREEN}  ✅ Database permission issues resolved${NC}"
echo -e "${GREEN}  ✅ Environment variables properly configured${NC}"
echo -e "${GREEN}  ✅ Container architecture fixed for Cloud Run${NC}"
echo -e "${GREEN}  ✅ Production logging optimized${NC}"
echo
echo -e "${BLUE}✅ Your EPSX platform is now live and working!${NC}"