#!/bin/bash
# scripts/recover_epsx_dev.sh
# Recover the main epsx_dev database
# Usage: ./scripts/recover_epsx_dev.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
MIGRATIONS_DIR="$PROJECT_ROOT/apps/backend/migrations/core"

eval "$(node "$PROJECT_ROOT/scripts/utils/root-env.js" --print-shell)"

# Default connection info (matches docker-compose defaults)
DB_USER=${DB_USER:-epsx_user}
DB_PASSWORD=${DB_PASSWORD:-password}
DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}

# Connection string for administrative commands (connect to 'postgres')
ADMIN_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/postgres"

# Define databases and their migration directories
# Format: "db_name:migration_dir_name"
DATABASES=(
    "epsx_dev:migrations/core"
    "epsx_analytics_dev:migrations/analytics"
    "epsx_notifications_dev:migrations/notifications"
    "epsx_payments_dev:migrations/payments"
)

echo "🔧 Recovering Databases..."

for entry in "${DATABASES[@]}"; do
    DB_NAME="${entry%%:*}"
    MIGRATION_DIR_NAME="${entry##*:}"
    MIGRATIONS_DIR="$PROJECT_ROOT/apps/backend/$MIGRATION_DIR_NAME"
    TARGET_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
    
    echo "---------------------------------------------------"
    echo "📦 Processing '$DB_NAME'..."
    echo "   Migrations: $MIGRATION_DIR_NAME"

    # 1. Create Database
    if psql "$ADMIN_URL" -c "SELECT 'CREATE DATABASE $DB_NAME' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec" 2>/dev/null; then
        echo "   ✅ Database created (or exists)."
    else
        echo "   ⚠️  Failed to connect to postgres to create DB. Assuming it exists."
    fi

    # 2. Run Migrations
    if [ ! -d "$MIGRATIONS_DIR" ]; then
        echo "   ❌ Migrations directory not found: $MIGRATIONS_DIR"
        continue
    fi
    
    echo "   🔄 Running migrations..."
    count=0
    # Loop through migration folders sorted by name
    for migration in $(ls -d "$MIGRATIONS_DIR"/*/ 2>/dev/null | sort); do
        UP_SQL="${migration}up.sql"
        if [ -f "$UP_SQL" ]; then
            MIGRATION_NAME=$(basename "$migration")
            # echo "     Applying: $MIGRATION_NAME"
            if ! psql "$TARGET_URL" -f "$UP_SQL" > /dev/null 2>&1; then
                 echo "     ⚠️  Failed/Skipped: $MIGRATION_NAME"
            else
                 count=$((count + 1))
            fi
        fi
    done
    echo "   ✅ Applied $count migrations to $DB_NAME."
done

echo ""
echo "🎉 All recoveries complete!"
