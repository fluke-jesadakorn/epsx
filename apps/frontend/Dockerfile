# Build stage
FROM oven/bun:1.2.3-alpine AS builder
WORKDIR /app

# Add build memory optimizations
ENV NODE_OPTIONS="--max-old-space-size=4096" \
    NEXT_TELEMETRY_DISABLED=1

# Copy root workspace files for monorepo setup
COPY package.json bun.lock turbo.json tsconfig.base.json ./
COPY packages ./packages
COPY apps/frontend/package.json ./apps/frontend/

# Set environment variables
ENV HUSKY=0 \
    NODE_ENV=production

# Install dependencies with memory optimizations
RUN bun install --frozen-lockfile

# Copy frontend source code
COPY apps/frontend/ ./apps/frontend/

# Build the application
WORKDIR /app/apps/frontend
# Build with existing next.config.ts (which has standalone output configured)
RUN NEXT_TELEMETRY_DISABLED=1 bun run build && \
    if [ ! -d ".next/standalone" ] || [ ! -d ".next/static" ]; then \
    echo "Error: Build outputs not found" && \
    exit 1; \
    fi && \
    # Move static files into standalone directory to maintain expected structure
    mkdir -p .next/standalone/.next && \
    cp -r .next/static .next/standalone/.next/

# Production stage
FROM oven/bun:1.2.3-alpine AS runner
WORKDIR /app

# Create app user
RUN addgroup -S -g 1001 bunjs && \
    adduser -S -u 1001 -G bunjs bunjs

# Set environment variables
ENV NODE_ENV=production \
    PORT=3000

# Copy standalone directory contents and public files
COPY --from=builder --chown=bunjs:bunjs /app/apps/frontend/.next/standalone/. ./
COPY --from=builder --chown=bunjs:bunjs /app/apps/frontend/public ./public

# Expose the port
EXPOSE 3000

# Switch to non-root user
USER bunjs

# Start the application
CMD ["bun", "server.js"]
