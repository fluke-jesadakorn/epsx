#!/bin/bash
set -e

# Load .env variables from project root
# We use 'set -a' to automatically export all variables
set -a
[ -f "../../.env" ] && . "../../.env"
set +a

# Move to backend directory where diesel configs live
cd "$(dirname "$0")/.."

# Default values if not in .env (but .env matches these usually)
DB_USER=${DB_USER:-postgres}
DB_PASS=${DB_PASSWORD:-password}
DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}

# Base Connection String (without DB name)
BASE_CONN="postgresql://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT"

echo "🚀 Starting Automatic Migration for All Databases..."

# ---------------------------------------------------------
# 1. Main Database (epsx_dev)
# ---------------------------------------------------------
echo ""
echo "📦 [1/4] Main Database (epsx_dev)"
export DATABASE_URL="$BASE_CONN/epsx_dev"
echo "   URL: $DATABASE_URL"
# Create DB if needed, and run migrations
diesel database setup --migration-dir migrations/core

# ---------------------------------------------------------
# 2. Analytics Database (epsx_analytics_dev)
# ---------------------------------------------------------
echo ""
echo "📊 [2/4] Analytics Database (epsx_analytics_dev)"
export DATABASE_URL="$BASE_CONN/epsx_analytics_dev"
echo "   URL: $DATABASE_URL"
diesel database setup --config-file diesel_analytics.toml

# ---------------------------------------------------------
# 3. Notifications Database (epsx_notifications_dev)
# ---------------------------------------------------------
echo ""
echo "🔔 [3/4] Notifications Database (epsx_notifications_dev)"
export DATABASE_URL="$BASE_CONN/epsx_notifications_dev"
echo "   URL: $DATABASE_URL"
diesel database setup --config-file diesel_notifications.toml

# ---------------------------------------------------------
# 4. Payments Database (epsx_payments_dev)
# ---------------------------------------------------------
echo ""
echo "cJ [4/4] Payments Database (epsx_payments_dev)"
export DATABASE_URL="$BASE_CONN/epsx_payments_dev"
echo "   URL: $DATABASE_URL"
diesel database setup --config-file diesel_payments.toml

echo ""
echo "✅ All databases migrated successfully!"
