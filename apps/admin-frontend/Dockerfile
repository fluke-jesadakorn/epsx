# EPSX Admin Frontend - Docker Build
# Next.js 16 with standalone output for production deployment

# Stage 1: Dependencies
FROM oven/bun:1 AS deps
WORKDIR /app

# Copy package files for dependency installation
COPY package.json bun.lock ./
COPY apps/admin-frontend/package.json ./apps/admin-frontend/
COPY shared/package.json ./shared/

# Install dependencies
RUN bun install --frozen-lockfile

# Stage 2: Builder
FROM oven/bun:1 AS builder
WORKDIR /app

# Build arguments for NEXT_PUBLIC_* environment variables (baked at build time)
ARG NEXT_PUBLIC_BACKEND_URL
ARG NEXT_PUBLIC_APP_URL
ARG NEXT_PUBLIC_ADMIN_URL
ARG NEXT_PUBLIC_BLOCKCHAIN_NETWORK
ARG NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID
ARG NEXT_PUBLIC_CHAIN_ID
ARG NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS
ARG NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS
ARG NEXT_PUBLIC_OAUTH_CLIENT_ID

# Set environment variables for build
ENV NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL
ENV NEXT_PUBLIC_APP_URL=$NEXT_PUBLIC_APP_URL
ENV NEXT_PUBLIC_ADMIN_URL=$NEXT_PUBLIC_ADMIN_URL
ENV NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK
ENV NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID
ENV NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID
ENV NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS=$NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS
ENV NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS=$NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS
ENV NEXT_PUBLIC_OAUTH_CLIENT_ID=$NEXT_PUBLIC_OAUTH_CLIENT_ID

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/apps/admin-frontend/node_modules ./apps/admin-frontend/node_modules

# Copy source code
COPY package.json bun.lock turbo.json ./
COPY apps/admin-frontend ./apps/admin-frontend
COPY shared ./shared

# Build the application
WORKDIR /app/apps/admin-frontend
ENV NODE_ENV=production
RUN bun run build

# Stage 3: Runner
FROM node:22-alpine AS runner
WORKDIR /app

# Install only what's needed for runtime
RUN apk add --no-cache libc6-compat

# Create non-root user for security
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Set production environment
ENV NODE_ENV=production
ENV HOSTNAME=0.0.0.0
ENV PORT=3000

# Copy standalone output
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/static ./apps/admin-frontend/.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/public ./apps/admin-frontend/public

# Switch to non-root user
USER nextjs

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3000/api/health', (res) => process.exit(res.statusCode === 200 ? 0 : 1)).on('error', () => process.exit(1))"

# Start the application
CMD ["node", "apps/admin-frontend/server.js"]
