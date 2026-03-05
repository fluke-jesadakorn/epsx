# EPSX Admin Frontend - Next.js 16 standalone production build

FROM oven/bun:1.2.17 AS deps
WORKDIR /app

COPY package.json bun.lock ./
COPY apps/admin-frontend/package.json ./apps/admin-frontend/
COPY shared/package.json ./shared/

RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install

FROM node:22-alpine AS builder
WORKDIR /app

ARG NEXT_PUBLIC_BACKEND_URL
ARG NEXT_PUBLIC_APP_URL
ARG NEXT_PUBLIC_ADMIN_URL
ARG NEXT_PUBLIC_BLOCKCHAIN_NETWORK
ARG NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID
ARG NEXT_PUBLIC_CHAIN_ID
ARG NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET
ARG NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET
ARG NEXT_PUBLIC_OAUTH_CLIENT_ID
ARG NEXT_PUBLIC_TURNSTILE_SITE_KEY

ENV NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL
ENV NEXT_PUBLIC_APP_URL=$NEXT_PUBLIC_APP_URL
ENV NEXT_PUBLIC_ADMIN_URL=$NEXT_PUBLIC_ADMIN_URL
ENV NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK
ENV NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID
ENV NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID
ENV NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=$NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET
ENV NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=$NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET
ENV NEXT_PUBLIC_OAUTH_CLIENT_ID=$NEXT_PUBLIC_OAUTH_CLIENT_ID
ENV NEXT_PUBLIC_TURNSTILE_SITE_KEY=$NEXT_PUBLIC_TURNSTILE_SITE_KEY

COPY --from=deps /app/node_modules ./node_modules

COPY package.json bun.lock turbo.json ./
COPY apps/admin-frontend ./apps/admin-frontend
COPY shared ./shared

WORKDIR /app/apps/admin-frontend
ENV NODE_ENV=production
# Clear proxy vars before build (CI runners inject these, corrupts middleware compilation)
ENV http_proxy= https_proxy= HTTP_PROXY= HTTPS_PROXY=
RUN npx next build --webpack

FROM node:22-alpine AS runner
WORKDIR /app

RUN apk add --no-cache libc6-compat

RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

ENV NODE_ENV=production
ENV NODE_OPTIONS="--max-old-space-size=512"
ENV HOSTNAME=0.0.0.0
ENV PORT=3000

COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/.next/static ./apps/admin-frontend/.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/admin-frontend/public ./apps/admin-frontend/public

USER nextjs
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3000/api/health', (res) => process.exit(res.statusCode === 200 ? 0 : 1)).on('error', () => process.exit(1))"

CMD ["node", "apps/admin-frontend/server.js"]
