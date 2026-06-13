# EPSX Analytics microservice (wave 12 track A) - Rust binary for the
# out-of-process analytics routes (/api/analytics/rankings, /filters,
# /countries, /available-countries, /sectors). This is the first wave-9+
# lift that runs as a separate binary (per ROADMAP §4 wave 12) and is
# fronted by the existing Cloudflare Tunnel + reverse-proxy path-based
# routing. The new binary has NO PostgreSQL connections (per Q2 in
# ROADMAP §7) and uses the `epsx-analytics-service` [[bin]] target
# (renamed from `epsx-analytics` to avoid collision with
# services/analytics event-tracking binary).

FROM rust:slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Stub the backend's `[[bin]]` targets so cargo can build the new
# analytics binary's deps without dragging in the full monolith build
# (the monolith Dockerfile does the same for its own binaries). The
# final `cargo build --bin epsx-analytics-service` overwrites the stub
# with the real source.
COPY apps/analytics/Cargo.toml apps/analytics/Cargo.lock ./

RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin epsx-analytics-service && rm -rf src

# Now copy the real source over the stub and re-link. The new binary
# re-exports the moved trees from `epsx` (the monolith lib) via path
# dep `epsx = { path = "../backend" }`, so the backend crate is
# transitively available on the cargo workspace.
COPY apps/analytics/src/ ./src/

RUN --mount=type=cache,sharing=private,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/local/cargo/git \
    cargo build --release --bin epsx-analytics-service

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 nonroot
WORKDIR /app

COPY --from=builder --chown=nonroot:nonroot \
    /app/target/release/epsx-analytics-service ./epsx-analytics-service

USER nonroot:nonroot

ENV RUST_LOG=info RUST_BACKTRACE=0 RUST_ENV=production \
    ENV=production HOST=0.0.0.0 PORT=8080 \
    EPSX_ANALYTICS_VERSION=wave12

EXPOSE 8080
STOPSIGNAL SIGTERM

HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
  CMD curl -sf http://localhost:8080/health || exit 1

CMD ["./epsx-analytics-service"]
