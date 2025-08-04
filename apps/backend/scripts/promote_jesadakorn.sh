#!/bin/bash

# Script to grant full system access to jesadakorn.kirtnu@gmail.com
# This script is specifically tailored for this user

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Promoting jesadakorn.kirtnu@gmail.com to SuperAdmin with full access${NC}"
echo -e "${YELLOW}⚠️  This will grant ALL system permissions!${NC}"

# Confirm action
read -p "Are you sure you want to proceed? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Operation cancelled${NC}"
    exit 1
fi

# Run the general script with specific parameters
"$(dirname "$0")/grant_full_access.sh" "jesadakorn.kirtnu@gmail.com" "System administrator - full access granted via admin script"