# Stage 1: Chef Dependencies (ultra-fast dependency caching) - Force x86_64
FROM --platform=linux/amd64 rust:1.85-slim@sha256:9f841bbe9e7d8e37ceb96ed907265a3a0df7f44e3737d0b100e7907a679acb36 AS chef

WORKDIR /app

# Install cargo-chef and cross-compilation dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    gcc-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
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
# Build dependencies with BuildKit cache mounts for ultra-fast rebuilds (x86_64 target)
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    rustup target add x86_64-unknown-linux-gnu && \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CC=x86_64-linux-gnu-gcc \
    cargo chef cook --release --features container-minimal --recipe-path recipe.json --target x86_64-unknown-linux-gnu

# Stage 4: Builder (compile application)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY . .
# Build application with cache mounts and explicit x86_64 target
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    rustup target add x86_64-unknown-linux-gnu && \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CC=x86_64-linux-gnu-gcc \
    RUSTFLAGS="-C target-cpu=x86-64 -C link-arg=-s -C opt-level=3" \
    cargo build --release --features container-minimal --bin epsx --target x86_64-unknown-linux-gnu && \
    ls -la /app/target/x86_64-unknown-linux-gnu/release/ && \
    cp /app/target/x86_64-unknown-linux-gnu/release/epsx /app/epsx-binary

# Development stage
FROM chef AS development
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
EXPOSE 8080
CMD ["cargo", "run"]

# Stage 5: Runtime (with glibc for database/TLS connections) - Force x86_64
FROM --platform=linux/amd64 gcr.io/distroless/cc-debian12:latest@sha256:00cc20b928afcc8296b72525fa68f39ab332f758c4f2a9e8d90845d3e06f1dc4 AS runtime

WORKDIR /app

# Copy optimized binary directly from builder (x86_64 target)
COPY --from=builder --chown=nonroot:nonroot /app/epsx-binary /app/epsx

# SSL certificates for HTTPS connections
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
ENV RUST_ENV=production
ENV ENV=production
ENV HOST=0.0.0.0
ENV PORT=8080

# Database connection pool optimized for Cloud Run
ENV DATABASE_MAX_CONNECTIONS=4
ENV DATABASE_MIN_CONNECTIONS=1
ENV DATABASE_ACQUIRE_TIMEOUT=30
ENV DATABASE_IDLE_TIMEOUT=600

# Production database configuration (will be overridden by Cloud Run env vars)
ENV DATABASE_URL="postgresql://neondb_owner:npg_UYc6GMDJfPk8@ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech/neondb?sslmode=require"
ENV NEXTAUTH_SECRET="prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum"  
ENV FIREBASE_PROJECT_ID="epsx-449804"

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
