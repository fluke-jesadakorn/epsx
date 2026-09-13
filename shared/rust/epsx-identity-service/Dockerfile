# EPSX Identity microservice (wave 13a Track A) — Rust binary
# exposing a tonic gRPC server with ONE method,
# `GetWalletRankingOffset`, on port 50051.
#
# Build context: the WORKSPACE ROOT (not
# shared/rust/epsx-identity-service/). The new crate has a
# path dep to `epsx-contracts` (../../shared/rust/epsx-contracts)
# plus every numeric/string dep comes from `workspace = true`
# so we need the root `Cargo.toml`. `.dockerignore` keeps the
# build context manageable.
#
# Build strategy: single cargo build (no stub-swap). The
# previous stub-swap pattern was producing 332KB stub binaries
# instead of the 7MB real binary. Single-build is slower on
# cache misses but reliable.
#
# Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.1
# (this track creates that section). Mirrors the wave-12
# analytics Dockerfile at `apps/analytics/Dockerfile`.

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev protobuf-compiler ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

COPY . .

WORKDIR /workspace/shared/rust/epsx-identity-service
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin epsx-identity-service

# ============================================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/epsx-identity-service /app/epsx-identity-service
EXPOSE 50051
CMD ["/app/epsx-identity-service"]
