# EPSX Frontend BFF — Rust/Axum/Dioxus
# Mirrors apps/backend/Dockerfile. See docs/plans/2026-06-11-rust-bff-deployment.md.

FROM rust:slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the workspace manifests + lockfile first so the dependency graph
# is cached independently of source changes.
COPY Cargo.toml Cargo.lock ./
COPY shared/rust/kernel/Cargo.toml       shared/rust/kernel/Cargo.toml
COPY shared/rust/client/Cargo.toml       shared/rust/client/Cargo.toml
COPY shared/rust/auth/Cargo.toml          shared/rust/auth/Cargo.toml
COPY shared/rust/crypto/Cargo.toml        shared/rust/crypto/Cargo.toml
COPY shared/rust/observability/Cargo.toml shared/rust/observability/Cargo.toml
COPY shared/rust/templates/Cargo.toml     shared/rust/templates/Cargo.toml
COPY shared/rust/renderer/Cargo.toml      shared/rust/renderer/Cargo.toml
COPY shared/rust/dioxus_ui/Cargo.toml     shared/rust/dioxus_ui/Cargo.toml
COPY shared/rust/bff/Cargo.toml           shared/rust/bff/Cargo.toml
COPY apps/frontend/Cargo.toml             apps/frontend/Cargo.toml
COPY apps/admin/Cargo.toml                apps/admin/Cargo.toml
COPY apps/backend/Cargo.toml              apps/backend/Cargo.toml
COPY apps/pay/Cargo.toml                  apps/pay/Cargo.toml
COPY apps/preview/Cargo.toml              apps/preview/Cargo.toml

# Stub out the binaries so `cargo build` resolves the dep graph and caches it.
# Only the bff-frontend binary needs a real `fn main()` signature, but we
# stub all of them so a future `cargo build --workspace` still resolves.
RUN mkdir -p apps/frontend/src apps/admin/src apps/backend/src apps/pay/src apps/preview/src && \
    echo "fn main() {}" > apps/frontend/src/main.rs && \
    echo "fn main() {}" > apps/admin/src/main.rs && \
    echo "fn main() {}" > apps/backend/src/main.rs && \
    echo "fn main() {}" > apps/pay/src/main.rs && \
    echo "fn main() {}" > apps/preview/src/main.rs

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-frontend && \
    rm -rf apps/frontend/src apps/admin/src apps/backend/src apps/pay/src apps/preview/src

# Now copy the real source and rebuild just the bff-frontend binary.
COPY shared/ ./shared/
COPY apps/frontend/src/ ./apps/frontend/src/
COPY apps/frontend/public/ ./apps/frontend/public/
COPY apps/backend/src/ ./apps/backend/src/
COPY apps/admin/src/ ./apps/admin/src/
COPY apps/pay/src/ ./apps/pay/src/
COPY apps/preview/src/ ./apps/preview/src/

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin bff-frontend

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot /app/target/release/bff-frontend ./bff-frontend
COPY --from=builder --chown=nonroot:nonroot /app/apps/frontend/public/ ./public/

USER nonroot:nonroot

ENV RUST_LOG=info RUST_BACKTRACE=0 RUST_ENV=production ENV=production \
    HOST=0.0.0.0 PORT=3000 \
    API_URL=https://api.epsx.io

EXPOSE 3000
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=15s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -sf http://localhost:3000/api/health || exit 1

CMD ["./bff-frontend"]
