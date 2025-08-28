#!/bin/bash

# Build all services from latest source code
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}🚀 Building All EPSX Services from Latest Source${NC}"

# Get script directory (project root/scripts)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${YELLOW}Project Root: $PROJECT_ROOT${NC}"
echo

# Track build results
SUCCESSFUL_BUILDS=()
FAILED_BUILDS=()

# Function to build a service
build_service() {
    local service_name=$1
    local service_path=$2
    
    echo -e "${PURPLE}=== Building $service_name ===${NC}"
    
    if [ -f "$service_path/build.sh" ]; then
        if (cd "$service_path" && ./build.sh); then
            echo -e "${GREEN}✅ $service_name build successful${NC}"
            SUCCESSFUL_BUILDS+=("$service_name")
            return 0
        else
            echo -e "${RED}❌ $service_name build failed${NC}"
            FAILED_BUILDS+=("$service_name")
            return 1
        fi
    else
        echo -e "${RED}❌ Build script not found: $service_path/build.sh${NC}"
        FAILED_BUILDS+=("$service_name")
        return 1
    fi
}

# Build all services
build_service "Backend" "$PROJECT_ROOT/apps/backend"
echo

build_service "Frontend" "$PROJECT_ROOT/apps/frontend" 
echo

build_service "Admin Frontend" "$PROJECT_ROOT/apps/admin-frontend"

# Build summary
echo -e "\n${BLUE}=========================================${NC}"
echo -e "${BLUE}           BUILD SUMMARY                 ${NC}"
echo -e "${BLUE}=========================================${NC}"

if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo -e "\n${GREEN}✅ Successfully Built Services:${NC}"
    for service in "${SUCCESSFUL_BUILDS[@]}"; do
        echo -e "  ${GREEN}✓${NC} $service"
    done
fi

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo -e "\n${RED}❌ Failed Builds:${NC}"
    for service in "${FAILED_BUILDS[@]}"; do
        echo -e "  ${RED}✗${NC} $service"
    done
fi

# Show next steps
if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo -e "\n${GREEN}🎉 Build completed with ${#SUCCESSFUL_BUILDS[@]} successful service(s)!${NC}"
    echo -e "${YELLOW}🚀 Ready to deploy with: ./scripts/deploy.sh${NC}"
else
    echo -e "\n${RED}❌ All builds failed${NC}"
    echo -e "${YELLOW}💡 Check individual build logs above for details${NC}"
fi

# Exit with appropriate code
if [ ${#FAILED_BUILDS[@]} -eq 0 ]; then
    exit 0
else
    exit 1
fi