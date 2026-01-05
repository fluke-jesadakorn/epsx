#!/bin/bash

# Grant Analytics Permissions to Wallet Address
# This script grants analytics permissions to a Web3 wallet address using the proper API endpoint

WALLET_ADDRESS="0x2aE367c0b689153954d8e099782D61037bb423be"
BACKEND_URL="http://localhost:8080"

# Analytics permissions to grant
PERMISSIONS=(
    "epsx:analytics:view"
    "epsx:analytics:basic"
    "epsx:analytics:premium"
    "epsx:analytics:professional"
)

echo "=== Granting Analytics Permissions ==="
echo "Wallet Address: $WALLET_ADDRESS"
echo "Backend URL: $BACKEND_URL"
echo ""

# Function to grant a single permission
grant_permission() {
    local permission=$1
    echo "Granting permission: $permission"
    
    curl -X POST "$BACKEND_URL/api/auth/web3/permissions/grant" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer admin-token" \
        -d '{
            "wallet_address": "'"$WALLET_ADDRESS"'",
            "permission": "'"$permission"'",
            "granted_by": "admin_script",
            "reason": "Manual grant for analytics access"
        }' \
        --silent --show-error
    
    if [ $? -eq 0 ]; then
        echo "✅ Successfully granted: $permission"
    else
        echo "❌ Failed to grant: $permission"
    fi
    echo ""
}

# Grant all analytics permissions
for permission in "${PERMISSIONS[@]}"; do
    grant_permission "$permission"
    sleep 1  # Small delay between requests
done

echo "=== Permission Grant Complete ==="
echo "The wallet address should now have analytics access."
echo "Try accessing: http://localhost:3000/analytics"