# EPSX Frontend (BFF) - Rust dioxus SSR server (wave 11 retro)
#
# BFF (backend-for-frontend) for the public site. Replaces the
# old Next.js app in apps-old/frontend. Built on dioxus-ssr +
# axum; serves the SSR'd pages and the /api/v1/* portfolio
# routes. The SSR'd output uses the shared renderer/templates
# crates so the look matches the pre-migration frontend.
#
# The binary is `bff-frontend` ([[bin]] name in Cargo.toml).
# Default bind: 0.0.0.0:3000 (overridable via HOST + PORT env
# vars per the main.rs defaults).
#
# Build context: the WORKSPACE ROOT (not apps/frontend), so
# the path deps to the shared kernel + dioxus-ui + bff crates
# resolve. The new crate has many transitive workspace deps;
# `.dockerignore` keeps the build context manageable (skips
# target/, node_modules/, .next/).

FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/workspace/target

# Copy the workspace source (path deps require it).
COPY . .

# ---- Stage 1: dep cache (stub main so cargo resolves all
#                  transitive deps without compiling the
#                  dioxus-ssr output). ----
WORKDIR /workspace/apps/frontend
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-frontend && \
    rm -rf src target/release/deps/bff_frontend* target/release/bff-frontend

# ---- Stage 2: real source over the stub, re-link. ----
COPY apps/frontend/src/ ./src/

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
