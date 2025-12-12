# EPSX Frontend - Next.js Standalone Build
# Optimized Docker build for Next.js frontend with proper standalone mode

FROM oven/bun:1-alpine AS builder
WORKDIR /app

# Install system dependencies
RUN apk add --no-cache libc6-compat dumb-init

# Copy monorepo workspace files
COPY package.json bun.lock ./

# Copy frontend app and shared dependencies
COPY apps/frontend/ ./apps/frontend/
COPY shared/ ./shared/

# Copy shared files into frontend app for build resolution
RUN cp -r ./shared ./apps/frontend/

# Install all monorepo dependencies (ignore postinstall scripts for Docker build)
RUN bun install --ignore-scripts

# Build the frontend app with Next.js standalone output
WORKDIR /app/apps/frontend

# Set build environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

# Define build arguments
ARG NEXT_PUBLIC_BACKEND_URL
ARG NEXT_PUBLIC_APP_URL
ARG NEXT_PUBLIC_ADMIN_URL
ARG NEXT_PUBLIC_BLOCKCHAIN_NETWORK
ARG NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID
ARG NEXT_PUBLIC_CHAIN_ID
ARG NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS
ARG NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS
ARG NEXT_PUBLIC_FIREBASE_API_KEY
ARG NEXT_PUBLIC_FIREBASE_PROJECT_ID
ARG NEXT_PUBLIC_FIREBASE_APP_ID
ARG NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
ARG NEXT_PUBLIC_OAUTH_CLIENT_ID

# Create build-time environment for Next.js validation
RUN echo "NODE_ENV=production" > .env && \
    echo "NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL" >> .env && \
    echo "NEXT_PUBLIC_APP_URL=$NEXT_PUBLIC_APP_URL" >> .env && \
    echo "NEXT_PUBLIC_ADMIN_URL=$NEXT_PUBLIC_ADMIN_URL" >> .env && \
    echo "NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK" >> .env && \
    echo "NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID" >> .env && \
    echo "NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID" >> .env && \
    echo "NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS=$NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS" >> .env && \
    echo "NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS=$NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS" >> .env && \
    echo "NEXT_PUBLIC_FIREBASE_API_KEY=$NEXT_PUBLIC_FIREBASE_API_KEY" >> .env && \
    echo "NEXT_PUBLIC_FIREBASE_PROJECT_ID=$NEXT_PUBLIC_FIREBASE_PROJECT_ID" >> .env && \
    echo "NEXT_PUBLIC_FIREBASE_APP_ID=$NEXT_PUBLIC_FIREBASE_APP_ID" >> .env && \
    echo "NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID=$NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID" >> .env && \
    echo "NEXT_PUBLIC_OAUTH_CLIENT_ID=${NEXT_PUBLIC_OAUTH_CLIENT_ID:-epsx-frontend}" >> .env

# Build Next.js app (generates .next/standalone with optimized server.js)
RUN bun run build

# Production runtime stage
FROM node:22-alpine AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache dumb-init

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs && adduser -S nextjs -u 1001

# Copy Next.js standalone output (includes optimized server.js and minimal dependencies)
COPY --from=builder --chown=nextjs:nodejs /app/apps/frontend/.next/standalone ./

# Copy static assets and public files
COPY --from=builder --chown=nextjs:nodejs /app/apps/frontend/.next/static ./apps/frontend/.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/frontend/public ./apps/frontend/public

# Set working directory to where server.js is located in standalone output
WORKDIR /app/apps/frontend

# Switch to non-root user
USER nextjs

# Runtime environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD node -e "require('http').get('http://localhost:3000/api/health', (res) => process.exit(res.statusCode === 200 ? 0 : 1)).on('error', () => process.exit(1))"

# Start application using Next.js generated standalone server
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "server.js"]