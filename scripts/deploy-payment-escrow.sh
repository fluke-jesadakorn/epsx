#!/bin/bash

# EPSX Payment Escrow Deployment Script
# Deploys the PaymentEscrow smart contract to BSC Testnet

set -e

echo "🚀 EPSX Payment Escrow Deployment Script"
echo "======================================"

# Check if we're in the right directory
if [ ! -d "apps/contracts" ]; then
    echo "❌ Error: Please run this script from the EPSX root directory"
    echo "   Usage: ./scripts/deploy-payment-escrow.sh"
    exit 1
fi

echo "📋 Prerequisites Check:"
echo "   ✅ Found contracts directory"

# Check if .env file exists in contracts
if [ ! -f "apps/contracts/.env" ]; then
    echo "❌ Error: contracts/.env file not found"
    echo "   Please create apps/contracts/.env with your private key"
    exit 1
fi

echo "   ✅ Found contracts environment file"

# Navigate to contracts directory
cd apps/contracts

echo ""
echo "🔧 Building contracts..."
bun install
bun run compile

echo ""
echo "💰 Checking BNB Testnet Balance..."
WALLET_ADDRESS=$(node -e "
const { ethers } = require('hardhat');
const wallet = new ethers.Wallet(process.env.PRIVATE_KEY || '0x' + require('fs').readFileSync('.env', 'utf8').match(/PRIVATE_KEY=([^\\s]+)/)?.[1]);
console.log(wallet.address);
")

echo "   Wallet: $WALLET_ADDRESS"

# Check balance (this will fail if 0 balance, but that's expected for development)
echo "   Note: If deployment fails, get BNB testnet tokens from: https://testnet.bnbchain.org/faucet-smart"

echo ""
echo "🚀 Deploying to BSC Testnet..."
bun run deploy:testnet

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ DEPLOYMENT SUCCESSFUL!"
    echo ""
    echo "📄 Deployment artifacts saved to: apps/contracts/deployments/testnet.json"
    echo ""
    echo "🔄 Next Steps:"
    echo "   1. Copy the contract address from deployments/testnet.json"
    echo "   2. Update NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET in .env file"
    echo "   3. Restart frontend: bun dev:frontend"
    echo ""
else
    echo ""
    echo "❌ DEPLOYMENT FAILED!"
    echo ""
    echo "💡 Troubleshooting:"
    echo "   1. Get BNB testnet tokens: https://testnet.bnbchain.org/faucet-smart"
    echo "   2. Check your wallet address above has sufficient BNB"
    echo "   3. Verify your private key is correct in apps/contracts/.env"
    echo "   4. Try again: ./scripts/deploy-payment-escrow.sh"
    echo ""
    exit 1
fi