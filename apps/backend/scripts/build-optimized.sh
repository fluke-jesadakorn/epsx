#!/bin/bash

# EPSX Backend Build Optimization Script
# This script demonstrates different build configurations for various deployment scenarios

set -e

echo "🦀 EPSX Backend Build Optimization"
echo "=================================="

# Function to print build info
print_build_info() {
    local features="$1"
    local profile="$2"
    local description="$3"
    
    echo ""
    echo "📦 Building: $description"
    echo "   Features: $features"
    echo "   Profile: $profile"
    echo "   Command: cargo build --profile $profile --features \"$features\""
}

# Function to get binary size
get_binary_size() {
    local binary_path="$1"
    if [ -f "$binary_path" ]; then
        local size=$(du -h "$binary_path" | cut -f1)
        echo "   Binary size: $size"
    else
        echo "   Binary not found: $binary_path"
    fi
}

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build 1: Minimal deployment (microservice, serverless)
print_build_info "minimal" "release-small" "Minimal deployment (microservice/serverless)"
cargo build --profile release-small --features "minimal" --bin epsx
get_binary_size "target/release-small/epsx"

# Build 2: Production deployment (full features)
print_build_info "production" "release" "Production deployment (full features)"
cargo build --profile release --features "production" --bin epsx
get_binary_size "target/release/epsx"

# Build 3: High-performance deployment
print_build_info "production" "release-fast" "High-performance deployment"
cargo build --profile release-fast --features "production" --bin epsx
get_binary_size "target/release-fast/epsx"

# Build 4: API-only deployment (no WebSockets, no templates)
print_build_info "database,auth,http-client,tls-rustls" "release-small" "API-only deployment"
cargo build --profile release-small --features "database,auth,http-client,tls-rustls" --bin epsx
get_binary_size "target/release-small/epsx"

# Build 5: Database-only deployment (for migration scripts)
print_build_info "database,cli-tools" "release-small" "Database migration tool"
cargo build --profile release-small --features "database,cli-tools" --bin migrate
get_binary_size "target/release-small/migrate"

# Build 6: Analytics deployment (no auth, cached data only)
print_build_info "database,cache,http-client,tls-rustls" "release-small" "Analytics service"
cargo build --profile release-small --features "database,cache,http-client,tls-rustls" --bin epsx
get_binary_size "target/release-small/epsx"

echo ""
echo "📊 Build Summary"
echo "================"
echo "1. Minimal deployment: Smallest binary for basic functionality"
echo "2. Production deployment: Full featured, balanced optimization"
echo "3. High-performance: Maximum runtime performance"
echo "4. API-only: REST API without real-time features"
echo "5. Database migration: CLI tool for schema updates"
echo "6. Analytics service: Read-only service with caching"

echo ""
echo "🚀 Deployment Recommendations:"
echo "• Use 'minimal' for serverless functions (AWS Lambda, Google Cloud Functions)"
echo "• Use 'production' for standard containerized deployments"
echo "• Use 'release-fast' for high-throughput trading systems"
echo "• Use 'api-only' for simple REST API services"
echo "• Use specific feature combinations for microservice architectures"

echo ""
echo "📏 Binary Size Optimization Tips:"
echo "• Use --profile release-small for smallest binaries"
echo "• Remove unused features to reduce dependencies"
echo "• Consider 'abort' panic strategy in production"
echo "• Enable LTO (Link Time Optimization) for size reduction"
echo "• Strip debug symbols in production builds"

echo ""
echo "✅ Build optimization complete!"