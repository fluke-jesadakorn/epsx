# Stage 1: Builder
FROM --platform=linux/amd64 rust:latest AS builder

WORKDIR /app

# Install required system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for better caching
COPY apps/backend/Cargo.toml Cargo.toml
COPY apps/backend/src/ src/

# Build release version with explicit target for amd64 Linux
RUN rustup target add x86_64-unknown-linux-gnu && \
    cargo build --release --target x86_64-unknown-linux-gnu --target-dir /app/target

# Remove debug symbols to reduce binary size further
RUN strip /app/target/x86_64-unknown-linux-gnu/release/backend

# Stage 2: Runtime
FROM --platform=linux/amd64 gcr.io/distroless/cc-debian12 AS runtime

# Copy the binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/backend /app/backend

# Required for HTTPS certificates
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app

# Container configuration
ENV RUST_LOG=info
EXPOSE 8080

# Use the non-root user from distroless
USER nonroot:nonroot

# Run the binary
CMD ["./backend"]
