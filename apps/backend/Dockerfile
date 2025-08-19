# Stage 1: Dependency Cache (maximize layer reuse)
FROM --platform=linux/amd64 rust:1.85-slim AS deps

WORKDIR /app

# Install build dependencies (cached layer)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy only dependency files for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Cache dependencies compilation
RUN cargo build --release --target-dir /app/target

# Stage 2: Builder (compile from source)
FROM deps AS builder

# Remove dummy src and copy real source code
RUN rm -rf src
COPY . .

# Build optimized binary for linux/amd64
RUN cargo build --release --target-dir /app/target

# Stage 3: Runtime (ultra-minimal for Cloud Run)
FROM --platform=linux/amd64 gcr.io/distroless/cc-debian12:latest AS runtime

WORKDIR /app

# Copy optimized binaries
COPY --from=builder --chown=nonroot:nonroot /app/target/release/epsx /app/epsx

# SSL certificates for HTTPS (minimal copy)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Cloud Run optimized environment
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0
ENV MALLOC_ARENA_MAX=2

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
