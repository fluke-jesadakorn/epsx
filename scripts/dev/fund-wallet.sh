#!/bin/bash
set -e

# =============================================================================
# Fund Wallet Script
# =============================================================================
# Mints BNB (native gas), USDT, and USDC to a wallet on local Anvil.
#
# Usage:
#   ./scripts/dev/fund-wallet.sh <address>
#   ./scripts/dev/fund-wallet.sh <address> <bnb_amount> <token_amount>
#
# Examples:
#   ./scripts/dev/fund-wallet.sh 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
#   ./scripts/dev/fund-wallet.sh 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 10 1000
# =============================================================================

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'
BOLD='\033[1m'

# Configuration
RPC_URL="${RPC_URL:-http://127.0.0.1:8545}"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# BSC Mainnet Token Addresses (etched to local Anvil via setup-local.sh)
USDT_ADDRESS="0x55d398326f99059fF775485246999027B3197955"
USDC_ADDRESS="0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d"

# Check arguments
if [ -z "$1" ]; then
    echo -e "${RED}Error: No address provided${NC}"
    echo ""
    echo "Usage: $0 <address> [bnb_amount] [token_amount]"
    echo ""
    echo "Examples:"
    echo "  $0 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    echo "  $0 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 10 1000"
    exit 1
fi

TARGET_ADDRESS=$1
BNB_AMOUNT=${2:-100}
TOKEN_AMOUNT=${3:-100000}

# Calculate BNB in hex using cast + python for large number handling
BNB_WEI=$(cast to-wei $BNB_AMOUNT ether)
BNB_HEX=$(python3 -c "print(hex(int('$BNB_WEI')))")

# Calculate token amount in wei (18 decimals) using cast
TOKEN_WEI=$(cast to-wei $TOKEN_AMOUNT ether)

echo ""
echo -e "${BLUE}${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}${BOLD}║           💰 Fund Wallet - Local Anvil                    ║${NC}"
echo -e "${BLUE}${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "   Target: ${YELLOW}$TARGET_ADDRESS${NC}"
echo -e "   BNB:    $BNB_AMOUNT BNB (for gas)"
echo -e "   Tokens: $TOKEN_AMOUNT USDT + $TOKEN_AMOUNT USDC"
echo ""

# Check if contracts exist
echo -e "${YELLOW}🔍 Checking for contracts...${NC}"
CODE_SIZE=$(cast code $USDT_ADDRESS --rpc-url $RPC_URL | wc -c)
# wc -c of "0x" is 3 (including newline) or 2 depending on system. Empty code is usually "0x"
if [[ $CODE_SIZE -le 3 ]]; then
    echo -e "${RED}❌ Contracts not found on local Anvil!${NC}"
    echo -e "${YELLOW}⚠️  You must run 'bun setup:local' to deploy contracts first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Contracts found${NC}"

# Step 1: Set BNB Balance (native token for gas)
echo -e "${YELLOW}⛽ Step 1: Setting BNB Balance...${NC}"
RESULT=$(curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\":\"2.0\",
    \"method\":\"anvil_setBalance\",
    \"params\":[\"$TARGET_ADDRESS\", \"$BNB_HEX\"],
    \"id\":1
  }")
if [[ "$RESULT" == *"result"* ]]; then
    echo -e "   ${GREEN}✅ Set $BNB_AMOUNT BNB${NC}"
else
    echo -e "   ${RED}❌ Failed to set BNB: $RESULT${NC}"
fi

# Step 2: Mint USDT
echo -e "${YELLOW}🪙 Step 2: Minting USDT...${NC}"
if cast send $USDT_ADDRESS "mint(address,uint256)" $TARGET_ADDRESS $TOKEN_WEI \
    --private-key $PRIVATE_KEY \
    --rpc-url $RPC_URL \
    > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ Minted $(printf "%'d" $TOKEN_AMOUNT) USDT${NC}"
else
    echo -e "   ${RED}❌ Failed to mint USDT${NC}"
    echo -e "   ${YELLOW}⚠️  Have you run 'bun setup:local' first?${NC}"
fi

# Step 3: Mint USDC
echo -e "${YELLOW}🪙 Step 3: Minting USDC...${NC}"
if cast send $USDC_ADDRESS "mint(address,uint256)" $TARGET_ADDRESS $TOKEN_WEI \
    --private-key $PRIVATE_KEY \
    --rpc-url $RPC_URL \
    > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ Minted $(printf "%'d" $TOKEN_AMOUNT) USDC${NC}"
else
    echo -e "   ${RED}❌ Failed to mint USDC${NC}"
fi

# Step 4: Show final balances
set +e
echo ""
echo -e "${YELLOW}📊 Final Balances:${NC}"

# Get BNB balance
BNB_BALANCE=$(cast balance $TARGET_ADDRESS --rpc-url $RPC_URL 2>/dev/null || echo "0")
if [ ! -z "$BNB_BALANCE" ] && [ "$BNB_BALANCE" != "0" ]; then
    BNB_HUMAN=$(cast from-wei $BNB_BALANCE ether 2>/dev/null || echo "$BNB_BALANCE")
    echo -e "   BNB:  ${GREEN}$BNB_HUMAN${NC}"
else
    echo -e "   BNB:  ${RED}Error fetching${NC}"
fi

# Get USDT balance
USDT_BALANCE_RAW=$(cast call $USDT_ADDRESS "balanceOf(address)(uint256)" $TARGET_ADDRESS --rpc-url $RPC_URL 2>/dev/null)
if [ ! -z "$USDT_BALANCE_RAW" ]; then
    # Extract just the number (remove any extra output like [3e23])
    USDT_BALANCE=$(echo "$USDT_BALANCE_RAW" | awk '{print $1}')
    USDT_HUMAN=$(cast from-wei $USDT_BALANCE ether 2>/dev/null || echo "$USDT_BALANCE")
    echo -e "   USDT: ${GREEN}$USDT_HUMAN${NC}"
else
    echo -e "   USDT: ${RED}Error fetching${NC}"
fi

# Get USDC balance
USDC_BALANCE_RAW=$(cast call $USDC_ADDRESS "balanceOf(address)(uint256)" $TARGET_ADDRESS --rpc-url $RPC_URL 2>/dev/null)
if [ ! -z "$USDC_BALANCE_RAW" ]; then
    # Extract just the number (remove any extra output like [3e23])
    USDC_BALANCE=$(echo "$USDC_BALANCE_RAW" | awk '{print $1}')
    USDC_HUMAN=$(cast from-wei $USDC_BALANCE ether 2>/dev/null || echo "$USDC_BALANCE")
    echo -e "   USDC: ${GREEN}$USDC_HUMAN${NC}"
else
    echo -e "   USDC: ${RED}Error fetching${NC}"
fi

echo ""
echo -e "${BLUE}${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}${BOLD}║                       Done! 🎉                            ║${NC}"
echo -e "${BLUE}${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "   Refresh MetaMask to see updated balances."
echo ""
