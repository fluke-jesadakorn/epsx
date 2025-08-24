# Stage 1: Base with optimized Node.js (Admin Frontend Standalone)
FROM --platform=linux/amd64 node:22-alpine AS base

WORKDIR /app

# Install pnpm and essential packages
RUN apk add --no-cache libc6-compat dumb-init \
    && npm install -g pnpm@10.14.0

# Development stage
FROM base AS development
COPY package.json ./
RUN pnpm install --no-frozen-lockfile
COPY . .
EXPOSE 3000
CMD ["pnpm", "dev"]

# Stage 2: Dependencies (maximize caching)
FROM base AS deps

# Copy package files for dependency caching
COPY apps/admin-frontend/package.json pnpm-lock.yaml ./

# Install dependencies with BuildKit cache mounts
RUN --mount=type=cache,target=/root/.pnpm-store \
    pnpm install --no-frozen-lockfile

# Stage 3: Builder (optimized build process)
FROM base AS builder

# Build arguments for production
ARG ADMIN_URL
ARG NEXTAUTH_SECRET
ARG OIDC_CLIENT_ID  
ARG OIDC_CLIENT_SECRET
ARG BACKEND_URL
ARG NEXT_PUBLIC_BACKEND_URL

# Copy cached dependencies
COPY --from=deps /app/node_modules ./node_modules

# Copy source code (exclude dev files via .dockerignore)
COPY apps/admin-frontend/ .

# Build optimization environment
ENV NODE_OPTIONS="--max-old-space-size=4096 --experimental-specifier-resolution=node"
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production
ENV NEXT_PUBLIC_BUILD_MODE=production
ENV ADMIN_URL=$ADMIN_URL
ENV NEXTAUTH_SECRET=$NEXTAUTH_SECRET
ENV OIDC_CLIENT_ID=$OIDC_CLIENT_ID
ENV OIDC_CLIENT_SECRET=$OIDC_CLIENT_SECRET
ENV BACKEND_URL=$BACKEND_URL
ENV NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL

# Optimized build with cache mounts for faster rebuilds
RUN --mount=type=cache,target=/app/.next/cache \
    --mount=type=cache,target=/root/.pnpm-store \
    pnpm build

# Stage 4: Runtime (ultra-minimal distroless for Cloud Run)
FROM --platform=linux/amd64 gcr.io/distroless/nodejs22-debian12:latest AS runner

WORKDIR /app

# No package installation needed - distroless is pre-configured
# Users are already configured in distroless (nonroot:nonroot)

# Production environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_OPTIONS="--max-old-space-size=512"

# Copy optimized Next.js standalone output (distroless uses nonroot user)
COPY --from=builder --chown=nonroot:nonroot /app/.next/standalone ./
COPY --from=builder --chown=nonroot:nonroot /app/.next/static ./.next/static
COPY --from=builder --chown=nonroot:nonroot /app/public ./public

# Health check removed - Cloud Run handles health checks automatically
# Security: distroless already runs as nonroot user (no USER directive needed)

# Network configuration
EXPOSE 3000

# Cloud Run environment
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Graceful shutdown support
STOPSIGNAL SIGTERM

# Distroless Node.js handles signals properly (no dumb-init needed)
CMD ["server.js"]