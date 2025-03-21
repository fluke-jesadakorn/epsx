# Stage 1: Builder
FROM rust:latest AS builder

WORKDIR /app

# Install required system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY . .

# Build release version with explicit target dir
RUN cargo build --release --target-dir /app/target

# Remove debug symbols to reduce binary size further
RUN strip /app/target/release/backend

# Stage 2: Runtime
FROM gcr.io/distroless/cc-debian12 AS runtime

# Copy the binary from builder
COPY --from=builder /app/target/release/backend /app/backend

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
