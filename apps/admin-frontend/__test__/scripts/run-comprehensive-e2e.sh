#!/bin/bash

# Comprehensive E2E Test Runner
# Assigns permissions and runs complete admin module tests

set -e

echo "🚀 EPSX Admin E2E Test Suite"
echo "============================="
echo ""

# Configuration
TEST_EMAIL="jesadakorn.kirtnu@gmail.com"
ADMIN_FRONTEND_PORT="3001"
BACKEND_PORT="8080"

echo "📧 Test User: $TEST_EMAIL"
echo "🖥️ Admin Frontend: http://localhost:$ADMIN_FRONTEND_PORT"
echo "⚙️ Backend: http://localhost:$BACKEND_PORT"
echo ""

# Check if services are running
echo "🔍 Checking service availability..."

if curl -s "http://localhost:$BACKEND_PORT/health" > /dev/null 2>&1; then
  echo "✅ Backend is running"
else
  echo "❌ Backend is not running on port $BACKEND_PORT"
  echo "💡 Please start the backend with: cd apps/backend && cargo run"
  exit 1
fi

if curl -s "http://localhost:$ADMIN_FRONTEND_PORT" > /dev/null 2>&1; then
  echo "✅ Admin frontend is running"
else
  echo "❌ Admin frontend is not running on port $ADMIN_FRONTEND_PORT"
  echo "💡 Please start the admin frontend with: cd apps/admin-frontend && npm run dev"
  exit 1
fi

echo ""
echo "🔧 Step 1: Assigning test user permissions..."
if ./assign-permissions.sh; then
  echo "✅ Permission assignment completed"
else
  echo "⚠️ Permission assignment had issues, but continuing with tests"
fi

echo ""
echo "⏳ Waiting for permissions to propagate..."
sleep 3

echo ""
echo "🧪 Step 2: Running comprehensive E2E tests..."

cd ..

# Run the comprehensive test
if npx playwright test all-modules-comprehensive.spec.ts --reporter=line; then
  echo ""
  echo "🎉 Comprehensive E2E tests completed successfully!"
  echo "📊 All admin modules have been tested"
  echo ""
  echo "📋 Test Coverage:"
  echo "  ✅ Dashboard"
  echo "  ✅ User Management"
  echo "  ✅ Permission Management"
  echo "  ✅ IAM Management"
  echo "  ✅ Analytics"
  echo "  ✅ Billing"
  echo "  ✅ System Administration"
  echo "  ✅ Stock Ranking"
  echo "  ✅ Access Control"
  echo ""
else
  echo ""
  echo "❌ Some E2E tests failed"
  echo "📋 Check the test output above for details"
  echo ""
  echo "🔧 Common troubleshooting:"
  echo "  1. Ensure both backend and frontend are running"
  echo "  2. Check if user has correct permissions"
  echo "  3. Verify OAuth configuration"
  echo "  4. Check browser console for errors"
  echo ""
  exit 1
fi

echo "📈 To view detailed test results:"
echo "npx playwright show-report"