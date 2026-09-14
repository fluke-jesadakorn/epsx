# ============================================================================
# EPSX Pay Service - Rust backend for pay.epsx.io microservice
#
# wave49(slice-1): extracted from `apps/backend/Dockerfile` pattern
# (the monolith backend's Dockerfile). The build context MUST be the
# WORKSPACE ROOT (not services/pay). The `epsx-pay-svc` binary has
# workspace-relative path deps to `epsx-kernel`, `epsx-web3`, and
# `epsx-observability`; the workspace root `Cargo.toml` +
# `Cargo.lock` must be in scope. `.dockerignore` keeps the build
# context manageable.
#
# Build strategy: single cargo build (no stub-swap). The previous
# stub-swap pattern was producing 332KB stub binaries in the
# monolith. The pay-svc binary is small (~9MB) and the build is
# fast — no caching tricks needed.
# ============================================================================

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

COPY . .

WORKDIR /workspace/services/pay
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin pay-service

# ============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libpq5 libssl3 curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /workspace/target/release/pay-service ./pay-service

USER nonroot:nonroot

ENV RUST_LOG=info RUST_BACKTRACE=0 RUST_ENV=production ENV=production HOST=0.0.0.0 PORT=8103
EXPOSE 8103
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
  CMD curl -sf http://localhost:8103/health || exit 1

CMD ["./pay-service"]