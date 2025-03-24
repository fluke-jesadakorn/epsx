# Build stage
FROM oven/bun:1.2.4-alpine AS builder
WORKDIR /app

# Add build memory optimizations
ENV NODE_OPTIONS="--max-old-space-size=4096" \
    NEXT_TELEMETRY_DISABLED=1

# Copy root package files
# Copy monorepo configuration
COPY package.json bun.lock ./
COPY turbo.json ./

# Copy all workspace packages
COPY packages ./packages
COPY apps/frontend ./apps/frontend

# Copy shared configurations
COPY tsconfig.base.json ./

# Set environment variables
ENV HUSKY=0 \
    NODE_ENV=production

# Install dependencies with memory optimizations
RUN bun install

# Build application from the root to maintain workspace context
RUN cd apps/frontend && \
    NEXT_TELEMETRY_DISABLED=1 bun run build && \
    if [ ! -d ".next/standalone" ] || [ ! -d ".next/static" ]; then \
        echo "Error: Build outputs not found" && \
        exit 1; \
    fi && \
    # Move required files into standalone directory maintaining Next.js structure
    mkdir -p .next/standalone/apps/frontend/.next/static && \
    mkdir -p .next/standalone/apps/frontend/public && \
    cp -r .next/static/* .next/standalone/apps/frontend/.next/static/ && \
    cp -r public/* .next/standalone/apps/frontend/public/ && \
    cp next.config.ts .next/standalone/ && \
    cp package.json .next/standalone/

# Production stage
FROM oven/bun:1.2.4-alpine AS runner
WORKDIR /app

# Create app user
RUN addgroup -S -g 1001 bunjs && \
    adduser -S -u 1001 -G bunjs bunjs

# Set environment variables
ENV NODE_ENV=production \
    PORT=3000

# Copy standalone directory contents
COPY --from=builder --chown=bunjs:bunjs /app/apps/frontend/.next/standalone/. ./

# Expose the port
EXPOSE 3000

# Switch to non-root user
USER bunjs

# Start the application
CMD ["bun", "run", "./apps/frontend/server.js"]
