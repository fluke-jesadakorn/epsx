# EPSX Backend - Rust + Axum + Diesel ORM + PostgreSQL
# Production-ready API server with TLS support (build from project root)
FROM --platform=linux/amd64 rust:1.85-slim AS builder

WORKDIR /app

# Install build dependencies including PostgreSQL development libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    gcc-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Add x86_64 target
RUN rustup target add x86_64-unknown-linux-gnu

# Copy workspace root and backend Cargo files from monorepo context
COPY apps/backend/Cargo.toml ./
# Create Cargo.lock from backend directory
RUN cargo generate-lockfile || echo "Using existing Cargo.lock if available"
COPY apps/backend/src/ ./src/

# Copy askama templates and configuration (required at compile time)
COPY apps/backend/askama.toml ./
COPY apps/backend/templates/ ./templates/

# Copy diesel migrations and configuration (required at compile time)
COPY apps/backend/diesel_migrations/ ./diesel_migrations/
COPY apps/backend/diesel.toml ./

# Build the application with production features
RUN CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CC=x86_64-linux-gnu-gcc \
    RUSTFLAGS="-C target-cpu=x86-64 -C link-arg=-s -C opt-level=3" \
    cargo build --release --features production --bin epsx --target x86_64-unknown-linux-gnu && \
    cp /app/target/x86_64-unknown-linux-gnu/release/epsx /app/epsx-binary

# Runtime stage - use debian slim for better library compatibility
FROM --platform=linux/amd64 debian:bookworm-slim AS runtime

# Install only runtime dependencies needed for PostgreSQL TLS
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot

WORKDIR /app

# Copy optimized binary
COPY --from=builder --chown=nonroot:nonroot /app/epsx-binary /app/epsx

# Change to non-root user
USER nonroot:nonroot

# Cloud Run optimized environment
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0
ENV MALLOC_ARENA_MAX=2
ENV RUST_ENV=production
ENV ENV=production
ENV HOST=0.0.0.0
ENV PORT=8080

# Database connection pool optimized for Cloud Run
ENV DATABASE_MAX_CONNECTIONS=2
ENV DATABASE_MIN_CONNECTIONS=1
ENV DATABASE_ACQUIRE_TIMEOUT=60
ENV DATABASE_IDLE_TIMEOUT=300

# Production configuration - these must be provided as environment variables during deployment:
# DATABASE_URL - PostgreSQL database connection string
# NEXTAUTH_SECRET - JWT signing secret (32+ characters)
# FIREBASE_PROJECT_ID - Firebase project identifier

# Security hardening
USER nonroot:nonroot

# Network configuration
EXPOSE 8080

# Graceful shutdown support for Cloud Run
STOPSIGNAL SIGTERM

# Run optimized binary
CMD ["/app/epsx"]