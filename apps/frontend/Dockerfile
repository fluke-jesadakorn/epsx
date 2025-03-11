# Build stage
FROM oven/bun:1.2.3 AS builder
WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Turbo globally
RUN bun add -g turbo

# Copy workspace files
COPY . .

# Set build environment
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1

# Install dependencies and build
RUN cd apps/frontend && bun install
RUN bun run turbo build --filter=@epsx/frontend...

# Production stage
FROM oven/bun:1.2.3-alpine AS runner
WORKDIR /app

# Create app user
RUN addgroup --system --gid 1001 bunjs
RUN adduser --system --uid 1001 nextjs

# Set environment variables
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV PORT 3000
ENV HOSTNAME "0.0.0.0"

# Copy necessary files from builder
COPY --from=builder /app/apps/frontend/public ./public
COPY --from=builder --chown=nextjs:bunjs /app/apps/frontend/.next/standalone ./
COPY --from=builder --chown=nextjs:bunjs /app/apps/frontend/.next/static ./.next/static

# Switch to non-root user
USER nextjs

# Expose the port
EXPOSE 3000

# Start the server
CMD ["bun", "server.js"]
