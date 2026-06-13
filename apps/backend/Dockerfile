# EPSX Backend - Rust API server for production deployment
#
# Build context: the WORKSPACE ROOT (not apps/backend). The
# monolith's `epsx` lib has workspace-relative path deps to
# `epsx-contracts` and other shared kernel crates; the
# workspace root `Cargo.toml` + `Cargo.lock` must be in scope.
# `.dockerignore` keeps the build context manageable.
#
# Build strategy: single cargo build (no stub-swap). The previous
# stub-swap pattern was producing 332KB stub binaries.

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

COPY . .

WORKDIR /workspace/apps/backend
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --features="cli-tools" --bin epsx --bin migrate

# ============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libpq5 libssl3 curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /workspace/target/release/epsx ./epsx
COPY --from=builder --chown=nonroot:nonroot /workspace/target/release/migrate ./migrate

USER nonroot:nonroot

ENV RUST_LOG=info RUST_BACKTRACE=0 RUST_ENV=production ENV=production HOST=0.0.0.0 PORT=8080
EXPOSE 8080
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=15s --timeout=5s --start-period=60s --retries=3 \
  CMD curl -sf http://localhost:8080/health || exit 1

CMD ["./epsx"]
