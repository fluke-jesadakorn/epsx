# EPSX Frontend (BFF) - Rust dioxus SSR server (wave 11 retro)
#
# BFF (backend-for-frontend) for the public site. Replaces the
# old Next.js app in apps-old/frontend. Built on dioxus-ssr +
# axum; serves the SSR'd pages and the /api/v1/* portfolio
# routes.
#
# Binary: `bff-frontend` ([[bin]] name in Cargo.toml).
# Default bind: 0.0.0.0:3000 (overridable via HOST + PORT env vars).
#
# Build strategy: copy source, single cargo build, copy binary
# out. The previous stub-swap pattern (write stub main, build
# for dep cache, copy real source, rebuild) was producing a
# 332KB stub binary instead of the 7MB real binary — likely a
# BuildKit cache-mount state issue where the second cargo build
# was finding the stub binary in the cache. The single-build
# approach is slower (no cache) but reliable.

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
WORKDIR /workspace/apps/frontend
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-frontend

# ============================================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/bff-frontend /app/bff-frontend

ENV HOST=0.0.0.0 PORT=3000 RUST_LOG=info RUST_BACKTRACE=0
EXPOSE 3000
CMD ["/app/bff-frontend"]
