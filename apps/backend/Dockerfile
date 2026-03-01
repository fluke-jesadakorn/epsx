# syntax=docker/dockerfile:1
# EPSX Backend - Rust API server for production deployment

FROM rust:slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*

COPY apps/backend/Cargo.toml apps/backend/Cargo.lock ./

RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/blockchain_monitor.rs && \
    echo "fn main() {}" > src/bin/migrate.rs && \
    echo "fn main() {}" > src/bin/grant_wallet_permission.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --bin epsx && rm -rf src

COPY apps/backend/src/ ./src/
COPY apps/backend/migrations/ ./migrations/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --features="cli-tools" --bin epsx --bin migrate

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libpq5 libssl3 curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /app/target/release/epsx ./epsx
COPY --from=builder --chown=nonroot:nonroot /app/target/release/migrate ./migrate

USER nonroot:nonroot

ENV RUST_LOG=info RUST_BACKTRACE=0 RUST_ENV=production ENV=production HOST=0.0.0.0 PORT=8080
EXPOSE 8080
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=15s --timeout=5s --start-period=60s --retries=3 \
  CMD curl -sf http://localhost:8080/health || exit 1

CMD ["./epsx"]
