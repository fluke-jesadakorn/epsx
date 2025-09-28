# EPSX Admin Frontend - Next.js Standalone Build
# Optimized Docker build for Next.js admin dashboard with proper standalone mode

FROM node:22-alpine AS builder
WORKDIR /app

# Install system dependencies
RUN apk add --no-cache libc6-compat dumb-init

# Install pnpm
RUN npm install -g pnpm@10.14.0

# Copy monorepo workspace files
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./

# Copy admin frontend app and shared dependencies
COPY apps/admin-frontend/ ./apps/admin-frontend/
COPY shared/ ./shared/

# Copy shared files into admin frontend app for build resolution
RUN cp -r ./shared ./apps/admin-frontend/

# Install all monorepo dependencies
RUN pnpm install --frozen-lockfile

# Build the admin frontend app with Next.js standalone output
WORKDIR /app/apps/admin-frontend

# Set build environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

# Create build-time environment for Next.js validation
RUN echo "NODE_ENV=production" > .env && \
    echo "NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io" >> .env && \
    echo "NEXT_PUBLIC_APP_URL=https://admin.epsx.io" >> .env && \
    echo "NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io" >> .env && \
    echo "NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin" >> .env

# Build Next.js app (generates .next/standalone with optimized server.js)
RUN pnpm build

# Production runtime stage
FROM node:22-alpine AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache dumb-init

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs && adduser -S nextjs -u 1001

# Copy Next.js standalone output (includes optimized server.js and minimal dependencies)
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/standalone ./

# Copy static assets and public files
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/static ./apps/admin-frontend/.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/public ./apps/admin-frontend/public

# Copy missing @swc/helpers from the build node_modules
COPY --from=builder --chown=nextjs:nodejs /app/node_modules/.pnpm/node_modules/@swc/helpers ./node_modules/@swc/helpers

# Set working directory to where server.js is located in standalone output
WORKDIR /app/apps/admin-frontend

# Switch to non-root user
USER nextjs

# Runtime environment
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Expose port
EXPOSE 3000

# Health check for admin dashboard
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD node -e "require('http').get('http://localhost:3000/api/health', (res) => process.exit(res.statusCode === 200 ? 0 : 1)).on('error', () => process.exit(1))"

# Start application using Next.js generated standalone server
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "server.js"]