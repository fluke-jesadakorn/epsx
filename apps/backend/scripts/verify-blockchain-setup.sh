#!/bin/bash

# Blockchain Payment System - Setup Verification Script
# Checks that all prerequisites are met for running blockchain-monitor

set -e

echo "🔍 EPSX Blockchain Payment System - Setup Verification"
echo "========================================================"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
ERRORS=0
WARNINGS=0

# Function to check environment variable
check_env_var() {
    local var_name=$1
    local required=$2

    if [ -z "${!var_name}" ]; then
        if [ "$required" = "true" ]; then
            echo -e "${RED}✗${NC} $var_name is not set (REQUIRED)"
            ((ERRORS++))
        else
            echo -e "${YELLOW}⚠${NC} $var_name is not set (optional)"
            ((WARNINGS++))
        fi
    else
        echo -e "${GREEN}✓${NC} $var_name is set"
    fi
}

echo "1. Checking Required Environment Variables"
echo "-------------------------------------------"
check_env_var "DATABASE_URL" true
check_env_var "BLOCKCHAIN_NETWORK" true
check_env_var "BLOCKCHAIN_START_BLOCK" false
check_env_var "BLOCKCHAIN_POLL_INTERVAL_SECONDS" false
echo ""

echo "2. Checking Network-Specific Variables"
echo "---------------------------------------"
if [ "$BLOCKCHAIN_NETWORK" = "testnet" ]; then
    check_env_var "BSC_TESTNET_RPC_URL" true
    check_env_var "PAYMENT_ESCROW_CONTRACT_TESTNET" true

    if [ -n "$PAYMENT_ESCROW_CONTRACT_TESTNET" ] && [ "$PAYMENT_ESCROW_CONTRACT_TESTNET" = "0x0000000000000000000000000000000000000000" ]; then
        echo -e "${YELLOW}⚠${NC} PAYMENT_ESCROW_CONTRACT_TESTNET is set to zero address (deploy contract first)"
        ((WARNINGS++))
    fi
elif [ "$BLOCKCHAIN_NETWORK" = "mainnet" ]; then
    check_env_var "BSC_MAINNET_RPC_URL" true
    check_env_var "PAYMENT_ESCROW_CONTRACT_MAINNET" true

    if [ -n "$PAYMENT_ESCROW_CONTRACT_MAINNET" ] && [ "$PAYMENT_ESCROW_CONTRACT_MAINNET" = "0x0000000000000000000000000000000000000000" ]; then
        echo -e "${RED}✗${NC} PAYMENT_ESCROW_CONTRACT_MAINNET is zero address (CRITICAL)"
        ((ERRORS++))
    fi
else
    echo -e "${YELLOW}⚠${NC} BLOCKCHAIN_NETWORK is set to '$BLOCKCHAIN_NETWORK' (expected 'testnet' or 'mainnet')"
    ((WARNINGS++))
fi
echo ""

echo "3. Checking Database Connection"
echo "--------------------------------"
if [ -n "$DATABASE_URL" ]; then
    if command -v psql &> /dev/null; then
        if psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
            echo -e "${GREEN}✓${NC} Database connection successful"
        else
            echo -e "${RED}✗${NC} Database connection failed"
            ((ERRORS++))
        fi
    else
        echo -e "${YELLOW}⚠${NC} psql not found, skipping database test"
        ((WARNINGS++))
    fi
else
    echo -e "${RED}✗${NC} DATABASE_URL not set, cannot test connection"
    ((ERRORS++))
fi
echo ""

echo "4. Checking Database Schema"
echo "---------------------------"
if [ -n "$DATABASE_URL" ] && command -v psql &> /dev/null; then
    if psql "$DATABASE_URL" -c "\d processed_blockchain_events" &> /dev/null; then
        echo -e "${GREEN}✓${NC} processed_blockchain_events table exists"
    else
        echo -e "${RED}✗${NC} processed_blockchain_events table missing (run migration 008)"
        ((ERRORS++))
    fi

    if psql "$DATABASE_URL" -c "\d wallet_users" &> /dev/null; then
        echo -e "${GREEN}✓${NC} wallet_users table exists"
    else
        echo -e "${RED}✗${NC} wallet_users table missing"
        ((ERRORS++))
    fi

    if psql "$DATABASE_URL" -c "\d active_subscriptions" &> /dev/null; then
        echo -e "${GREEN}✓${NC} active_subscriptions table exists"
    else
        echo -e "${RED}✗${NC} active_subscriptions table missing"
        ((ERRORS++))
    fi

    if psql "$DATABASE_URL" -c "\d pricing_plans" &> /dev/null; then
        echo -e "${GREEN}✓${NC} pricing_plans table exists"
    else
        echo -e "${RED}✗${NC} pricing_plans table missing"
        ((ERRORS++))
    fi
else
    echo -e "${YELLOW}⚠${NC} Skipping schema check (database not accessible)"
    ((WARNINGS++))
fi
echo ""

echo "5. Checking Binary"
echo "------------------"
if [ -f "target/release/blockchain-monitor" ]; then
    echo -e "${GREEN}✓${NC} blockchain-monitor binary exists (release)"
    ls -lh target/release/blockchain-monitor | awk '{print "  Size: " $5}'
elif [ -f "target/debug/blockchain-monitor" ]; then
    echo -e "${YELLOW}⚠${NC} blockchain-monitor binary exists (debug only)"
    echo -e "${YELLOW}⚠${NC} Build release version with: cargo build --release --bin blockchain-monitor"
    ((WARNINGS++))
else
    echo -e "${RED}✗${NC} blockchain-monitor binary not found"
    echo "  Build with: cargo build --release --bin blockchain-monitor"
    ((ERRORS++))
fi
echo ""

echo "6. Checking RPC Endpoint"
echo "------------------------"
if [ "$BLOCKCHAIN_NETWORK" = "testnet" ] && [ -n "$BSC_TESTNET_RPC_URL" ]; then
    if command -v curl &> /dev/null; then
        if curl -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            "$BSC_TESTNET_RPC_URL" -s -o /dev/null -w "%{http_code}" | grep -q "200"; then
            echo -e "${GREEN}✓${NC} BSC Testnet RPC endpoint is accessible"
        else
            echo -e "${RED}✗${NC} BSC Testnet RPC endpoint is not responding"
            ((ERRORS++))
        fi
    else
        echo -e "${YELLOW}⚠${NC} curl not found, skipping RPC test"
        ((WARNINGS++))
    fi
elif [ "$BLOCKCHAIN_NETWORK" = "mainnet" ] && [ -n "$BSC_MAINNET_RPC_URL" ]; then
    if command -v curl &> /dev/null; then
        if curl -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            "$BSC_MAINNET_RPC_URL" -s -o /dev/null -w "%{http_code}" | grep -q "200"; then
            echo -e "${GREEN}✓${NC} BSC Mainnet RPC endpoint is accessible"
        else
            echo -e "${RED}✗${NC} BSC Mainnet RPC endpoint is not responding"
            ((ERRORS++))
        fi
    else
        echo -e "${YELLOW}⚠${NC} curl not found, skipping RPC test"
        ((WARNINGS++))
    fi
else
    echo -e "${YELLOW}⚠${NC} Skipping RPC test (network/URL not configured)"
    ((WARNINGS++))
fi
echo ""

echo "========================================================"
echo "Verification Complete"
echo "========================================================"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "You can now start the blockchain monitor:"
    echo "  ./target/release/blockchain-monitor"
    echo ""
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠ ${WARNINGS} warning(s) found${NC}"
    echo ""
    echo "The system may work but has some warnings."
    echo "Review the warnings above before proceeding."
    echo ""
    exit 0
else
    echo -e "${RED}✗ ${ERRORS} error(s) found, ${WARNINGS} warning(s)${NC}"
    echo ""
    echo "Please fix the errors above before running blockchain-monitor."
    echo ""
    echo "Common fixes:"
    echo "  1. Copy .env.example to .env and fill in values"
    echo "  2. Run database migration: sqlx migrate run"
    echo "  3. Build binary: cargo build --release --bin blockchain-monitor"
    echo "  4. Deploy smart contract and update PAYMENT_ESCROW_CONTRACT_* variables"
    echo ""
    exit 1
fi
