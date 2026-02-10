#!/bin/bash
set -e

# Configuration
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="epsx_user"
DB_PASS="password"
ADMIN_USER="fluke" # System user with CREATEDB permissions

# Connection string base for running migrations
DB_BASE="postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT"

echo "🧪 Verifying migrations on test databases..."

# 1. Create Test Databases (using admin user)
echo "Creating test databases as $ADMIN_USER..."

# Drop if exists (cleanup)
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE IF EXISTS epsx_core_test;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE IF EXISTS epsx_analytics_test;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE IF EXISTS epsx_notifications_test;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE IF EXISTS epsx_payments_test;"

# Create fresh with proper owner
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "CREATE DATABASE epsx_core_test OWNER $DB_USER;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "CREATE DATABASE epsx_analytics_test OWNER $DB_USER;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "CREATE DATABASE epsx_notifications_test OWNER $DB_USER;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "CREATE DATABASE epsx_payments_test OWNER $DB_USER;"

# 2. Run Migrations (using epsx_user)
echo "Running migrations..."

export DATABASE_URL="$DB_BASE/epsx_core_test"
echo "Running Core Migrations..."
diesel migration run --config-file apps/backend/diesel.toml

export DATABASE_URL="$DB_BASE/epsx_analytics_test"
echo "Running Analytics Migrations..."
diesel migration run --config-file apps/backend/diesel_analytics.toml

export DATABASE_URL="$DB_BASE/epsx_notifications_test"
echo "Running Notifications Migrations..."
diesel migration run --config-file apps/backend/diesel_notifications.toml

export DATABASE_URL="$DB_BASE/epsx_payments_test"
echo "Running Payments Migrations..."
diesel migration run --config-file apps/backend/diesel_payments.toml

# 3. Cleanup
echo "Cleaning up test databases..."
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE epsx_core_test;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE epsx_analytics_test;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE epsx_notifications_test;"
psql -h $DB_HOST -p $DB_PORT -U $ADMIN_USER -d postgres -c "DROP DATABASE epsx_payments_test;"

echo "✅ All migrations passed successfully!"
