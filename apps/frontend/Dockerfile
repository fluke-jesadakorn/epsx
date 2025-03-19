# Build stage
FROM oven/bun:1.2.3 AS builder
WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Set environment variables
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1

# Install dependencies
RUN bun install

# Copy source files
COPY . .

# Build application
RUN bun run build

# Production stage
FROM oven/bun:1.2.3-alpine AS runner
WORKDIR /app

# Create non-root user
RUN addgroup -S -g 1001 bunjs && \
    adduser -S -u 1001 -G bunjs bunjs

# Set environment variables
ENV NODE_ENV=production \
    PORT=3000 \
    NEXT_TELEMETRY_DISABLED=1

# Copy necessary files from builder
COPY --from=builder /app/public ./public
COPY --from=builder --chown=bunjs:bunjs /app/.next/standalone ./
COPY --from=builder --chown=bunjs:bunjs /app/.next/static ./.next/static

# Switch to non-root user
USER bunjs

# Expose port
EXPOSE 3000

# Start application
CMD ["bun", "server.js"]
