#!/bin/bash
# ============================================================================
# EPSX Microservices - Production Build & Deploy
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building EPSX Microservices..."

cd "$ROOT_DIR"

# Build all services in release mode
echo "Compiling services..."
cargo build --release

echo "Build complete."
echo ""
echo "To deploy:"
echo "  1. Stop existing services: ./infrastructure/scripts/stop.sh"
echo "  2. Run migrations: ./infrastructure/scripts/migrate.sh"
echo "  3. Start services: ./infrastructure/scripts/start.sh"
echo "  4. Check health: ./infrastructure/scripts/health.sh"
