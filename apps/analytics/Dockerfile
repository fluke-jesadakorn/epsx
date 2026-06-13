# EPSX Analytics microservice (wave 12 track A) - Rust binary for the
# out-of-process analytics routes (/api/analytics/rankings, /filters,
# /countries, /available-countries, /sectors).
#
# Build context: the WORKSPACE ROOT (not apps/analytics). The new
# crate has path deps to the monolith lib (`../backend`) and the
# shared kernel (`../../shared/rust/epsx-contracts`), plus every
# numeric/string dep comes from `workspace = true` so we need the
# root `Cargo.toml`. `.dockerignore` keeps the build context
# manageable.
#
# Build strategy: single cargo build (no stub-swap). The previous
# stub-swap pattern was producing 332KB stub binaries instead of
# the 7MB real binary. Single-build is slower on cache misses
# but reliable.

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

COPY . .

WORKDIR /workspace/apps/analytics
RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin epsx-analytics-service

# ============================================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/epsx-analytics-service /app/epsx-analytics-service
EXPOSE 8080
CMD ["/app/epsx-analytics-service"]
