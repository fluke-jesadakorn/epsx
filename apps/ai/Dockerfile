# Build stage
FROM oven/bun:1.2.2 AS builder
WORKDIR /app

# Copy root package.json and lockfile
COPY package.json bun.lockb ./
COPY packages ./packages
COPY apps/ai ./apps/ai
COPY turbo.json ./

# Install dependencies
RUN bun install --frozen-lockfile

# Build the application
RUN bun run build --filter=ai

# Production stage
FROM oven/bun:1.2.2-slim AS runner
WORKDIR /app

# Set production environment
ENV NODE_ENV=production
ENV PORT=8080

# Copy necessary files
COPY --from=builder /app/apps/ai/dist ./dist
COPY --from=builder /app/apps/ai/package.json ./
COPY --from=builder /app/node_modules ./node_modules

# Expose the port that Cloud Run will use
EXPOSE ${PORT}

# Health check for Cloud Run
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:${PORT}/health || exit 1

# Start the application
CMD ["bun", "dist/main.js"]
