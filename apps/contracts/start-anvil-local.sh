#!/bin/bash
# Start Anvil in pure local mode (no BSC fork)
# This provides a clean, fast, offline-capable development environment

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if port 8545 is in use and kill it
if lsof -i :8545 -t >/dev/null 2>&1; then
    echo -e "${BLUE}⚠️  Port 8545 is in use. Killing existing process...${NC}"
    lsof -i :8545 -t | xargs kill -9
    sleep 1
    echo -e "${GREEN}✅ Port 8545 freed.${NC}"
fi

echo -e "${BLUE}🚀 Starting Anvil Local Development Chain...${NC}"
echo ""
echo "   Network: Local Development"
echo "   Chain ID: 31337"
echo "   RPC URL: http://localhost:8545"
echo ""
echo -e "${GREEN}📝 Default Test Accounts:${NC}"
echo "   Account #0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
echo "   Account #1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
echo ""
echo -e "${BLUE}💡 Run 'bun run setup:local' in another terminal to deploy contracts & tokens${NC}"
echo ""

# Start Anvil with deterministic addresses
anvil \
    --chain-id 31337 \
    --host 0.0.0.0 \
    --gas-price 3000000000 \
    --block-time 1 \
    --accounts 10 \
    --balance 10000
