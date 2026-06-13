# EPSX Admin (BFF) - Rust dioxus SSR server (wave 11 retro)
#
# BFF for the admin app (the wave-9+ replacement for the
# pre-dioxus Next.js admin frontend in apps-old/admin-frontend).
# Binary: `bff-admin` ([[bin]] name in Cargo.toml).
# Default bind: 0.0.0.0:3001 (admin is on a different port
# than the public frontend, which is :3000). The K8s base
# manifest's existing patch sets the service NodePort to
# 3000-target (the pre-migration Next.js port) — this
# Dockerfile corrects that to 3001 via the PORT env var,
# and the matching K8s service patch in
# infrastructure/kubernetes/overlays/{dev,staging,prod}/patches
# must be updated to NodePort the admin service's container
# port 3001.

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

COPY . .

WORKDIR /workspace/apps/admin
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-admin && \
    rm -rf src target/release/deps/bff_admin* target/release/bff-admin

COPY apps/admin/src/ ./src/

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
