# EPSX Admin (BFF) - Rust dioxus SSR server (wave 11 retro)
#
# BFF for the admin app (the wave-9+ replacement for the
# pre-dioxus Next.js admin frontend in apps-old/admin-frontend).
# Binary: `bff-admin` ([[bin]] name in Cargo.toml).
# Default bind: 0.0.0.0:3001.
#
# Build strategy: copy source, single cargo build, copy binary
# out. The previous stub-swap pattern (write stub main, build
# for dep cache, copy real source, rebuild) was producing a
# 332KB stub binary instead of the 7MB real binary. The
# single-build approach is slower on cache misses but reliable.

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

COPY . .

WORKDIR /workspace/apps/admin
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-admin

# ============================================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/bff-admin /app/bff-admin

ENV HOST=0.0.0.0 PORT=3001 RUST_LOG=info RUST_BACKTRACE=0
EXPOSE 3001
CMD ["/app/bff-admin"]
