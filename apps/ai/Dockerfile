# Build stage
FROM oven/bun:1.2.2 AS builder
WORKDIR /app
COPY package.json bun.lockb ./
COPY . .
RUN bun install --frozen-lockfile
RUN bun run build

# Production stage
FROM oven/bun:1.2.2-slim AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./
EXPOSE 3000
CMD ["bun", "dist/main.js"]
