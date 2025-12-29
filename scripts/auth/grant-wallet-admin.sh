#!/bin/bash

# Grant Admin Permissions to Wallet Address
# This script grants admin permissions to a Web3 wallet address using the proper API endpoint

WALLET_ADDRESS="0x2aE367c0b689153954d8e099782D61037bb423be"
BACKEND_URL="http://localhost:8080"

# Admin permissions to grant
PERMISSIONS=(
    "admin:*:*"
    "admin:users:view"
    "admin:users:manage"
    "admin:users:create"
    "admin:users:delete"
    "admin:users:permissions"
    "admin:system:view"
    "admin:system:manage"
    "admin:analytics:view"
    "admin:analytics:manage"
    "admin:security:view"
    "admin:security:manage"
    "admin:notifications:view"
    "admin:notifications:manage"
    "admin:settings:view"
    "admin:settings:manage"
)

echo "=== Granting Admin Permissions ==="
echo "Wallet Address: $WALLET_ADDRESS"
echo "Backend URL: $BACKEND_URL"
echo ""

# Function to grant a single permission
grant_permission() {
    local permission=$1
    echo "Granting permission: $permission"
    
    curl -X POST "$BACKEND_URL/api/v1/auth/web3/permissions/grant" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer admin-token" \
        -d '{
            "wallet_address": "'"$WALLET_ADDRESS"'",
            "permission": "'"$permission"'",
            "granted_by": "admin_script",
            "reason": "Manual grant for admin access"
        }' \
        --silent --show-error
    
    if [ $? -eq 0 ]; then
        echo "✅ Successfully granted: $permission"
    else
        echo "❌ Failed to grant: $permission"
    fi
    echo ""
}

# Grant all admin permissions
for permission in "${PERMISSIONS[@]}"; do
    grant_permission "$permission"
    sleep 1  # Small delay between requests
done

echo "=== Admin Permission Grant Complete ==="
echo "The wallet address should now have full admin access."
echo "Try accessing: http://localhost:3001/"