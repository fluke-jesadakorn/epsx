#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKEND_DIR="$REPO_ROOT/apps/backend"

cd "$BACKEND_DIR"

echo "Building EPSX backend release binary..."
cargo build --release --locked --bin epsx
echo "Built: $BACKEND_DIR/target/release/epsx"
