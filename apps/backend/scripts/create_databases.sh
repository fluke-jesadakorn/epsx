#!/bin/bash
# scripts/create_databases.sh
# Create separate databases for each domain if they don't exist

set -e

# Default to user 'postgres' if not set, or extract from DATABASE_URL if possible, 
# but simply running createdb often works if user has permissions.
# We'll try to use the 'createdb' command which uses standard PG env vars.

DATABASES=("epsx_primary" "epsx_analytics" "epsx_notifications" "epsx_payments")
# Also create dev/test versions
DATABASES+=("epsx_dev" "epsx_analytics_dev" "epsx_notifications_dev" "epsx_payments_dev")

echo "🛠️  Checking/Creating Databases..."

for db in "${DATABASES[@]}"; do
    if psql -lqt | cut -d \| -f 1 | grep -qw "$db"; then
        echo "   ✅ Database $db already exists"
    else
        echo "   Creating database: $db"
        createdb "$db" || echo "   ❌ Failed to create $db (you might need to run this manually or check permissions)"
    fi
done

echo "✨ Database setup complete."
