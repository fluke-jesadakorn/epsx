#!/bin/bash
# ============================================================================
# EPSX Microservices - Start All Services
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Starting EPSX Microservices..."

# Create data directories
mkdir -p /Users/fluke/epsx-prod/data/{postgresql,redis,minio,loki/{chunks,rules},tempo/{wal,generator/{wal,traces}}}
mkdir -p /Users/fluke/epsx-prod/logs

# Check if PostgreSQL is running
if ! pg_isready -q 2>/dev/null; then
    echo "Starting PostgreSQL..."
    brew services start postgresql@16
    sleep 2
fi

# Create databases if they don't exist
echo "Creating databases..."
for db in epsx_identity epsx_wallet epsx_payment epsx_subscription epsx_content epsx_notification epsx_analytics epsx_indexer; do
    psql -U epsx -tc "SELECT 1 FROM pg_database WHERE datname = '$db'" | grep -q 1 || \
        psql -U epsx -c "CREATE DATABASE $db"
done

# Check if Redis is running
if ! redis-cli ping 2>/dev/null | grep -q PONG; then
    echo "Starting Redis..."
    brew services start redis
fi

# Start all services with Overmind
cd "$ROOT_DIR"
echo "Starting services with Overmind..."
overmind start -f Procfile

echo "EPSX Microservices started successfully!"
echo ""
echo "Services:"
echo "  Frontend:  http://localhost:3000 (epsx.io)"
echo "  Admin:     http://localhost:3001 (admin.epsx.io)"
echo "  Pay:       http://localhost:3002 (pay.epsx.io)"
echo "  Preview:   http://localhost:3003 (preview.epsx.io)"
echo "  API:       http://localhost:8080 (api.epsx.io)"
echo ""
echo "  Identity:  http://localhost:8101"
echo "  Wallet:    http://localhost:8102"
echo "  Payment:   http://localhost:8103"
echo "  Subscription: http://localhost:8104"
echo "  Content:   http://localhost:8105"
echo "  Notification: http://localhost:8106"
echo "  Analytics: http://localhost:8107"
echo "  Indexer:   http://localhost:8108"
echo ""
echo "Monitoring:"
echo "  Prometheus: http://localhost:9090"
echo "  Grafana:    http://localhost:3000 (separate)"
echo "  Loki:       http://localhost:3100"
echo "  Tempo:      http://localhost:3200"
