# Build stage
FROM oven/bun:1.2.3-alpine AS builder
WORKDIR /app

# Copy package files first for better caching
COPY apps/frontend/package.json ./
COPY bun.lock ./

# Set environment variables
ENV HUSKY=0 \
    NODE_ENV=production

# Install dependencies with memory optimizations
RUN bun install --frozen-lockfile

# Copy frontend source code
COPY apps/frontend/ .
# Build the application and verify output existence
RUN bun run build && \
    if [ ! -d ".next/standalone" ] || [ ! -d ".next/static" ]; then \
        echo "Error: Build outputs not found" && exit 1; \
    fi

# Production stage
FROM oven/bun:1.2.3-alpine AS runner
WORKDIR /app

# Create app user
RUN addgroup -S -g 1001 bunjs && \
    adduser -S -u 1001 -G bunjs bunjs

# Set environment variables
ENV NODE_ENV=production \
    PORT=3000

# Copy necessary files from builder
COPY --from=builder --chown=bunjs:bunjs /app/public ./public
COPY --from=builder --chown=bunjs:bunjs /app/.next/standalone ./
COPY --from=builder --chown=bunjs:bunjs /app/.next/static ./.next/static
COPY --from=builder --chown=bunjs:bunjs /app/.next/standalone/server.js ./

# Switch to non-root user
USER bunjs

# Expose the port
EXPOSE 3000

# Start the application
CMD ["bun", "server.js"]
