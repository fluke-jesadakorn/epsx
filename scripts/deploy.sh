#!/bin/bash

# EPSX - Deploy All Services with Latest Images to Google Cloud Run
# Deploys the freshly built images from build-all.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying EPSX Platform with Latest Images${NC}"

# Configuration
PROJECT_ID="${GOOGLE_CLOUD_PROJECT:-epsx-469400}"
REGION="${GOOGLE_CLOUD_REGION:-us-central1}"
REPOSITORY="${ARTIFACT_REGISTRY_REPO:-epsx}"

echo -e "${YELLOW}Deploy Configuration:${NC}"
echo -e "  Project: $PROJECT_ID"
echo -e "  Region: $REGION"
echo -e "  Repository: $REPOSITORY"
echo

# Function to deploy a service
deploy_service() {
    local service_name=$1
    local service_slug=$2
    local image_name=$3
    local memory=$4
    local cpu=$5
    local max_instances=$6
    local port=$7
    local custom_domain=$8
    
    echo -e "${PURPLE}=== Deploying $service_name ===${NC}"
    
    local image="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/$image_name:latest"
    
    echo -e "${YELLOW}Service: $service_slug${NC}"
    echo -e "${YELLOW}Image: $image${NC}"
    echo -e "${YELLOW}Resources: $memory, $cpu CPU${NC}"
    echo -e "${YELLOW}Domain: $custom_domain${NC}"
    
    # Deploy to Cloud Run
    if gcloud run deploy "$service_slug" \
        --image="$image" \
        --platform=managed \
        --region="$REGION" \
        --allow-unauthenticated \
        --port="$port" \
        --memory="$memory" \
        --cpu="$cpu" \
        --min-instances=0 \
        --max-instances="$max_instances" \
        --timeout=300s \
        --concurrency=80 \
        --execution-environment=gen2 \
        --set-env-vars="NODE_ENV=production,RUST_LOG=info,NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum" \
        --quiet; then
        
        echo -e "${GREEN}✅ $service_name deployed successfully${NC}"
        
        # Get service URL
        local service_url=$(gcloud run services describe "$service_slug" --region="$REGION" --format="value(status.url)" 2>/dev/null || echo "")
        
        echo -e "${GREEN}   Service URL: $service_url${NC}"
        echo -e "${GREEN}   Custom Domain: $custom_domain${NC}"
        
        return 0
    else
        echo -e "${RED}❌ $service_name deployment failed${NC}"
        return 1
    fi
}

# Deploy services
SUCCESSFUL_SERVICES=()
FAILED_SERVICES=()

# Backend
echo -e "${BLUE}📦 Backend API${NC}"
if deploy_service "Backend API" "epsx-backend" "backend" "4Gi" "4" "10" "8080" "https://api.epsx.io"; then
    SUCCESSFUL_SERVICES+=("Backend API")
else
    FAILED_SERVICES+=("Backend API")
fi

echo

# Frontend  
echo -e "${BLUE}📦 Frontend${NC}"
if deploy_service "Frontend" "epsx-frontend" "frontend" "2Gi" "2" "10" "3000" "https://epsx.io"; then
    SUCCESSFUL_SERVICES+=("Frontend")
else
    FAILED_SERVICES+=("Frontend")
fi

echo

# Admin Frontend
echo -e "${BLUE}📦 Admin Dashboard${NC}"
if deploy_service "Admin Dashboard" "epsx-admin" "admin" "1Gi" "1" "5" "3000" "https://admin.epsx.io"; then
    SUCCESSFUL_SERVICES+=("Admin Dashboard")
else
    FAILED_SERVICES+=("Admin Dashboard")
fi

# Final status report
echo -e "\n${BLUE}=========================================${NC}"
echo -e "${BLUE}           DEPLOYMENT SUMMARY           ${NC}"
echo -e "${BLUE}=========================================${NC}"

if [ ${#SUCCESSFUL_SERVICES[@]} -gt 0 ]; then
    echo -e "\n${GREEN}✅ Successfully Deployed:${NC}"
    for service in "${SUCCESSFUL_SERVICES[@]}"; do
        echo -e "  ${GREEN}✓${NC} $service"
    done
fi

if [ ${#FAILED_SERVICES[@]} -gt 0 ]; then
    echo -e "\n${RED}❌ Failed Deployments:${NC}"
    for service in "${FAILED_SERVICES[@]}"; do
        echo -e "  ${RED}✗${NC} $service"
    done
fi

# Show service URLs
echo -e "\n${BLUE}🌐 Your EPSX Platform:${NC}"
echo -e "  ${GREEN}Main Platform:${NC}  https://epsx.io"
echo -e "  ${GREEN}API Backend:${NC}    https://api.epsx.io"
echo -e "  ${GREEN}Admin Panel:${NC}    https://admin.epsx.io"

# Exit with appropriate code
if [ ${#FAILED_SERVICES[@]} -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All Services Deployed Successfully!${NC}"
    exit 0
else
    echo -e "\n${RED}⚠️  Deployment completed with ${#FAILED_SERVICES[@]} error(s)${NC}"
    exit 1
fi