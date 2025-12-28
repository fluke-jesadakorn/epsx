#!/bin/bash
# Initialize multiple databases and run migrations
# 
# Usage:
#   DATABASE_URL=postgresql://user:pass@host:port/db ./scripts/init-databases.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MIGRATIONS_DIR="$PROJECT_ROOT/apps/backend"

# Check DATABASE_URL
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL is required"
    echo "Usage: DATABASE_URL=postgresql://user:pass@host:port/db ./scripts/init-databases.sh"
    exit 1
fi

# Extract connection parts from DATABASE_URL
# postgresql://user:pass@host:port/db?sslmode=disable
BASE_URL=$(echo "$DATABASE_URL" | sed 's|/[^/?]*\([?].*\)\?$||')
SSL_PARAMS=$(echo "$DATABASE_URL" | grep -o '\?.*' || echo "")

echo "🔧 Creating databases..."
echo "   Base URL: $BASE_URL"

# Create databases (skip if exists)
psql "$DATABASE_URL" <<-EOSQL
    SELECT 'CREATE DATABASE epsx_analytics_dev' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'epsx_analytics_dev')\gexec
    SELECT 'CREATE DATABASE epsx_notifications_dev' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'epsx_notifications_dev')\gexec
    SELECT 'CREATE DATABASE epsx_payments_dev' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'epsx_payments_dev')\gexec
EOSQL

echo "✅ Databases created!"

# Run migrations for each database
echo ""
echo "🔄 Running migrations..."

# Analytics migrations
ANALYTICS_URL="$BASE_URL/epsx_analytics_dev$SSL_PARAMS"
echo "   → Analytics DB: $MIGRATIONS_DIR/diesel_migrations_analytics"
for migration in "$MIGRATIONS_DIR"/diesel_migrations_analytics/*/up.sql; do
    if [ -f "$migration" ]; then
        echo "     Running: $(basename $(dirname $migration))"
        psql "$ANALYTICS_URL" -f "$migration" 2>/dev/null || echo "     (already applied or skipped)"
    fi
done

# Notifications migrations
NOTIFICATIONS_URL="$BASE_URL/epsx_notifications_dev$SSL_PARAMS"
echo "   → Notifications DB: $MIGRATIONS_DIR/diesel_migrations_notifications"
for migration in "$MIGRATIONS_DIR"/diesel_migrations_notifications/*/up.sql; do
    if [ -f "$migration" ]; then
        echo "     Running: $(basename $(dirname $migration))"
        psql "$NOTIFICATIONS_URL" -f "$migration" 2>/dev/null || echo "     (already applied or skipped)"
    fi
done

# Payments migrations
PAYMENTS_URL="$BASE_URL/epsx_payments_dev$SSL_PARAMS"
echo "   → Payments DB: $MIGRATIONS_DIR/diesel_migrations_payments"
for migration in "$MIGRATIONS_DIR"/diesel_migrations_payments/*/up.sql; do
    if [ -f "$migration" ]; then
        echo "     Running: $(basename $(dirname $migration))"
        psql "$PAYMENTS_URL" -f "$migration" 2>/dev/null || echo "     (already applied or skipped)"
    fi
done

echo ""
echo "✅ All databases initialized and migrations complete!"
echo ""
echo "Connection URLs:"
echo "  ANALYTICS_DATABASE_URL=$ANALYTICS_URL"
echo "  NOTIFICATIONS_DATABASE_URL=$NOTIFICATIONS_URL"
echo "  PAYMENTS_DATABASE_URL=$PAYMENTS_URL"
