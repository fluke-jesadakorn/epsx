#!/bin/bash
set -e

# =============================================================================
# Fund Tokens Script
# =============================================================================
# Mint USDT and USDC to a specified address on local Anvil.
#
# Usage:
#   ./scripts/fund-tokens.sh <address>
#   ./scripts/fund-tokens.sh <address> <amount>
#
# Examples:
#   ./scripts/fund-tokens.sh 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
#   ./scripts/fund-tokens.sh 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 50000
# =============================================================================

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
RPC_URL="http://127.0.0.1:8545"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# BSC Mainnet Token Addresses
USDT_ADDRESS="0x55d398326f99059fF775485246999027B3197955"
USDC_ADDRESS="0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d"

# Check arguments
if [ -z "$1" ]; then
    echo -e "${RED}Error: No address provided${NC}"
    echo ""
    echo "Usage: $0 <address> [amount]"
    echo ""
    echo "Examples:"
    echo "  $0 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    echo "  $0 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 50000"
    exit 1
fi

TARGET_ADDRESS=$1
AMOUNT=${2:-100000}

# Convert amount to wei (18 decimals)
AMOUNT_WEI=$(echo "$AMOUNT * 1000000000000000000" | bc)

echo -e "${YELLOW}💰 Funding $TARGET_ADDRESS${NC}"
echo ""

# Mint USDT
echo -n "   Minting $AMOUNT USDT... "
cast send $USDT_ADDRESS "mint(address,uint256)" $TARGET_ADDRESS $AMOUNT_WEI \
    --private-key $PRIVATE_KEY \
    --rpc-url $RPC_URL \
    > /dev/null 2>&1
echo -e "${GREEN}✅${NC}"

# Mint USDC
echo -n "   Minting $AMOUNT USDC... "
cast send $USDC_ADDRESS "mint(address,uint256)" $TARGET_ADDRESS $AMOUNT_WEI \
    --private-key $PRIVATE_KEY \
    --rpc-url $RPC_URL \
    > /dev/null 2>&1
echo -e "${GREEN}✅${NC}"

echo ""
echo -e "${GREEN}Done!${NC} Minted $AMOUNT USDT + $AMOUNT USDC to $TARGET_ADDRESS"

# Show current balance
echo ""
echo "Current Balances:"
USDT_BALANCE=$(cast call $USDT_ADDRESS "balanceOf(address)(uint256)" $TARGET_ADDRESS --rpc-url $RPC_URL 2>/dev/null || echo "0")
USDC_BALANCE=$(cast call $USDC_ADDRESS "balanceOf(address)(uint256)" $TARGET_ADDRESS --rpc-url $RPC_URL 2>/dev/null || echo "0")

# Convert from wei to human readable (divide by 1e18)
if command -v python3 &> /dev/null; then
    USDT_HUMAN=$(python3 -c "print(f'{int(\"$USDT_BALANCE\") / 1e18:,.2f}')" 2>/dev/null || echo "$USDT_BALANCE")
    USDC_HUMAN=$(python3 -c "print(f'{int(\"$USDC_BALANCE\") / 1e18:,.2f}')" 2>/dev/null || echo "$USDC_BALANCE")
    echo "   USDT: $USDT_HUMAN"
    echo "   USDC: $USDC_HUMAN"
else
    echo "   USDT: $USDT_BALANCE (wei)"
    echo "   USDC: $USDC_BALANCE (wei)"
fi
