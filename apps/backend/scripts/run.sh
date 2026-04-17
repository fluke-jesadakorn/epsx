#!/bin/bash
# Helper script to run backend with the merged root env stack

set -e

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(cd "$BACKEND_DIR/../.." && pwd)"

cd "$BACKEND_DIR"

eval "$(node "$REPO_ROOT/scripts/utils/root-env.js" --print-shell)"

# Check required environment variables
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL not set"
    exit 1
fi

if [ -z "$REDIS_URL" ]; then
    echo "⚠️  Warning: REDIS_URL not set (notifications will not work)"
fi

echo "🚀 Starting EPSX backend..."
echo "   DATABASE: ${DATABASE_URL%%@*}@***"
echo "   REDIS: ${REDIS_URL:+configured}"
echo "   PORT: ${PORT:-8080}"
echo ""

cargo run --release
