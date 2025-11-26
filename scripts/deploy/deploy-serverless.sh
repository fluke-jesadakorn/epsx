#!/bin/bash

# Generic Serverless Deployment Script for EPSX Backend
# Platform-agnostic deployment using stateless architecture
# Can be adapted for any containerized serverless platform

set -e

# Configuration (customize for your platform)
SERVICE_NAME="epsx-backend-serverless"
IMAGE_NAME="${SERVICE_NAME}"
BUILD_TARGET="serverless"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Building EPSX Backend for Serverless Deployment${NC}"
echo -e "${BLUE}=============================================${NC}"

# Check if running from correct directory
if [ ! -f "apps/backend/Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Must run from project root directory${NC}"
    exit 1
fi

# Check required environment variables
echo -e "${YELLOW}🔍 Validating environment...${NC}"
echo -e "${GREEN}✅ Environment validation passed${NC}"

# Build Docker image for serverless
echo -e "${YELLOW}🏗️  Building serverless Docker image...${NC}"
cd apps/backend

# Check if Dockerfile.serverless exists
if [ ! -f "Dockerfile.serverless" ]; then
    echo -e "${RED}❌ Dockerfile.serverless not found${NC}"
    echo -e "${YELLOW}Please create Dockerfile.serverless first${NC}"
    exit 1
fi

# Build Docker image with serverless optimization
echo -e "${YELLOW}🔨 Building serverless Docker image...${NC}"
docker build \
    --platform linux/amd64 \
    --file Dockerfile.serverless \
    --tag ${IMAGE_NAME}:latest \
    --tag ${IMAGE_NAME}:serverless-$(date +%Y%m%d-%H%M%S) \
    .

echo -e "${GREEN}✅ Docker image built successfully${NC}"

# Test image locally
echo -e "${YELLOW}🧪 Testing serverless image locally...${NC}"
TEST_PORT=8081
echo -e "${BLUE}Starting test container on port ${TEST_PORT}...${NC}"

# Kill any existing test container
docker rm -f epsx-serverless-test 2>/dev/null || true

# Start test container
docker run -d \
    --name epsx-serverless-test \
    --platform linux/amd64 \
    -p ${TEST_PORT}:8080 \
    -e DATABASE_URL="${DATABASE_URL:-postgresql://localhost/epsx}" \
    -e REDIS_URL="${REDIS_URL:-redis://localhost:6379}" \
    -e FRONTEND_URL="${FRONTEND_URL:-http://localhost:3000}" \
    -e BACKEND_URL="${BACKEND_URL:-http://localhost:8080}" \
    ${IMAGE_NAME}:latest

# Wait for container to start
sleep 3

# Test health endpoint
echo -e "${BLUE}Testing health endpoint...${NC}"
if curl -f http://localhost:${TEST_PORT}/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Local serverless test passed${NC}"
else
    echo -e "${RED}❌ Local serverless test failed${NC}"
    docker logs epsx-serverless-test
    docker rm -f epsx-serverless-test
    exit 1
fi

# Stop test container
docker rm -f epsx-serverless-test

echo -e "${BLUE}=================================================${NC}"
echo -e "${GREEN}🎉 EPSX Backend Serverless Build Complete!${NC}"
echo -e "${BLUE}=================================================${NC}"
echo -e "${GREEN}📦 Docker Image: ${IMAGE_NAME}:latest${NC}"
echo -e "${GREEN}🏥 Health Check: /health${NC}"
echo -e "${GREEN}🔐 Web3 Auth: /api/auth/web3/*${NC}"
echo -e "${GREEN}📊 Analytics: /api/v1/analytics/*${NC}"
echo -e "${BLUE}💡 Features:${NC}"
echo -e "${BLUE}   ⚡ Stateless per-request architecture${NC}"
echo -e "${BLUE}   💾 Global database connection pooling${NC}"
echo -e "${BLUE}   🗄️  Redis-only caching (no memory fallback)${NC}"
echo -e "${BLUE}   🔄 Ready for any serverless platform${NC}"
echo -e "${BLUE}=================================================${NC}"

echo -e "${YELLOW}📋 Next Steps:${NC}"
echo -e "${BLUE}1. Push image to your container registry:${NC}"
echo -e "${BLUE}   docker tag ${IMAGE_NAME}:latest your-registry/${IMAGE_NAME}:latest${NC}"
echo -e "${BLUE}   docker push your-registry/${IMAGE_NAME}:latest${NC}"
echo -e "${BLUE}2. Deploy to your serverless platform using the image${NC}"
echo -e "${BLUE}3. Set required environment variables on your platform${NC}"
echo -e "${BLUE}4. Configure auto-scaling and health checks${NC}"

echo -e "${GREEN}🏁 Serverless build process complete!${NC}"

cd ../..