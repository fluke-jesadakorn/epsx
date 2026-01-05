#!/bin/bash
set -e

# =============================================================================
# EPSX Local Development Setup
# =============================================================================
# This script sets up a complete local Anvil environment:
#   1. Deploys PaymentEscrow contract
#   2. Etches BEP20 mock tokens to mainnet addresses
#   3. Mints tokens to test accounts
# =============================================================================

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
RPC_URL="http://127.0.0.1:8545"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# BSC Mainnet Token Addresses (we etch mocks to these addresses)
USDT_ADDRESS="0x55d398326f99059fF775485246999027B3197955"
USDC_ADDRESS="0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d"

# Test Accounts (Anvil defaults)
ACCOUNT_0="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
ACCOUNT_1="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

# Amount to mint: 100,000 tokens (18 decimals)
MINT_AMOUNT="100000000000000000000000"

# Navigate to contracts directory
if [ -d "apps/contracts" ]; then
    cd apps/contracts
fi

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           EPSX Local Development Setup                    ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# -----------------------------------------------------------------------------
# Step 1: Deploy PaymentEscrow
# -----------------------------------------------------------------------------
echo -e "${YELLOW}📦 Step 1: Deploying PaymentEscrow...${NC}"
forge script script/Deploy.s.sol \
    --rpc-url $RPC_URL \
    --broadcast \
    --private-key $PRIVATE_KEY \
    2>&1 | grep -E "(PaymentEscrow|deployed|enabled)" || true
echo -e "${GREEN}✅ PaymentEscrow deployed${NC}"
echo ""

# -----------------------------------------------------------------------------
# Step 2: Etch Mock Tokens
# -----------------------------------------------------------------------------
echo -e "${YELLOW}🎨 Step 2: Etching Mock Tokens to Mainnet Addresses...${NC}"

# Build contracts first to ensure bytecode is available
forge build --quiet

# Get bytecode for MockUSDT and MockUSDC
USDT_BYTECODE=$(forge inspect contracts/BEP20Mock.sol:MockUSDT deployedBytecode)
USDC_BYTECODE=$(forge inspect contracts/BEP20Mock.sol:MockUSDC deployedBytecode)

# Etch USDT
echo "   Etching USDT to $USDT_ADDRESS..."
cast rpc anvil_setCode "$USDT_ADDRESS" "$USDT_BYTECODE" --rpc-url $RPC_URL > /dev/null

# Etch USDC
echo "   Etching USDC to $USDC_ADDRESS..."
cast rpc anvil_setCode "$USDC_ADDRESS" "$USDC_BYTECODE" --rpc-url $RPC_URL > /dev/null

echo -e "${GREEN}✅ Mock tokens etched${NC}"
echo ""

# -----------------------------------------------------------------------------
# Step 3: Mint Tokens to Test Accounts
# -----------------------------------------------------------------------------
echo -e "${YELLOW}💰 Step 3: Minting Tokens to Test Accounts...${NC}"

mint_tokens() {
    local TOKEN_NAME=$1
    local TOKEN_ADDR=$2
    local TO_ADDR=$3
    
    cast send $TOKEN_ADDR "mint(address,uint256)" $TO_ADDR $MINT_AMOUNT \
        --private-key $PRIVATE_KEY \
        --rpc-url $RPC_URL \
        > /dev/null 2>&1
}

echo "   Minting to Account #0 ($ACCOUNT_0)..."
mint_tokens "USDT" $USDT_ADDRESS $ACCOUNT_0
mint_tokens "USDC" $USDC_ADDRESS $ACCOUNT_0

echo "   Minting to Account #1 ($ACCOUNT_1)..."
mint_tokens "USDT" $USDT_ADDRESS $ACCOUNT_1
mint_tokens "USDC" $USDC_ADDRESS $ACCOUNT_1

echo -e "${GREEN}✅ Tokens minted (100,000 each)${NC}"
echo ""

# -----------------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------------
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Setup Complete! 🎉                     ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "   ${GREEN}USDT:${NC} $USDT_ADDRESS"
echo -e "   ${GREEN}USDC:${NC} $USDC_ADDRESS"
echo ""
echo -e "   ${GREEN}Account #0:${NC} $ACCOUNT_0"
echo -e "   ${GREEN}Account #1:${NC} $ACCOUNT_1"
echo -e "   ${GREEN}Balance:${NC} 100,000 USDT + 100,000 USDC each"
echo ""
echo -e "   To mint more tokens: ${YELLOW}bun fund:tokens <address>${NC}"
echo ""
