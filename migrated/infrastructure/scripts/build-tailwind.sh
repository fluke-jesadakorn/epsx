#!/bin/bash
# Build Tailwind CSS for EPSX BFFs
# Uses the standalone CLI to avoid Node.js dependency

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$ROOT_DIR/content"

# Try to use npx if available, otherwise fall back to CDN
if command -v npx &> /dev/null; then
    echo "Building Tailwind CSS via npx..."
    npx tailwindcss -i tailwind.css -o "$ROOT_DIR/content/tailwind.compiled.css" --minify 2>&1 || {
        echo "Tailwind build failed, using pre-built CSS..."
    }
else
    echo "npx not found, using pre-built CSS..."
fi

echo "Done."
