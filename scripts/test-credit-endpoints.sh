#!/bin/bash

# Credit Wallet System - API Endpoint Test Script
# Tests all credit endpoints with proper authentication

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
USER_TOKEN="${USER_TOKEN:-}"
ADMIN_TOKEN="${ADMIN_TOKEN:-}"
TEST_WALLET="${TEST_WALLET:-0x1234567890123456789012345678901234567890}"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Credit Wallet API Tests${NC}"
echo -e "${BLUE}================================${NC}\n"

# Check if tokens are set
if [ -z "$USER_TOKEN" ]; then
    echo -e "${YELLOW}⚠️  USER_TOKEN not set. Set it with: export USER_TOKEN='your_jwt_token'${NC}"
    echo -e "${YELLOW}   You can get a token by authenticating at /api/auth/web3/verify${NC}\n"
fi

if [ -z "$ADMIN_TOKEN" ]; then
    echo -e "${YELLOW}⚠️  ADMIN_TOKEN not set. Set it with: export ADMIN_TOKEN='your_admin_jwt_token'${NC}\n"
fi

# Test 1: Get Credit Balance (User)
echo -e "${BLUE}Test 1: Get User Credit Balance${NC}"
echo -e "Endpoint: ${GREEN}GET /api/payments/credits/balance${NC}"
if [ -n "$USER_TOKEN" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        -H "Authorization: Bearer $USER_TOKEN" \
        -H "Content-Type: application/json" \
        "$API_URL/api/payments/credits/balance")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Success (200)${NC}"
        echo "$BODY" | jq '.'
    else
        echo -e "${RED}✗ Failed ($HTTP_CODE)${NC}"
        echo "$BODY"
    fi
else
    echo -e "${YELLOW}Skipped (no USER_TOKEN)${NC}"
fi
echo ""

# Test 2: Get Credit History (User)
echo -e "${BLUE}Test 2: Get User Credit History${NC}"
echo -e "Endpoint: ${GREEN}GET /api/payments/credits/history?limit=10${NC}"
if [ -n "$USER_TOKEN" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        -H "Authorization: Bearer $USER_TOKEN" \
        -H "Content-Type: application/json" \
        "$API_URL/api/payments/credits/history?limit=10")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Success (200)${NC}"
        echo "$BODY" | jq '.data.data | length' | xargs -I {} echo "Transactions: {}"
        echo "$BODY" | jq '.data.data[0]' 2>/dev/null || echo "No transactions yet"
    else
        echo -e "${RED}✗ Failed ($HTTP_CODE)${NC}"
        echo "$BODY"
    fi
else
    echo -e "${YELLOW}Skipped (no USER_TOKEN)${NC}"
fi
echo ""

# Test 3: Get Credit Stats (Admin)
echo -e "${BLUE}Test 3: Get System Credit Statistics (Admin)${NC}"
echo -e "Endpoint: ${GREEN}GET /api/payments/admin/credits/stats${NC}"
if [ -n "$ADMIN_TOKEN" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json" \
        "$API_URL/api/payments/admin/credits/stats")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Success (200)${NC}"
        echo "$BODY" | jq '.'
    else
        echo -e "${RED}✗ Failed ($HTTP_CODE)${NC}"
        echo "$BODY"
    fi
else
    echo -e "${YELLOW}Skipped (no ADMIN_TOKEN)${NC}"
fi
echo ""

# Test 4: Grant Credits (Admin)
echo -e "${BLUE}Test 4: Grant Credits to User (Admin)${NC}"
echo -e "Endpoint: ${GREEN}POST /api/payments/admin/credits/grant${NC}"
if [ -n "$ADMIN_TOKEN" ]; then
    GRANT_DATA=$(cat <<EOF
{
  "wallet_address": "$TEST_WALLET",
  "amount": 50.00,
  "reason": "Test grant via API test script",
  "expires_at": null
}
EOF
)

    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json" \
        -d "$GRANT_DATA" \
        "$API_URL/api/payments/admin/credits/grant")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Success (200) - Granted \$50.00 to $TEST_WALLET${NC}"
        echo "$BODY" | jq '.'
    else
        echo -e "${RED}✗ Failed ($HTTP_CODE)${NC}"
        echo "$BODY"
    fi
else
    echo -e "${YELLOW}Skipped (no ADMIN_TOKEN)${NC}"
fi
echo ""

# Test 5: Get User Credits (Admin)
echo -e "${BLUE}Test 5: Get User Credit Details (Admin)${NC}"
echo -e "Endpoint: ${GREEN}GET /api/payments/admin/credits/:wallet${NC}"
if [ -n "$ADMIN_TOKEN" ]; then
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json" \
        "$API_URL/api/payments/admin/credits/$TEST_WALLET?limit=5")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Success (200)${NC}"
        echo "$BODY" | jq '.data.balance'
        echo "$BODY" | jq '.data.transactions | length' | xargs -I {} echo "Recent transactions: {}"
    else
        echo -e "${RED}✗ Failed ($HTTP_CODE)${NC}"
        echo "$BODY"
    fi
else
    echo -e "${YELLOW}Skipped (no ADMIN_TOKEN)${NC}"
fi
echo ""

# Test 6: Revoke Credits (Admin)
echo -e "${BLUE}Test 6: Revoke Credits from User (Admin)${NC}"
echo -e "Endpoint: ${GREEN}POST /api/payments/admin/credits/revoke${NC}"
if [ -n "$ADMIN_TOKEN" ]; then
    REVOKE_DATA=$(cat <<EOF
{
  "wallet_address": "$TEST_WALLET",
  "amount": 10.00,
  "reason": "Test revoke via API test script"
}
EOF
)

    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json" \
        -d "$REVOKE_DATA" \
        "$API_URL/api/payments/admin/credits/revoke")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Success (200) - Revoked \$10.00 from $TEST_WALLET${NC}"
        echo "$BODY" | jq '.'
    else
        echo -e "${RED}✗ Failed ($HTTP_CODE)${NC}"
        echo "$BODY"
    fi
else
    echo -e "${YELLOW}Skipped (no ADMIN_TOKEN)${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo -e "User Endpoints (require USER_TOKEN):"
echo -e "  - GET /api/payments/credits/balance"
echo -e "  - GET /api/payments/credits/history"
echo ""
echo -e "Admin Endpoints (require ADMIN_TOKEN with admin:credits:manage permission):"
echo -e "  - GET  /api/payments/admin/credits/:wallet"
echo -e "  - POST /api/payments/admin/credits/grant"
echo -e "  - POST /api/payments/admin/credits/revoke"
echo -e "  - GET  /api/payments/admin/credits/stats"
echo ""
echo -e "${BLUE}To run tests:${NC}"
echo -e "  1. Start backend: ${GREEN}bun dev:backend${NC}"
echo -e "  2. Get auth tokens (authenticate via Web3)"
echo -e "  3. Export tokens:"
echo -e "     ${GREEN}export USER_TOKEN='your_user_jwt'${NC}"
echo -e "     ${GREEN}export ADMIN_TOKEN='your_admin_jwt'${NC}"
echo -e "  4. Run script: ${GREEN}./scripts/test-credit-endpoints.sh${NC}"
echo ""
