# Stage 1: Base with optimized Node.js (Frontend Standalone)
FROM --platform=$BUILDPLATFORM node:22-alpine AS base

WORKDIR /app

# Install only essential packages
RUN apk add --no-cache libc6-compat dumb-init

# Stage 2: Dependencies (maximize caching)
FROM base AS deps

# Copy package files for dependency caching
COPY package.json package-lock.json* ./

# Install dependencies
RUN npm ci --frozen-lockfile

# Stage 3: Builder (optimized build process)
FROM base AS builder

# Copy cached dependencies
COPY --from=deps /app/node_modules ./node_modules

# Copy source code (exclude dev files via .dockerignore)
COPY . .

# Build optimization environment
ENV NODE_OPTIONS="--max-old-space-size=4096 --experimental-specifier-resolution=node"
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production
ENV NEXT_PUBLIC_BUILD_MODE=production

# Optimized build with standalone output
RUN npm run build

# Stage 4: Runtime (ultra-minimal for Cloud Run)
FROM --platform=$TARGETPLATFORM node:22-alpine AS runner

WORKDIR /app

# Install only runtime essentials
RUN apk add --no-cache dumb-init \
    && addgroup --system --gid 1001 nodejs \
    && adduser --system --uid 1001 nextjs

# Production environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_OPTIONS="--max-old-space-size=512"

# Copy optimized Next.js standalone output
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static
COPY --from=builder --chown=nextjs:nodejs /app/public ./public

# Health check for Cloud Run
HEALTHCHECK --interval=30s --timeout=10s --start-period=15s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

# Security and performance
USER nextjs

# Network configuration
EXPOSE 3000

# Cloud Run environment
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Graceful shutdown support
STOPSIGNAL SIGTERM

# Use dumb-init for proper signal handling
CMD ["dumb-init", "node", "server.js"]