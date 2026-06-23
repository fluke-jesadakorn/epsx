# ============================================================================
# EPSX Pay BFF - Dioxus SSR server for pay.epsx.io
#
# wave49(slice-1): extracted from `apps/frontend/Dockerfile` pattern.
# The bff-pay binary has workspace-relative path deps to `epsx-kernel`,
# `epsx-client`, `epsx-observability`, `epsx-templates`; the
# workspace root `Cargo.toml` + `Cargo.lock` must be in scope.
#
# Binary: `bff-pay` ([[bin]] name in apps/pay/Cargo.toml).
# Default bind: 0.0.0.0:3002 (overridable via HOST + PORT env vars).
# ============================================================================

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

# Copy the workspace source (path deps require it).
COPY . .

# Build the real binary directly from the real source. No stub
# cache, no second rebuild. Slower on cache misses but produces
# a real binary reliably.
WORKDIR /workspace/apps/pay
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-pay

# ============================================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /workspace/target/release/bff-pay ./bff-pay

USER nonroot:nonroot

ENV RUST_LOG=info RUST_BACKTRACE=0 RUST_ENV=production ENV=production HOST=0.0.0.0 PORT=3002
EXPOSE 3002
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
  CMD curl -sf http://localhost:3002/api/health || exit 1

CMD ["./bff-pay"]