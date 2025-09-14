# EPSX Admin Frontend - Next.js + Administrative Dashboard
# User management, permissions, analytics & system monitoring interface
# Stage 1: Base with optimized Node.js
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

# Copy workspace configuration for monorepo support
COPY pnpm-workspace.yaml ./
COPY package.json pnpm-lock.yaml ./

# Copy all app package.json files for workspace resolution
COPY apps/frontend/package.json ./apps/frontend/package.json
COPY apps/admin-frontend/package.json ./apps/admin-frontend/package.json

# Install dependencies as workspace (monorepo-aware)
RUN pnpm install --no-frozen-lockfile

# Stage 3: Builder (optimized build process)
FROM base AS builder

# Build arguments for production (server-side variables)
ARG ADMIN_FRONTEND_URL
ARG NEXTAUTH_SECRET
ARG OIDC_ADMIN_CLIENT_ID  
ARG OIDC_ADMIN_CLIENT_SECRET
ARG BACKEND_URL
# Build arguments for production (client-side variables)
ARG NEXT_PUBLIC_BACKEND_URL
ARG NEXT_PUBLIC_ADMIN_URL
ARG NEXT_PUBLIC_APP_URL
ARG NEXT_PUBLIC_OAUTH_CLIENT_ID
ARG NEXT_PUBLIC_FIREBASE_API_KEY
ARG NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN
ARG NEXT_PUBLIC_FIREBASE_PROJECT_ID
ARG NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET
ARG NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID
ARG NEXT_PUBLIC_FIREBASE_APP_ID

# Copy cached dependencies from workspace
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/apps/admin-frontend/node_modules ./apps/admin-frontend/node_modules

# Copy source code (exclude dev files via .dockerignore)
COPY apps/admin-frontend/ ./apps/admin-frontend/
# Copy shared directory for monorepo dependencies
COPY shared/ ./shared/

# Set working directory to admin-frontend app for build
WORKDIR /app/apps/admin-frontend

# Build optimization environment
ENV NODE_OPTIONS="--max-old-space-size=4096 --experimental-specifier-resolution=node"
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production
ENV NEXT_PUBLIC_BUILD_MODE=production
# Server-side environment variables (for Next.js build process)
ENV ADMIN_FRONTEND_URL=$ADMIN_FRONTEND_URL
ENV NEXTAUTH_SECRET=$NEXTAUTH_SECRET
ENV OIDC_ADMIN_CLIENT_ID=$OIDC_ADMIN_CLIENT_ID
ENV OIDC_ADMIN_CLIENT_SECRET=$OIDC_ADMIN_CLIENT_SECRET
ENV BACKEND_URL=$BACKEND_URL
# Client-side environment variables (exposed to browser)
ENV NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL
ENV NEXT_PUBLIC_ADMIN_URL=$NEXT_PUBLIC_ADMIN_URL
ENV NEXT_PUBLIC_APP_URL=$NEXT_PUBLIC_APP_URL
ENV NEXT_PUBLIC_OAUTH_CLIENT_ID=$NEXT_PUBLIC_OAUTH_CLIENT_ID
ENV NEXT_PUBLIC_FIREBASE_API_KEY=$NEXT_PUBLIC_FIREBASE_API_KEY
ENV NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=$NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN
ENV NEXT_PUBLIC_FIREBASE_PROJECT_ID=$NEXT_PUBLIC_FIREBASE_PROJECT_ID
ENV NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=$NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET
ENV NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=$NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID
ENV NEXT_PUBLIC_FIREBASE_APP_ID=$NEXT_PUBLIC_FIREBASE_APP_ID

# Optimized build without BuildKit cache
RUN pnpm build

# Stage 4: Runtime (Node.js Alpine for reliability)
FROM --platform=linux/amd64 node:22-alpine AS runner

WORKDIR /app

# Install dumb-init for proper signal handling
RUN apk add --no-cache dumb-init

# Create non-root user
RUN addgroup -g 1001 -S nodejs && adduser -S nextjs -u 1001

# Production environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_OPTIONS="--max-old-space-size=512"

# Copy optimized Next.js standalone output with proper structure
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/standalone ./

# Fix broken pnpm symlinks by creating proper module links
COPY --from=builder --chown=nextjs:nodejs /app/node_modules ./node_modules_temp/
RUN rm -rf ./node_modules && mv ./node_modules_temp ./node_modules && \
    cd node_modules && \
    ln -sf .pnpm/next@*/node_modules/next next && \
    ln -sf .pnpm/react@*/node_modules/react react && \
    ln -sf .pnpm/react-dom@*/node_modules/react-dom react-dom

# Copy static files to correct locations for Next.js standalone mode
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/static ./.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/public ./public

# Switch to non-root user
USER nextjs

# Network configuration
EXPOSE 3000

# Cloud Run environment
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Graceful shutdown support
STOPSIGNAL SIGTERM

# Use dumb-init for proper signal handling
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "server.js"]