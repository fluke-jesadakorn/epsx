# Build stage
FROM oven/bun:1.2.2 AS builder

WORKDIR /app

# Copy necessary files
COPY package.json bun.lockb ./
COPY . .

# Install dependencies and build
RUN bun install --frozen-lockfile
RUN bun run build

# Production stage
FROM oven/bun:1.2.2-slim AS runner

WORKDIR /app

# Set to production environment
ENV NODE_ENV=production
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

# Copy necessary files from builder
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/next.config.ts ./

# TODO: Future improvements
# - Add health check endpoint
# - Implement proper logging configuration
# - Consider adding security scanning
# - Setup proper cache management for build optimization
# - Implement rate limiting for API routes
# - Add monitoring and observability tools
# - Setup automated backups for persistent data
# - Implement circuit breakers for external services

# Expose port
EXPOSE 3000

# Start the application
CMD ["bun", "server.js"]
