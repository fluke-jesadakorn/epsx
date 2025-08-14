#!/bin/bash

# Test User Permission Assignment Script
# Quick shell script to assign admin permissions to test user

set -e

TEST_EMAIL="jesadakorn.kirtnu@gmail.com"
BACKEND_URL="${BACKEND_URL:-http://localhost:8080}"
DATABASE_URL="${DATABASE_URL:-postgresql://localhost/epsx}"

echo "🚀 Assigning admin permissions to test user"
echo "📧 Email: $TEST_EMAIL"
echo "🔧 Backend: $BACKEND_URL"
echo "======================================================"

# Admin modules to assign
ADMIN_MODULES='["system_admin","user_management","permission_management","iam_management","analytics","billing_management","stock_ranking_management","database_management","developer_portal","module_management"]'

# Permissions to assign
PERMISSIONS='["admin:read","admin:write","system:manage","system:configure","user:manage","user:create","user:edit","user:delete","user:read","permission:manage","permission:assign","permission:revoke","permission:read","iam:manage","iam:read","iam:write","analytics:read","analytics:export","analytics:configure","billing:read","billing:manage","billing:export","stock_ranking:manage","stock_ranking:assign","stock_ranking:read","database:read","database:manage","database:backup","developer:read","developer:manage","api:read","api:manage"]'

echo "🔄 Method 1: Trying API assignment..."

# Try API assignment first
API_SUCCESS=false
if curl -s -X POST "$BACKEND_URL/api/v1/admin/users/assign-modules" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer admin-test-token" \
  -d "{\"email\":\"$TEST_EMAIL\",\"admin_modules\":$ADMIN_MODULES,\"permissions\":$PERMISSIONS,\"reason\":\"E2E test setup\"}" \
  --connect-timeout 5 --max-time 10 > /dev/null 2>&1; then
  echo "✅ API assignment successful"
  API_SUCCESS=true
else
  echo "❌ API assignment failed"
fi

# Fallback to database assignment
if [ "$API_SUCCESS" = false ]; then
  echo ""
  echo "🔄 Method 2: Trying database assignment..."
  
  # Create/update user with full permissions
  UPDATE_QUERY="
    INSERT INTO users (
      id, 
      email, 
      name, 
      role, 
      admin_modules, 
      permissions,
      firebase_uid,
      created_at,
      updated_at
    ) VALUES (
      gen_random_uuid(),
      '$TEST_EMAIL',
      'Test Admin User',
      'admin',
      '$ADMIN_MODULES'::jsonb,
      '$PERMISSIONS'::jsonb,
      'test-admin-uid',
      NOW(),
      NOW()
    ) ON CONFLICT (email) DO UPDATE SET
      admin_modules = '$ADMIN_MODULES'::jsonb,
      permissions = '$PERMISSIONS'::jsonb,
      role = 'admin',
      updated_at = NOW();
  "
  
  if psql "$DATABASE_URL" -c "$UPDATE_QUERY" > /dev/null 2>&1; then
    echo "✅ Database assignment successful"
  else
    echo "❌ Database assignment failed"
    echo "⚠️ Manual intervention may be required"
    echo ""
    echo "🔧 To manually assign permissions, run:"
    echo "psql $DATABASE_URL"
    echo "Then execute:"
    echo "$UPDATE_QUERY"
    exit 1
  fi
fi

echo ""
echo "🔍 Verifying user permissions..."

# Try to verify via database
if VERIFY_RESULT=$(psql "$DATABASE_URL" -t -c "SELECT email, role, admin_modules, permissions FROM users WHERE email = '$TEST_EMAIL';" 2>/dev/null); then
  echo "✅ User found in database:"
  echo "$VERIFY_RESULT"
else
  echo "⚠️ Could not verify user in database"
fi

echo ""
echo "🎉 Permission assignment completed!"
echo "💡 You can now run the E2E tests with full admin access"
echo ""
echo "📝 To run the comprehensive E2E test:"
echo "cd /Users/fluke/Desktop/Work/Outsource/epsx/apps/admin-frontend"
echo "npx playwright test all-modules-comprehensive.spec.ts"