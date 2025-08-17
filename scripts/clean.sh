#!/bin/bash

# EPSX Build Artifacts Cleanup Script
# Removes all build artifacts to ensure clean builds

set -e

# Change to project root directory
cd "$(dirname "$0")/.."

echo "🧹 Cleaning EPSX build artifacts..."
echo "Working directory: $(pwd)"

# Remove Next.js build artifacts
echo "  Removing .next directories..."
find . -name ".next" -type d -exec rm -rf {} + 2>/dev/null || true

# Remove Rust build artifacts  
echo "  Removing target directories..."
find . -name "target" -type d -exec rm -rf {} + 2>/dev/null || true

# Remove Node.js dependencies (will need to run pnpm install after)
echo "  Removing node_modules directories..."
find . -name "node_modules" -type d -exec rm -rf {} + 2>/dev/null || true

# Remove other build artifacts
echo "  Removing dist directories..."
find . -name "dist" -type d -exec rm -rf {} + 2>/dev/null || true

echo "  Removing build directories..."
find . -name "build" -type d -exec rm -rf {} + 2>/dev/null || true

# Remove cache directories
echo "  Removing cache directories..."
find . -name ".cache" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name ".turbo" -type d -exec rm -rf {} + 2>/dev/null || true

echo "✅ Cleanup completed!"
echo ""
echo "📝 Next steps:"
echo "   1. Run 'pnpm install' to reinstall dependencies"
echo "   2. Run 'pnpm build:packages' to rebuild packages"
echo "   3. Run 'pnpm build:apps' to rebuild applications"