#!/bin/bash
# Helper script to run backend with .env loaded

set -e

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"

cd "$BACKEND_DIR"

# Load .env file if it exists
if [ -f .env ]; then
    echo "✅ Loading environment from .env"
    set -a
    source .env
    set +a
else
    echo "⚠️  Warning: .env file not found in $BACKEND_DIR"
    echo "    Using system environment variables"
fi

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
