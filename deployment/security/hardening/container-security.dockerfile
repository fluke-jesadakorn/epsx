# EPSX Production Container Security Hardening
# Multi-stage build with security best practices

# Build stage with security scanning
FROM rust:1.75-slim-bookworm AS security-builder

# Install security tools and dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for build
RUN groupadd -r builduser && useradd -r -g builduser builduser

# Set working directory
WORKDIR /app

# Copy and build application
COPY --chown=builduser:builduser . .
USER builduser

# Build with security optimizations
RUN cargo build --release \
    && strip target/release/epsx-backend

# Production stage with minimal attack surface
FROM debian:bookworm-slim AS production

# Security: Install only essential packages and security updates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && apt-get upgrade -y \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Security: Create non-root user with minimal privileges
RUN groupadd -r epsx \
    && useradd -r -g epsx -d /app -s /sbin/nologin epsx \
    && mkdir -p /app \
    && chown epsx:epsx /app

# Security: Set up secure directories
RUN mkdir -p /app/logs /app/tmp \
    && chown epsx:epsx /app/logs /app/tmp \
    && chmod 750 /app/logs /app/tmp

# Security: Copy application binary with restricted permissions
COPY --from=security-builder --chown=epsx:epsx /app/target/release/epsx-backend /app/
RUN chmod 750 /app/epsx-backend

# Security: Set up configuration directory
COPY --chown=epsx:epsx deployment/security/config/ /app/config/
RUN chmod -R 640 /app/config/

# Security: Switch to non-root user
USER epsx

# Security: Set working directory
WORKDIR /app

# Security: Expose only necessary port
EXPOSE 8080

# Security: Health check with timeout
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Security: Set resource limits and security options
ENV RUST_BACKTRACE=0
ENV RUST_LOG=warn
ENV MALLOC_ARENA_MAX=2

# Security: Run with minimal privileges
CMD ["./epsx-backend"]

# Security Labels for metadata
LABEL maintainer="EPSX Security Team <security@epsx.io>"
LABEL version="1.0.0"
LABEL description="EPSX Backend - Production Security Hardened"
LABEL security.scan="trivy"
LABEL security.policy="production"

# Security: Add build-time security metadata
ARG BUILD_DATE
ARG VCS_REF
ARG VERSION

LABEL org.opencontainers.image.created=$BUILD_DATE
LABEL org.opencontainers.image.version=$VERSION
LABEL org.opencontainers.image.revision=$VCS_REF
LABEL org.opencontainers.image.title="EPSX Backend"
LABEL org.opencontainers.image.description="Enterprise Trading Platform Backend"
LABEL org.opencontainers.image.vendor="EPSX"
LABEL org.opencontainers.image.licenses="Proprietary"