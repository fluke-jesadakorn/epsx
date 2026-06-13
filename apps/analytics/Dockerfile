# EPSX Analytics microservice (wave 12 track A) - Rust binary for the
# out-of-process analytics routes (/api/analytics/rankings, /filters,
# /countries, /available-countries, /sectors).
#
# Build context: the WORKSPACE ROOT (not apps/analytics). The new
# crate has path deps to the monolith lib (`../backend`) and the
# shared kernel (`../../shared/rust/epsx-contracts`), plus every
# numeric/string dep comes from `workspace = true` so we need the
# root `Cargo.toml`. `.dockerignore` keeps the build context
# manageable (skips `target/`, `node_modules/`, `.next/`, etc.).

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy workspace root, including the real source under apps/analytics/src.
# We do NOT include any target/ — that's created by cargo inside this stage.
COPY . .

WORKDIR /workspace/apps/analytics

# ---- Single RUN: stub-build → restore real source → real build ----
# BuildKit's `--mount=type=cache` is ephemeral across `RUN` steps,
# so we do the whole sequence in one shell `RUN`. We back up the
# real src before the stub clobbers it, then restore.
RUN set -e; \
    echo "==> backing up real source"; \
    cp -r src src-real; \
    echo "==> installing stub main for dep cache"; \
    echo "fn main() {}" > src/main.rs; \
    find src -mindepth 1 -maxdepth 1 ! -name main.rs -exec rm -rf {} +; \
    echo "==> building stub for dep cache (this is the slow part)"; \
    cargo build --release --bin epsx-analytics-service; \
    echo "==> restoring real source"; \
    rm -rf src && mv src-real src; \
    echo "==> building real binary"; \
    cargo build --release --bin epsx-analytics-service; \
    echo "==> binary check:"; \
    ls -la /workspace/target/release/epsx-analytics-service

# ============================================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/target/release/epsx-analytics-service /app/epsx-analytics-service
EXPOSE 8080
CMD ["/app/epsx-analytics-service"]
