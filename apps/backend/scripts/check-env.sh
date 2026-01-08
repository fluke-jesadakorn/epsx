#!/bin/bash
# Check environment configuration for EPSX backend

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"

cd "$BACKEND_DIR"

echo "🔍 Checking EPSX Backend Environment Configuration"
echo "=================================================="
echo ""

# Load .env file if it exists
if [ -f .env ]; then
    echo -e "${GREEN}✅ .env file found${NC}"
    # Source .env safely (handles multiline values)
    set -a
    source .env 2>/dev/null || true
    set +a
else
    echo -e "${YELLOW}⚠️  .env file not found in $BACKEND_DIR${NC}"
    echo "   Create one from the root .env or copy required variables"
fi

echo ""
echo "Required Environment Variables:"
echo "================================"

# Check DATABASE_URL
if [ -n "$DATABASE_URL" ]; then
    echo -e "${GREEN}✅ DATABASE_URL${NC}: ${DATABASE_URL%%@*}@***"
else
    echo -e "${RED}❌ DATABASE_URL${NC}: not set"
fi

# Check ANALYTICS_DATABASE_URL
if [ -n "$ANALYTICS_DATABASE_URL" ]; then
    echo -e "${GREEN}✅ ANALYTICS_DATABASE_URL${NC}: ${ANALYTICS_DATABASE_URL%%@*}@***"
else
    echo -e "${YELLOW}⚠️  ANALYTICS_DATABASE_URL${NC}: not set (will fallback to DATABASE_URL)"
fi

# Check NOTIFICATIONS_DATABASE_URL
if [ -n "$NOTIFICATIONS_DATABASE_URL" ]; then
    echo -e "${GREEN}✅ NOTIFICATIONS_DATABASE_URL${NC}: ${NOTIFICATIONS_DATABASE_URL%%@*}@***"
else
    echo -e "${YELLOW}⚠️  NOTIFICATIONS_DATABASE_URL${NC}: not set (will fallback to DATABASE_URL)"
fi

# Check PAYMENTS_DATABASE_URL
if [ -n "$PAYMENTS_DATABASE_URL" ]; then
    echo -e "${GREEN}✅ PAYMENTS_DATABASE_URL${NC}: ${PAYMENTS_DATABASE_URL%%@*}@***"
else
    echo -e "${YELLOW}⚠️  PAYMENTS_DATABASE_URL${NC}: not set (will fallback to DATABASE_URL)"
fi

# Check REDIS_URL
if [ -n "$REDIS_URL" ]; then
    echo -e "${GREEN}✅ REDIS_URL${NC}: configured (required for notifications)"
else
    echo -e "${YELLOW}⚠️  REDIS_URL${NC}: not set (notifications will not work)"
fi

# Check OIDC secrets
if [ -n "$OIDC_CLIENT_SECRET" ]; then
    echo -e "${GREEN}✅ OIDC_CLIENT_SECRET${NC}: configured"
else
    echo -e "${RED}❌ OIDC_CLIENT_SECRET${NC}: not set"
fi

if [ -n "$OIDC_ADMIN_CLIENT_SECRET" ]; then
    echo -e "${GREEN}✅ OIDC_ADMIN_CLIENT_SECRET${NC}: configured"
else
    echo -e "${RED}❌ OIDC_ADMIN_CLIENT_SECRET${NC}: not set"
fi

# Check Firebase
if [ -n "$FIREBASE_PROJECT_ID" ]; then
    echo -e "${GREEN}✅ FIREBASE_PROJECT_ID${NC}: $FIREBASE_PROJECT_ID"
else
    echo -e "${YELLOW}⚠️  FIREBASE_PROJECT_ID${NC}: not set"
fi

echo ""
echo "Optional Environment Variables:"
echo "================================"

# Check BACKEND_URL
if [ -n "$BACKEND_URL" ]; then
    echo -e "${GREEN}✅ BACKEND_URL${NC}: $BACKEND_URL"
else
    echo -e "${YELLOW}⚠️  BACKEND_URL${NC}: not set (default: http://localhost:8080)"
fi

# Check FRONTEND_URL
if [ -n "$FRONTEND_URL" ]; then
    echo -e "${GREEN}✅ FRONTEND_URL${NC}: $FRONTEND_URL"
else
    echo -e "${YELLOW}⚠️  FRONTEND_URL${NC}: not set (default: http://localhost:3000)"
fi

# Check logging
if [ -n "$RUST_LOG" ]; then
    echo -e "${GREEN}✅ RUST_LOG${NC}: $RUST_LOG"
else
    echo -e "${YELLOW}⚠️  RUST_LOG${NC}: not set (default: info)"
fi

echo ""
echo "=================================================="

# Test database connection
echo ""
echo "🔌 Testing Database Connection..."
if command -v psql &> /dev/null && [ -n "$DATABASE_URL" ]; then
    if psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
        echo -e "${GREEN}✅ Database connection successful${NC}"
    else
        echo -e "${RED}❌ Database connection failed${NC}"
        echo "   Check your DATABASE_URL and ensure PostgreSQL is running"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping database test (psql not found or DATABASE_URL not set)${NC}"
fi

# Test Redis connection
echo ""
echo "🔌 Testing Redis Connection..."
if command -v redis-cli &> /dev/null && [ -n "$REDIS_URL" ]; then
    # Extract host and port from REDIS_URL
    if redis-cli -u "$REDIS_URL" ping &> /dev/null; then
        echo -e "${GREEN}✅ Redis connection successful${NC}"
    else
        echo -e "${YELLOW}⚠️  Redis connection failed (but might work with TLS)${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping Redis test (redis-cli not found or REDIS_URL not set)${NC}"
fi

echo ""
echo "=================================================="
echo "✅ Environment check complete!"
echo ""
echo "Next steps:"
echo "  - Run migrations: ./scripts/migrate.sh"
echo "  - Start backend:  ./scripts/run.sh"
echo "  - Or use cargo:   cargo run"
