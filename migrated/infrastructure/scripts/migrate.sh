#!/bin/bash
# ============================================================================
# EPSX Microservices - Database Migrations
# ============================================================================

set -e

echo "Running database migrations..."

databases=(
    "epsx_identity"
    "epsx_wallet"
    "epsx_payment"
    "epsx_subscription"
    "epsx_content"
    "epsx_notification"
    "epsx_analytics"
    "epsx_indexer"
)

for db in "${databases[@]}"; do
    echo "Migrating $db..."
    psql -U epsx -d "$db" -c "
        CREATE TABLE IF NOT EXISTS _migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) UNIQUE NOT NULL,
            applied_at TIMESTAMPTZ DEFAULT NOW()
        );
    "
done

echo "Migrations complete."
