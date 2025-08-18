# EPSX Backend Makefile
# Provides convenient build targets for different deployment scenarios

.PHONY: help clean check test build-dev build-prod build-minimal build-fast build-api build-migrate
.PHONY: size-analysis security-audit deny-check docker-build

# Default target
help:
	@echo "🦀 EPSX Backend Build Targets"
	@echo "============================="
	@echo ""
	@echo "Development:"
	@echo "  check          - Check code compilation without building"
	@echo "  test           - Run all tests"
	@echo "  build-dev      - Development build (fast compilation)"
	@echo ""
	@echo "Production Builds:"
	@echo "  build-prod     - Production build (full features, balanced optimization)"
	@echo "  build-minimal  - Minimal build (smallest binary size)"
	@echo "  build-fast     - Performance build (maximum runtime speed)"
	@echo "  build-api      - API-only build (no WebSockets/templates)"
	@echo "  build-migrate  - Database migration tool only"
	@echo ""
	@echo "Analysis:"
	@echo "  size-analysis  - Compare binary sizes across build profiles"
	@echo "  security-audit - Run cargo-deny security audit"
	@echo "  deny-check     - Check dependencies for security issues"
	@echo ""
	@echo "Docker:"
	@echo "  docker-build   - Build optimized Docker image"
	@echo ""
	@echo "Utilities:"
	@echo "  clean          - Clean all build artifacts"

# Development builds
check:
	@echo "🔍 Checking code compilation..."
	cargo check

check-minimal:
	@echo "🔍 Checking minimal feature compilation..."
	cargo check --no-default-features --features "minimal"

test:
	@echo "🧪 Running tests..."
	cargo test

build-dev:
	@echo "🛠️  Building development version..."
	cargo build

# Production builds
build-prod:
	@echo "🚀 Building production version (full features)..."
	cargo build --profile release --features "production"
	@echo "✅ Production build complete: target/release/epsx"

build-minimal:
	@echo "📦 Building minimal version (smallest size)..."
	cargo build --profile release-small --features "minimal"
	@echo "✅ Minimal build complete: target/release-small/epsx"

build-fast:
	@echo "⚡ Building high-performance version..."
	cargo build --profile release-fast --features "production"
	@echo "✅ Fast build complete: target/release-fast/epsx"

build-api:
	@echo "🌐 Building API-only version..."
	cargo build --profile release-small --features "database,auth,http-client,tls-rustls"
	@echo "✅ API build complete: target/release-small/epsx"

build-migrate:
	@echo "🗄️  Building database migration tool..."
	cargo build --profile release-small --features "database,cli-tools" --bin migrate
	@echo "✅ Migration tool complete: target/release-small/migrate"

# Analysis targets
size-analysis: build-minimal build-prod build-fast
	@echo "📊 Binary Size Analysis"
	@echo "======================="
	@echo "Minimal build (release-small):"
	@ls -lh target/release-small/epsx 2>/dev/null || echo "  Binary not found"
	@echo ""
	@echo "Production build (release):"
	@ls -lh target/release/epsx 2>/dev/null || echo "  Binary not found"
	@echo ""
	@echo "Performance build (release-fast):"
	@ls -lh target/release-fast/epsx 2>/dev/null || echo "  Binary not found"
	@echo ""
	@echo "💡 Tip: Use 'make build-minimal' for smallest deployments"

security-audit:
	@echo "🔒 Running security audit..."
	cargo audit

deny-check:
	@echo "🚫 Running dependency audit with cargo-deny..."
	cargo deny check

# Docker builds
docker-build:
	@echo "🐳 Building optimized Docker image..."
	docker build -f Dockerfile.production -t epsx-backend:latest .
	@echo "✅ Docker image built: epsx-backend:latest"

# Utility targets
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete"

# Feature testing
test-features:
	@echo "🧪 Testing different feature combinations..."
	@echo "Testing minimal features..."
	cargo check --no-default-features --features "minimal"
	@echo "Testing database only..."
	cargo check --no-default-features --features "database"
	@echo "Testing auth only..."
	cargo check --no-default-features --features "auth"
	@echo "✅ Feature testing complete"

# Benchmarking
bench:
	@echo "🏃 Running benchmarks..."
	cargo bench --profile bench

# Complete optimization pipeline
optimize: clean check test-features build-minimal build-prod size-analysis security-audit
	@echo "🎯 Complete optimization pipeline finished!"
	@echo ""
	@echo "📊 Results Summary:"
	@make size-analysis