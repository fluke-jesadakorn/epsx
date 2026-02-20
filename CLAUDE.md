# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Development
- `bun dev` - All services (frontend :3000, admin :3001, backend :8080)
- `bun dev:frontend` / `bun dev:admin` / `bun dev:backend` - Individual apps
- `bun dev:web` - Both frontends without backend
- `bun dev:anvil` - Start local Anvil chain (:8545)
- `bun setup:local` - Deploy contracts & tokens to local chain

### Build
- `bun build` - All apps
- `bun build:dev` / `bun build:prod` - Environment-specific builds

### Lint & Format
- `bun lint` - ESLint all apps
- `bun lint:frontend` / `bun lint:admin` - Individual apps
- `bun lint:fix` - Auto-fix ESLint issues
- `bun type-check` - TypeScript checking
- `bun format` - Prettier format all files

### Test
- `bun test` - All tests (Jest)
- `bun test:frontend` / `bun test:admin` / `bun test:backend` - Per-app
- `bun test:e2e` - Playwright E2E tests
- `bun test:watch` - Watch mode

### Backend (Rust)
- `cargo build` from `apps/backend/`
- `cargo test` from `apps/backend/`
- Binary: `apps/backend/src/bin/migrate.rs` for DB migrations
- Multiple Diesel configs: `diesel.toml`, `diesel_analytics.toml`, `diesel_notifications.toml`, `diesel_payments.toml`
- **Migration safety**: Never drop/delete existing data unless the structural change requires it. Prefer `ALTER TABLE ADD/RENAME` over `DROP`+recreate. Use `IF EXISTS`/`IF NOT EXISTS` guards.

### Deployment (Local Docker + Cloudflare Tunnel)
Production runs locally via Docker Compose with Cloudflare Tunnel exposing services.

**Quick deploy (restart with existing images):**
```bash
cd infrastructure/docker
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --force-recreate
```

**Full rebuild & deploy:**
```bash
export DOCKER_DEFAULT_PLATFORM=linux/arm64
WC_PROJECT_ID="04e0a500abfa1e095bf8f64b15fa2812"

# Build all images
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WC_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=https://epsx.io \
  --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  --build-arg NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=mainnet \
  --build-arg NEXT_PUBLIC_CHAIN_ID=56 \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0 \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF \
  -f apps/frontend/Dockerfile -t epsx-frontend:prod .

docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WC_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  --build-arg NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=mainnet \
  --build-arg NEXT_PUBLIC_CHAIN_ID=56 \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0 \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF \
  -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .

docker build -f apps/backend/Dockerfile -t epsx-backend:prod .

# Deploy
cd infrastructure/docker
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --force-recreate
```

**Key files:**
- `infrastructure/docker/docker-compose.prod.yml` - Service definitions
- `infrastructure/docker/.env.prod` - Env vars (DB creds, JWT secrets, tunnel token)
- `~/.cloudflared/config.yml` - Tunnel ingress routes

**Services & ports:**
| Service | Container | Host Port |
|---------|-----------|-----------|
| Frontend | epsx-prod-frontend | 4700 |
| Admin | epsx-prod-admin | 4701 |
| Backend | epsx-prod-backend | 9180 |
| PostgreSQL | epsx-prod-postgres | 5491 |
| Redis | epsx-prod-redis | 6342 |
| Cloudflared | epsx-prod-cloudflared | - |

**Cloudflare Tunnel:** Config mounted from `~/.cloudflared/`. Routes: `epsx.io` -> frontend:3000, `admin.epsx.io` -> admin:3000, `api.epsx.io` -> backend:8080. Token stored in `.env.prod` as `CLOUDFLARE_TUNNEL_TOKEN`. Refresh with `cloudflared tunnel token epsx-prod`.

**Verify:**
```bash
curl -s https://api.epsx.io/health   # Backend health
curl -s -o /dev/null -w "%{http_code}" https://epsx.io        # Frontend
curl -s -o /dev/null -w "%{http_code}" https://admin.epsx.io  # Admin (307 = OK, redirects to auth)
```

## Architecture

### Monorepo Structure
Bun workspaces + Turborepo. Four apps share code via `shared/`.

```
apps/
  frontend/         # Next.js 16, React 19 - Main analytics platform (:3000)
  admin-frontend/   # Next.js 16, React 19 - Admin portal (:3001)
  backend/          # Rust + Axum - API server (:8080)
  contracts/        # Solidity (Forge) - BSC smart contracts
shared/             # Consolidated TS modules shared across frontends
infrastructure/     # Docker & Cloudflare config
packages/           # NPM packages
scripts/            # Build & deployment scripts
```

### Frontend Tech Stack
- **UI**: Tailwind CSS
- **State**: Zustand + React Context (optimistic updates via `useTransition`)
- **Data**: React Query (TanStack Query v5) + Server Actions
- **Forms**: React Hook Form + Zod
- **Web3**: WAGMI + RainbowKit

### Backend (Rust/Axum) - Clean Architecture + DDD + CQRS
```
src/
  domain/           # Aggregates, value objects, domain events (pure business logic)
  application/      # Command/query handlers, DTOs, controllers
  infrastructure/   # Diesel ORM repos, Redis cache, blockchain adapters, DI container
  web/              # Axum routes, middleware, error handling, validation
  auth/             # Web3 SIWE auth, API key management, permissions
  schemas/          # Diesel auto-generated DB schema
  core/             # AppError, telemetry, shared types
```

Key backend patterns:
- `DomainContainer` for dependency injection (created in main.rs, passed to routes)
- Diesel async with deadpool connection pooling (PostgreSQL + TLS)
- Redis Streams for domain event publishing
- Routes organized by domain: `/api/auth/*`, `/api/analytics/*`, `/api/admin/*`, `/api/public/*`, `/api/users/*`
- WebSocket at `/ws/notifications`
- Domain value objects: `PlanCategory` (Base/Addon/System/Exclusive), `PlanGroup` (Personal/Enterprise/Api/Custom)

### Shared Modules (shared/)
Single source of truth for both frontends:
- `api/` - Domain API clients (UsersApi, PermissionsApi, AnalyticsApi, etc.) built on `UnifiedApiClient`
- `auth/` - Web3 auth client, SIWE challenge/verify, cookie management, middleware factory
- `components/` - Shared React components (buttons, cards, forms, modals, navigation)
- `config/` - Auth config, IAM permissions (`"platform:resource:action"`), feature flags, route constants
- `hooks/` - useApiClient, useSmartPolling, useNotificationBell, useSSENotifications, useWatchlist
- `state/` - React Query (TanStack Query v5) provider, platform-aware query client factory
- `types/` - All TypeScript interfaces (ApiResponse, auth types, domain models)
- `utils/` - API client core, response handlers, formatting (currency/date/display), logging

### Import Pattern
Both frontends import shared via tsconfig path alias:
```typescript
import { SomeComponent } from '@/shared/components';
import type { ApiResponse } from '@/shared/types';
import { UsersApi } from '@/shared/api';
```

### Development Guidelines
- **Server Components first**: Default to RSC over Client Components
- **Server Actions for data fetching**: Use `'use server'` actions, avoid client-side `fetch`/SWR
- **Client-side only exceptions**: Payment processing, smart contracts, Web3 wallet (WAGMI/RainbowKit)
- **Optimistic updates**: Use `useTransition` + Context providers for instant UI feedback (see watchlist pattern)
- **Check `shared/` before creating**: Always search for existing components/hooks/utils first

### Authentication
Web3-first (SIWE - Sign-In with Ethereum). No email/password. Strict `Authorization: Bearer <token>` for all API calls.
- Multi-chain: BSC Mainnet (56) & Testnet (97)
- Session: HttpOnly cookies with JWT. All cookies use `epsx.` prefix
- Permissions: IAM format `"platform:resource:action"` (e.g., `"admin:users:manage"`)
- Platforms: `epsx:*`, `admin:*`, `epsx-pay:*`, `epsx-token:*`
- Temporal permissions: `"platform:resource:action:unix_timestamp"` for expiring access

### API Routes
All endpoints use `/api/` prefix. Route constants in `shared/config/route-constants.ts`.
- `/api/public/*` - No auth required
- `/api/auth/*` - Web3 SIWE authentication
- `/api/users/*` - User management + watchlist
- `/api/analytics/*` - Market data + filter options
- `/api/admin/*` - Admin endpoints (permission required)
- `/api/permissions/*` - Permission authority
- `/api/plans/*` - Subscription management (features, categories, groups)

```typescript
API_ROUTES.AUTH.WEB3_CHALLENGE     // '/api/auth/web3/challenge'
API_ROUTES.ANALYTICS.RANKINGS      // '/api/analytics/rankings'
API_ROUTES.USERS.PROFILE           // '/api/users/profile'
API_ROUTES.USERS.WATCHLIST         // '/api/users/watchlist' (GET/POST/DELETE)
```

### Frontend Routes
- `/analytics` - Stock rankings & analytics dashboard
- `/portfolio` - User watchlist & portfolio tracking
- `/plans` - Subscription plans
- `/contact` - Contact & support
- `/dashboard` - User dashboard

### Infrastructure
- **Host**: Local Mac Mini (arm64) via Docker Compose + Cloudflare Tunnel
- **DB**: PostgreSQL (`epsx_prod`, `epsx_dev`), Redis (DB 0: Dev, DB 1: Prod)
- **Prod**: epsx.io / admin.epsx.io / api.epsx.io
- **Dev**: dev.epsx.io / dev-admin.epsx.io / dev-api.epsx.io

## ESLint Configuration

Ultra-strict config in `shared/config/eslint.cjs`. Key rules enforced as errors:
- `strict-boolean-expressions` - No implicit truthiness checks on nullable values
- `no-floating-promises` - All promises must be awaited or voided
- `no-unsafe-*` / `no-explicit-any` - No `any` types
- `no-console` - No console.log/error
- `max-lines-per-function: 120` - Use `// eslint-disable-next-line` for complex files
- `complexity: 12` - Extract logic into hooks/helpers
- `max-params: 3` - Use context objects for more params
- `no-misused-promises` - Correct async handler patterns
- `require-await` - Don't mark functions async unless they await

### Common ESLint Fix Patterns
- `||` → `??` for nullish coalescing
- Prefix unused vars with `_`
- Wrap fire-and-forget promises with `void`
- Use `useCallback` with complete dependency arrays
- Extract hooks for business logic, components for UI sections
