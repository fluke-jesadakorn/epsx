# Stage 1: Chef Dependencies (ultra-fast dependency caching)
FROM --platform=linux/amd64 rust:1.85-slim AS chef

WORKDIR /app

# Install cargo-chef and build dependencies in one layer
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean \
    && cargo install cargo-chef

# Stage 2: Planner (analyze dependencies)
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Dependencies Cache (build dependencies only)
FROM chef AS dependencies
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies with size optimization profile
RUN cargo chef cook --release --profile release-small --features container-full --recipe-path recipe.json

# Stage 4: Builder (compile application)
FROM chef AS builder
COPY . .
# Copy cached dependencies
COPY --from=dependencies /app/target target
# Build application with ultra-small profile
RUN cargo build --release --profile release-small --features container-full --bin epsx

# Development stage
FROM chef AS development
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
EXPOSE 8080
CMD ["cargo", "run"]

# Stage 5: UPX Compression (optional, for maximum size reduction)
FROM --platform=linux/amd64 ubuntu:22.04 AS compressor
RUN apt-get update && apt-get install -y upx-ucl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release-small/epsx /tmp/epsx
# Compress binary (40-60% size reduction)
RUN upx --best --lzma /tmp/epsx -o /app/epsx

# Stage 6: Runtime (ultra-minimal for Cloud Run)
FROM --platform=linux/amd64 gcr.io/distroless/static-debian12:latest AS runtime

WORKDIR /app

# Copy compressed optimized binary
COPY --from=compressor --chown=nonroot:nonroot /app/epsx /app/epsx

# SSL certificates for HTTPS (from chef stage which has them)
COPY --from=chef /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Accept build arguments for production environment
ARG DATABASE_URL
ARG NEXTAUTH_SECRET  
ARG FIREBASE_PROJECT_ID
ARG PORT=8080

# Cloud Run optimized environment
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0
ENV MALLOC_ARENA_MAX=2

# Production database configuration (hardcoded for reliability)
ENV DATABASE_URL="postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require"
ENV NEXTAUTH_SECRET="prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum"  
ENV FIREBASE_PROJECT_ID="epsx-449804"
ENV PORT=8080

# Health check for Cloud Run - remove since Cloud Run has its own health checks
# HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
#     CMD ["/app/epsx", "--health-check"] || exit 1

# Security hardening
USER nonroot:nonroot

# Network configuration
EXPOSE 8080

# Graceful shutdown support for Cloud Run
STOPSIGNAL SIGTERM

# Run optimized binary
CMD ["/app/epsx"]
