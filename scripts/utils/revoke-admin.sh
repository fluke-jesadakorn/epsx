#!/bin/bash

# Revoke all admin access from a user
# Usage: ./revoke-admin.sh user@example.com

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <email>"
    echo "Example: $0 user@example.com"
    echo ""
    echo "This script will REMOVE ALL admin module assignments from the user."
    exit 1
fi

EMAIL=$1

echo "❌ Revoking ALL admin access from user: $EMAIL"
echo "⚠️  This will remove ALL admin module assignments"
echo "======================================================="

read -p "Are you sure you want to revoke ALL admin access from $EMAIL? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Operation cancelled."
    exit 1
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL environment variable is required"
    exit 1
fi

echo "🔍 Looking up user in database..."

# Get firebase_uid for the user
FIREBASE_UID=$(psql "$DATABASE_URL" -t -c "SELECT firebase_uid FROM users WHERE email = '$EMAIL';" | xargs)

if [ -z "$FIREBASE_UID" ]; then
    echo "❌ User with email $EMAIL not found in database"
    exit 1
fi

echo "📧 User: $EMAIL"
echo "🔑 Firebase UID: $FIREBASE_UID"

echo "🗑️  Revoking admin module assignments..."

# Deactivate all admin role assignments
UPDATE_RESULT=$(psql "$DATABASE_URL" -t -c "
    UPDATE user_admin_roles 
    SET is_active = false, updated_at = NOW() 
    WHERE firebase_uid = '$FIREBASE_UID' AND is_active = true;
    SELECT ROW_COUNT();
")

echo "✅ Revoked $UPDATE_RESULT admin module assignments"

echo ""
echo "❌ Admin access revocation completed!"
echo "💡 User $EMAIL no longer has admin access."
echo "💡 You can verify by checking the user_admin_roles table in the database."