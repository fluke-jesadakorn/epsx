#!/bin/bash

# EPSX - Build Latest Images and Deploy to Google Cloud Run
# Complete build and deployment pipeline

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}🚀 EPSX Complete Build & Deploy Pipeline${NC}"
echo -e "${PURPLE}Building fresh images from latest source code...${NC}"
echo

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Step 1: Build all services
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}           STEP 1: BUILD                ${NC}"
echo -e "${BLUE}=========================================${NC}"

if "$SCRIPT_DIR/build-all.sh"; then
    echo -e "\n${GREEN}✅ All services built successfully!${NC}"
else
    echo -e "\n${RED}❌ Build failed - stopping deployment${NC}"
    echo -e "${YELLOW}💡 Check build logs above for details${NC}"
    exit 1
fi

# Brief pause between build and deploy
echo -e "\n${YELLOW}⏳ Preparing for deployment...${NC}"
sleep 3

# Step 2: Deploy all services
echo -e "\n${BLUE}=========================================${NC}"
echo -e "${BLUE}           STEP 2: DEPLOY               ${NC}"
echo -e "${BLUE}=========================================${NC}"

if "$SCRIPT_DIR/deploy.sh"; then
    echo -e "\n${GREEN}🎉 Complete Build & Deploy Successful!${NC}"
    echo -e "\n${BLUE}🌐 Your EPSX Platform (with latest code):${NC}"
    echo -e "  ${GREEN}Main Platform:${NC}  https://epsx.io"
    echo -e "  ${GREEN}API Backend:${NC}    https://api.epsx.io"
    echo -e "  ${GREEN}Admin Panel:${NC}    https://admin.epsx.io"
    echo -e "\n${YELLOW}🔥 Fresh deployment complete with your latest source code!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Deployment failed${NC}"
    echo -e "${YELLOW}💡 Images were built successfully but deployment encountered issues${NC}"
    echo -e "${YELLOW}💡 You can retry deployment with: ./scripts/deploy.sh${NC}"
    exit 1
fi