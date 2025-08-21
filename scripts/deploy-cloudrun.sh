#!/bin/bash

# EPSX - Deploy to Google Cloud Run
# Deploys frontend, admin-frontend, and backend services to Cloud Run with optimal configuration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"
VERSION="${BUILD_VERSION:-latest}"

echo -e "${BLUE}🚀 EPSX Cloud Run Deployment${NC}"
echo -e "${PURPLE}Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Repository: $REPOSITORY"
echo -e "  Version: $VERSION"
echo

# Image names
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:$VERSION"
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:$VERSION"
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:$VERSION"

# Service names
FRONTEND_SERVICE="epsx-frontend"
ADMIN_SERVICE="epsx-admin"  
BACKEND_SERVICE="epsx-backend"

# Check if images exist in registry
echo -e "${BLUE}🔍 Verifying images in registry...${NC}"
check_image() {
    local image=$1
    local name=$2
    
    if gcloud artifacts docker images describe "$image" --quiet >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $name image found${NC}"
    else
        echo -e "${RED}❌ $name image not found: $image${NC}"
        echo -e "${YELLOW}💡 Run ./scripts/push-all.sh first${NC}"
        exit 1
    fi
}

check_image "$FRONTEND_IMAGE" "Frontend"
check_image "$ADMIN_IMAGE" "Admin"
check_image "$BACKEND_IMAGE" "Backend"

# Function to deploy service
deploy_service() {
    local service_name=$1
    local image=$2
    local port=$3
    local memory=$4
    local cpu=$5
    local min_instances=$6
    local max_instances=$7
    local display_name=$8
    local env_vars=("${@:9}")
    
    echo -e "${BLUE}🚀 Deploying $display_name...${NC}"
    
    # Prepare environment variables
    local env_vars_str=""
    for env_var in "${env_vars[@]}"; do
        if [[ -n "$env_var" ]]; then
            if [[ -z "$env_vars_str" ]]; then
                env_vars_str="$env_var"
            else
                env_vars_str="$env_vars_str,$env_var"
            fi
        fi
    done
    
    # Deploy to Cloud Run
    local deploy_cmd="gcloud run deploy $service_name \
        --image=$image \
        --platform=managed \
        --region=$REGION \
        --allow-unauthenticated \
        --port=$port \
        --memory=$memory \
        --cpu=$cpu \
        --min-instances=$min_instances \
        --max-instances=$max_instances \
        --timeout=300s \
        --concurrency=80 \
        --execution-environment=gen2"
    
    # Add environment variables if provided
    if [[ -n "$env_vars_str" ]]; then
        deploy_cmd+=" --set-env-vars \"$env_vars_str\""
    fi
    
    # Execute deployment
    if eval "$deploy_cmd --quiet"; then
        echo -e "${GREEN}✅ $display_name deployed successfully${NC}"
        
        # Get service URL
        local service_url=$(gcloud run services describe "$service_name" --region="$REGION" --format="value(status.url)")
        echo -e "${BLUE}🔗 $display_name URL: $service_url${NC}"
        echo
        return 0
    else
        echo -e "${RED}❌ $display_name deployment failed${NC}"
        return 1
    fi
}

# Deploy Backend first (other services depend on it)
echo -e "${YELLOW}📦 Step 1: Deploying Backend API...${NC}"
BACKEND_ENV_VARS=(
    "NODE_ENV=production"
    "RUST_ENV=production"
    "ENV=production"
    "DATABASE_URL=postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require"
    "NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum"
    "FIREBASE_PROJECT_ID=epsx-449804"
    "DATABASE_STATEMENT_LOGGING=false"
    "DATABASE_MAX_CONNECTIONS=20"
    "DATABASE_MIN_CONNECTIONS=2"
    "DATABASE_ACQUIRE_TIMEOUT=15"
    "RUST_LOG=info"
)

deploy_service "$BACKEND_SERVICE" "$BACKEND_IMAGE" "8080" "1Gi" "1" "1" "10" "Backend API" "${BACKEND_ENV_VARS[@]}"

# Get backend URL for frontend services
BACKEND_URL=$(gcloud run services describe "$BACKEND_SERVICE" --region="$REGION" --format="value(status.url)")

# Deploy Frontend
echo -e "${YELLOW}📦 Step 2: Deploying Frontend...${NC}"
FRONTEND_ENV_VARS=(
    "NODE_ENV=production"
    "NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL"
    "NEXT_TELEMETRY_DISABLED=1"
)

deploy_service "$FRONTEND_SERVICE" "$FRONTEND_IMAGE" "3000" "512Mi" "1" "1" "10" "Frontend" "${FRONTEND_ENV_VARS[@]}"

# Deploy Admin Frontend
echo -e "${YELLOW}📦 Step 3: Deploying Admin Frontend...${NC}"
ADMIN_ENV_VARS=(
    "NODE_ENV=production"
    "NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL"
    "NEXT_TELEMETRY_DISABLED=1"
)

deploy_service "$ADMIN_SERVICE" "$ADMIN_IMAGE" "3000" "512Mi" "1" "0" "5" "Admin Frontend" "${ADMIN_ENV_VARS[@]}"

# Get all service URLs
echo -e "${GREEN}🎉 All services deployed successfully!${NC}"
echo
echo -e "${BLUE}📋 Service URLs:${NC}"

FRONTEND_URL=$(gcloud run services describe "$FRONTEND_SERVICE" --region="$REGION" --format="value(status.url)")
ADMIN_URL=$(gcloud run services describe "$ADMIN_SERVICE" --region="$REGION" --format="value(status.url)")

echo -e "${GREEN}  Frontend:${NC}     $FRONTEND_URL"
echo -e "${GREEN}  Admin:${NC}        $ADMIN_URL"
echo -e "${GREEN}  Backend API:${NC}  $BACKEND_URL"

echo
echo -e "${YELLOW}🔧 Service Configuration:${NC}"
echo -e "${GREEN}  Frontend:${NC} 512Mi memory, 1 CPU, 1-10 instances"
echo -e "${GREEN}  Admin:${NC}    512Mi memory, 1 CPU, 0-5 instances (scales to zero)"
echo -e "${GREEN}  Backend:${NC}  1Gi memory, 1 CPU, 1-10 instances"

echo
echo -e "${BLUE}✅ Deployment Complete!${NC}"
echo -e "${YELLOW}📊 Monitor services:${NC} gcloud run services list --region=$REGION"
echo -e "${YELLOW}📋 View logs:${NC} gcloud run services logs read SERVICE_NAME --region=$REGION"
echo -e "${YELLOW}🔧 Update config:${NC} gcloud run services update SERVICE_NAME --region=$REGION"

echo
echo -e "${PURPLE}🌐 Your EPSX platform is now live on Google Cloud Run!${NC}"