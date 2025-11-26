#!/bin/bash
# Helper script to run migrations with .env loaded (Diesel)

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

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL not set"
    echo "   Please create .env file with DATABASE_URL"
    exit 1
fi

echo "📦 Running database migrations with Diesel..."
echo "   DATABASE_URL: ${DATABASE_URL%%@*}@***"

# Build and run the Diesel migration tool
cargo run --bin migrate -- up

echo "✅ Migrations completed successfully!"
echo ""
echo "💡 Other migration commands:"
echo "   ./scripts/migrate.sh status    - Check migration status"
echo "   cargo run --bin migrate -- help    - See all commands"
