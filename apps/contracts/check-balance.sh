#!/bin/bash

ADDRESS=$1
RPC_URL="http://127.0.0.1:8545"

if [ -z "$ADDRESS" ]; then
    echo "Usage: ./check-balance.sh <address>"
    exit 1
fi

echo "🔍 Checking balances for $ADDRESS on Local Anvil Node..."

# USDT
USDT_BAL=$(cast call 0x55d398326f99059fF775485246999027B3197955 "balanceOf(address)(uint256)" "$ADDRESS" --rpc-url "$RPC_URL")
# Convert to human readable (approx)
USDT_FMT=$(echo "$USDT_BAL / 10^18" | bc)

# USDC
USDC_BAL=$(cast call 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d "balanceOf(address)(uint256)" "$ADDRESS" --rpc-url "$RPC_URL")
USDC_FMT=$(echo "$USDC_BAL / 10^18" | bc)

# BNB
BNB_BAL=$(cast balance "$ADDRESS" --rpc-url "$RPC_URL")
BNB_FMT=$(cast from-wei "$BNB_BAL")

echo "----------------------------------------"
echo "💰 USDT: $USDT_BAL ($USDT_FMT)"
echo "💰 USDC: $USDC_BAL ($USDC_FMT)"
echo "💰 BNB:  $BNB_BAL ($BNB_FMT)"
echo "----------------------------------------"

if [ "$USDT_BAL" != "0" ]; then
    echo "✅ Tokens are present on the blockchain!"
    echo "👉 If MetaMask shows 0, it is a display issue."
    echo "   1. Ensure MetaMask network is 'Chain ID: 56' (Not 31337)"
    echo "   2. Reset MetaMask Account (Settings -> Advanced -> Clear activity)"
else
    echo "❌ Balances are 0. Funding failed."
fi
