# Build the notification microservice from the workspace root.
#
# This image is intentionally independent from the monolith image so the
# service can be rolled out, scaled, and rolled back as its own Rust binary.
FROM rust:slim-bookworm AS builder
WORKDIR /workspace

RUN apt-get update \
    && apt-get install --no-install-recommends -y pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --package epsx-notification --bin notification

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home --home-dir /nonexistent epsx-notification
COPY --from=builder /workspace/target/release/notification /usr/local/bin/epsx-notification
USER 10001:10001
ENV HOST=0.0.0.0 \
    PORT=8106 \
    EPSX_ENV=production
EXPOSE 8106
ENTRYPOINT ["/usr/local/bin/epsx-notification"]
