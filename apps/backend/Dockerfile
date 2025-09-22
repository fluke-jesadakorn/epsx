# EPSX Backend - Local Docker Build
# Rust API server with SQLx for Portainer deployment

# Stage 1: Build stage with Rust and dependencies
FROM rust:1.85-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy Cargo files for dependency caching
COPY apps/backend/Cargo.toml apps/backend/Cargo.lock ./

# Create src directory and dummy main.rs for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (for caching)
RUN cargo build --release && rm -rf src

# Copy source code
COPY apps/backend/src/ ./src/

# Copy askama templates (required at compile time)
COPY apps/backend/askama.toml ./
COPY apps/backend/templates/ ./templates/

# Build application with optimizations
RUN cargo build --release --bin epsx

# Stage 2: Runtime stage with minimal Debian
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder --chown=nonroot:nonroot /app/target/release/epsx ./epsx

# Switch to non-root user
USER nonroot:nonroot

# Production environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0
ENV RUST_ENV=production
ENV ENV=production
ENV HOST=0.0.0.0
ENV PORT=8080

# Database connection pool settings optimized for local deployment
ENV DATABASE_MAX_CONNECTIONS=10
ENV DATABASE_MIN_CONNECTIONS=2
ENV DATABASE_ACQUIRE_TIMEOUT=30
ENV DATABASE_IDLE_TIMEOUT=300

# Expose port
EXPOSE 8080

# Graceful shutdown support
STOPSIGNAL SIGTERM

# Run the application
CMD ["./epsx"]