# EPSX Backend Binary Size Optimization Guide

This guide explains how to build optimized versions of the EPSX backend for different deployment scenarios.

## Overview

The EPSX backend uses Cargo feature flags and build profiles to enable selective compilation, allowing you to build binaries with only the features you need for specific deployments.

## Feature Flags

### Core Features

- `database` - PostgreSQL database support via Diesel ORM
- `cache` - Redis caching layer
- `auth` - Authentication subsystem (JWT, Firebase, crypto)
- `websockets` - Real-time WebSocket support
- `templates` - HTML template engine (Askama)
- `api-docs` - OpenAPI documentation generation (utoipa)
- `http-client` - HTTP client support (reqwest)
- `cli-tools` - Command-line utilities (clap)

### Security Features

- `jwt` - JWT token handling (jsonwebtoken, hmac, sha2)
- `firebase` - Firebase authentication (google-cloud-auth, rsa, x509-parser)
- `crypto` - Encryption support (argon2, aes-gcm, ring)

### TLS Options (Mutually Exclusive)

- `tls-rustls` - Pure Rust TLS implementation (recommended)
- `tls-native` - Native system TLS (smaller but platform-dependent)

### Preset Combinations

- `default` - Full feature set for development
- `production` - All production features with rustls
- `minimal` - Basic server with database and JWT only

## Build Profiles

### Development Profile (`dev`)
```toml
opt-level = 0           # No optimization (fast compile)
debug = true            # Full debug info
lto = false            # No link-time optimization
panic = "unwind"       # Full panic handling
codegen-units = 256    # Parallel compilation
```

### Release Profile (`release`)
```toml
opt-level = 3          # Full optimization
debug = false          # No debug info
strip = "symbols"      # Remove symbols
lto = "thin"          # Thin LTO
panic = "abort"       # Smaller panic handling
codegen-units = 1     # Single unit for better optimization
```

### Size-Optimized Profile (`release-small`)
```toml
opt-level = "z"        # Optimize for size
lto = "fat"           # Full link-time optimization
panic = "abort"       # Remove panic unwinding
strip = "symbols"     # Remove debug symbols
codegen-units = 1     # Single compilation unit
```

### Performance Profile (`release-fast`)
```toml
opt-level = 3         # Maximum runtime optimization
lto = "thin"         # Balanced LTO
panic = "abort"      # Fast panic handling
codegen-units = 1    # Better optimization
```

## Quick Build Commands

### Minimal Deployment (Serverless/Microservices)
```bash
cargo build --profile release-small --features "minimal"
```
**Use for:** AWS Lambda, Google Cloud Functions, minimal microservices
**Size:** ~5-10MB (estimated)
**Features:** Database, JWT auth, HTTP client

### Production Deployment (Full Features)
```bash
cargo build --profile release --features "production"
```
**Use for:** Standard containerized deployments
**Size:** ~15-25MB (estimated)
**Features:** All production features with caching and real-time support

### API-Only Deployment
```bash
cargo build --profile release-small --features "database,auth,http-client,tls-rustls"
```
**Use for:** REST API services without WebSockets
**Size:** ~8-15MB (estimated)
**Features:** Database, authentication, HTTP client

### High-Performance Deployment
```bash
cargo build --profile release-fast --features "production"
```
**Use for:** Trading systems requiring maximum throughput
**Size:** ~20-30MB (estimated)
**Features:** All features optimized for runtime performance

### Database Migration Tool
```bash
cargo build --profile release-small --features "database,cli-tools" --bin migrate
```
**Use for:** Schema migrations and database operations
**Size:** ~3-8MB (estimated)
**Features:** Database access and CLI tools only

## Using the Makefile

The project includes a comprehensive Makefile for easy builds:

```bash
# Development
make check              # Check compilation
make test              # Run tests
make build-dev         # Development build

# Production builds
make build-prod        # Full production build
make build-minimal     # Smallest binary
make build-fast        # Performance optimized
make build-api         # API-only build

# Analysis
make size-analysis     # Compare binary sizes
make security-audit    # Security vulnerability check
```

## Deployment-Specific Recommendations

### Serverless Functions (AWS Lambda, GCF)
- **Profile:** `release-small`
- **Features:** `minimal` or custom subset
- **Why:** Cold start performance and size limits
- **Command:** `make build-minimal`

### Kubernetes/Container Deployments
- **Profile:** `release`
- **Features:** `production`
- **Why:** Balanced optimization for containerized environments
- **Command:** `make build-prod`

### High-Frequency Trading Systems
- **Profile:** `release-fast`
- **Features:** `production`
- **Why:** Maximum runtime performance for latency-sensitive operations
- **Command:** `make build-fast`

### API Gateway/Load Balancer
- **Profile:** `release-small`
- **Features:** `database,auth,http-client,tls-rustls`
- **Why:** Minimal feature set for request routing
- **Command:** `make build-api`

### Analytics/Read-Only Services
- **Profile:** `release-small`
- **Features:** `database,cache,http-client,tls-rustls`
- **Why:** Cached read operations without auth complexity
- **Command:** Custom build with specific features

## Size Optimization Techniques

### 1. Feature Selection
Remove unused features to eliminate dependencies:
```bash
# Instead of default features
cargo build --no-default-features --features "database,jwt,http-client"
```

### 2. Profile Optimization
Use size-optimized profiles:
```bash
cargo build --profile release-small
```

### 3. Link-Time Optimization (LTO)
Enable LTO for smaller binaries:
```toml
[profile.release-small]
lto = "fat"  # or "thin" for faster builds
```

### 4. Panic Strategy
Use `abort` strategy to remove unwinding code:
```toml
[profile.release]
panic = "abort"
```

### 5. Symbol Stripping
Remove debug symbols:
```toml
[profile.release]
strip = "symbols"
```

### 6. Dependency Audit
Regularly check for duplicate dependencies:
```bash
cargo tree --duplicates
```

## Performance vs Size Trade-offs

| Optimization | Binary Size | Compile Time | Runtime Performance |
|-------------|-------------|--------------|-------------------|
| `release-small` | Smallest | Longest | Good |
| `release` | Medium | Medium | Very Good |
| `release-fast` | Largest | Medium | Best |
| `dev` | Large | Fastest | Poor |

## Troubleshooting

### Feature Compilation Errors
If you get "unresolved import" errors when building with minimal features:
1. Check that the code using those imports is behind appropriate feature gates
2. Verify feature dependencies in Cargo.toml
3. Ensure conditional compilation attributes are correct

### Large Binary Size
1. Check enabled features: `cargo build --features "minimal"`
2. Use size-optimized profile: `--profile release-small`
3. Enable LTO: Add `lto = "fat"` to profile
4. Strip symbols: Add `strip = "symbols"` to profile

### Slow Compilation
1. Reduce optimization level for development
2. Use incremental compilation: `incremental = true`
3. Increase codegen units: `codegen-units = 256`
4. Use `dev` profile for development builds

## Example Docker Multi-Stage Build

```dockerfile
# Build stage
FROM rust:1.85-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --profile release-small --features "production"

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release-small/epsx /usr/local/bin/epsx
EXPOSE 8080
CMD ["epsx"]
```

This multi-stage build produces a minimal container with just the optimized binary and runtime dependencies.