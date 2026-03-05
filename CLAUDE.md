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
**CRITICAL: Never deploy to production unless explicitly instructed by the user. Making code changes locally is always safe; deploying to prod requires explicit user confirmation each time.**

Production runs locally via Docker Compose with Cloudflare Tunnel exposing services.

**Quick deploy (restart with existing images):**
```bash
cd infrastructure/docker
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --force-recreate
```

**Full rebuild & deploy:**
```bash
# Source all env vars from .env.prod (single source of truth)
set -a && source infrastructure/docker/.env.prod && set +a
export DOCKER_DEFAULT_PLATFORM=$DOCKER_PLATFORM

# Build frontend
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WALLETCONNECT_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=$FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL \
  --build-arg NEXT_PUBLIC_ADMIN_URL=$ADMIN_FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK \
  --build-arg NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=$OAUTH_CLIENT_ID \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=$NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=$NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET \
  --build-arg NEXT_PUBLIC_TURNSTILE_SITE_KEY=$NEXT_PUBLIC_TURNSTILE_SITE_KEY \
  -f apps/frontend/Dockerfile -t epsx-frontend:prod .

# Build admin frontend
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WALLETCONNECT_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=$ADMIN_FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL \
  --build-arg NEXT_PUBLIC_ADMIN_URL=$ADMIN_FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK \
  --build-arg NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=$NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=$NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET \
  --build-arg NEXT_PUBLIC_TURNSTILE_SITE_KEY=$NEXT_PUBLIC_TURNSTILE_SITE_KEY \
  -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .

# Build backend
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

**Troubleshooting - 502 / "Unable to reach origin service":**

Cloudflared uses Docker service names (`backend`, `frontend`, `admin-frontend`) for routing. When a container is restarted manually (outside docker compose), its service name DNS alias may not be re-registered in the Docker network. Diagnose:
```bash
docker run --rm --network epsx_prod_network alpine nslookup backend
# If NXDOMAIN → DNS alias is missing
```
Fix without full redeploy (briefly disconnects backend from network):
```bash
docker network disconnect epsx_prod_network epsx-prod-backend
docker network connect --alias backend epsx_prod_network epsx-prod-backend
docker restart epsx-prod-cloudflared
```
Fix via full redeploy (preferred):
```bash
cd infrastructure/docker
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --force-recreate
```
**Rule:** Always use `docker compose up --force-recreate` to restart containers, never `docker restart <container>` alone.

## Architecture Constraints

### Permissions & Plan Logic — Backend Only
All business logic related to permissions, plan access, ranking offsets, feature flags, and subscription rules **must be implemented in the Rust backend only**. Frontend (`apps/frontend`) and admin-frontend (`apps/admin-frontend`) are UI-only layers:
- **Never** compute or derive permission values in frontend/admin code
- **Never** parse or transform permission strings (e.g., `epsx:rankings:offset:N`) in frontend
- **Never** calculate ranking offsets, access levels, or plan features client-side
- Frontends only display data returned by backend APIs and pass user inputs as-is

The backend is the single source of truth for all permission derivations.

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

## Plan Seed Data (Manual DB Insert Fallback)

If a migration fails and plans are missing, run this SQL against the `epsx_prod` (or `epsx_dev`) database:

```sql
-- ============================================================
-- EPSX SUBSCRIPTION PLAN SEED (manual fallback)
-- Run: psql -h localhost -p 5491 -U <user> -d epsx_prod -f seed.sql
-- Update promotion end_date values before running.
-- ============================================================

-- Step 1: Ensure all required permissions exist
INSERT INTO permissions (permission_string, platform, resource, action, permission_type, is_system, is_active)
VALUES
  ('epsx:analytics:view',      'epsx', 'analytics', 'view',      'manual', true, true),
  ('epsx:analytics:advanced',  'epsx', 'analytics', 'advanced',  'manual', true, true),
  ('epsx:trading:basic',       'epsx', 'trading',   'basic',     'manual', true, true),
  ('epsx:trading:pro',         'epsx', 'trading',   'pro',       'manual', true, true),
  ('epsx:trading:advanced',    'epsx', 'trading',   'advanced',  'manual', true, true),
  ('epsx:api:read',            'epsx', 'api',       'read',      'manual', true, true),
  ('epsx:api:write',           'epsx', 'api',       'write',     'manual', true, true),
  ('epsx:data:export',         'epsx', 'data',      'export',    'manual', true, true),
  ('epsx:notifications:manage','epsx', 'notifications','manage', 'manual', true, true),
  ('epsx:alerts:create',       'epsx', 'alerts',    'create',    'manual', true, true)
ON CONFLICT (permission_string) DO NOTHING;

-- Step 2: Upsert plans (ON CONFLICT (slug) DO UPDATE)
INSERT INTO plans (
  name, slug, description, plan_type, plan_metadata,
  price, currency, billing_cycle,
  is_active, is_promoted, is_public, is_system,
  plan_category, plan_group, tier_level, grace_period_hours,
  display_order,
  rate_limit_per_minute, rate_limit_per_hour, rate_limit_per_day, burst_capacity,
  created_by
) VALUES

-- 1. ONE DAY PLAN (Personal, trial)
(
  'One Day Plan', 'one-day',
  '24-hour trial access to explore the platform',
  'subscription',
  '{
    "features": [
      "Basic analytics view",
      "Rankings from position 6+",
      "Basic trading features",
      "24-hour trial access",
      "Explore the platform"
    ],
    "ranking_offset": 5,
    "rankings_limit": 5,
    "promotion": {
      "enabled": true,
      "type": "percentage",
      "value": 80.0,
      "price": 1.0,
      "start_date": "",
      "end_date": "2026-03-25T14:00:00Z"
    }
  }'::jsonb,
  5.00, 'USD', 'one_time',
  true, false, true, false,
  'base', 'personal', 0, 0,
  1, 60, 1000, 10000, 10,
  '0x0000000000000000000000000000000000000000'
),

-- 2. STARTER PLAN (Personal, 30-day)
(
  'Starter Plan', 'starter',
  'Advanced analytics for individual investors and traders',
  'subscription',
  '{
    "features": [
      "Advanced analytics view",
      "25 stock rankings",
      "Basic Analytic features",
      "Price alerts",
      "Email support",
      "30-day access"
    ],
    "ranking_offset": 1,
    "rankings_limit": 25,
    "promotion": {
      "enabled": true,
      "type": "percentage",
      "value": 90.0,
      "price": 9.9,
      "start_date": "",
      "end_date": "2026-03-25T14:00:00Z"
    }
  }'::jsonb,
  99.00, 'USD', 'one_time',
  true, false, true, false,
  'base', 'personal', 1, 0,
  2, 120, 3000, 50000, 20,
  '0x0000000000000000000000000000000000000000'
),

-- 3. LIFE TIME (Personal, lifetime)
(
  'Life Time', 'lifetime',
  'Full platform access with lifetime membership',
  'subscription',
  '{
    "features": [
      "Advanced analytics suite",
      "Full rankings access (Rank 1+)",
      "API read access",
      "Basic & Pro trading",
      "Priority support",
      "Lifetime access"
    ],
    "ranking_offset": 0,
    "rankings_limit": -1,
    "promotion": {
      "enabled": true,
      "type": "percentage",
      "value": 50.0,
      "price": 4999.0,
      "start_date": "",
      "end_date": "2026-03-25T14:00:00Z"
    }
  }'::jsonb,
  9999.00, 'USD', 'lifetime',
  true, true, true, false,
  'base', 'personal', 3, 0,
  3, 300, 10000, 200000, 50,
  '0x0000000000000000000000000000000000000000'
),

-- 4. COMPANY PLAN (Enterprise, 365-day)
(
  'Company Plan', 'company',
  'Complete solutions for professional teams and institutions',
  'subscription',
  '{
    "features": [
      "Advanced analytics suite",
      "Full trading suite (Basic, Pro & Advanced)",
      "API read & write access",
      "Data export",
      "Notifications management",
      "365-day corporate access",
      "Dedicated support"
    ],
    "ranking_offset": 0,
    "rankings_limit": -1,
    "promotion": {
      "enabled": true,
      "type": "percentage",
      "value": 57.0,
      "price": 2999.0,
      "start_date": "",
      "end_date": "2026-04-04T05:00:00Z"
    }
  }'::jsonb,
  6999.00, 'USD', 'one_time',
  true, false, true, false,
  'base', 'enterprise', 4, 0,
  4, 1000, 50000, 1000000, 200,
  '0x0000000000000000000000000000000000000000'
),

-- 5. API PERSONAL (API, 30-day)
(
  'API Personal', 'api-personal',
  'Integrate our powerful API into your systems',
  'subscription',
  '{
    "features": [
      "Analytics view access",
      "API read access",
      "Data export capability",
      "Full developer documentation",
      "30-day access"
    ],
    "ranking_offset": 1,
    "rankings_limit": -1,
    "promotion": {
      "enabled": true,
      "type": "percentage",
      "value": 75.0,
      "price": 999.0,
      "start_date": "",
      "end_date": "2026-03-25T14:00:00Z"
    }
  }'::jsonb,
  3999.00, 'USD', 'one_time',
  true, false, true, false,
  'base', 'api', 2, 0,
  5, 300, 10000, 100000, 50,
  '0x0000000000000000000000000000000000000000'
),

-- 6. CUSTOM (Custom, revenue share)
(
  'Custom', 'custom',
  'Tailored solutions for partners, corporate, and enterprise needs',
  'manual',
  '{
    "features": [
      "Custom feature set & permissions",
      "Dedicated support & SLA",
      "Volume-based pricing",
      "Custom API rate limits",
      "White-label options",
      "Priority onboarding"
    ],
    "contact_sales": true
  }'::jsonb,
  0.00, 'USD', 'pay_per_use',
  true, false, true, false,
  'exclusive', 'custom', 5, 0,
  6, 1000, 50000, 1000000, 200,
  '0x0000000000000000000000000000000000000000'
)

ON CONFLICT (slug) DO UPDATE SET
  name            = EXCLUDED.name,
  description     = EXCLUDED.description,
  plan_type       = EXCLUDED.plan_type,
  plan_metadata   = EXCLUDED.plan_metadata,
  price           = EXCLUDED.price,
  currency        = EXCLUDED.currency,
  billing_cycle   = EXCLUDED.billing_cycle,
  is_active       = EXCLUDED.is_active,
  is_promoted     = EXCLUDED.is_promoted,
  plan_category   = EXCLUDED.plan_category,
  plan_group      = EXCLUDED.plan_group,
  tier_level      = EXCLUDED.tier_level,
  display_order   = EXCLUDED.display_order,
  rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
  rate_limit_per_hour   = EXCLUDED.rate_limit_per_hour,
  rate_limit_per_day    = EXCLUDED.rate_limit_per_day,
  burst_capacity        = EXCLUDED.burst_capacity,
  updated_at      = NOW();

-- Step 3: Link plan permissions
DO $$
DECLARE
  p_one_day    UUID;
  p_starter    UUID;
  p_lifetime   UUID;
  p_company    UUID;
  p_api_pers   UUID;
BEGIN
  SELECT id INTO p_one_day  FROM plans WHERE slug = 'one-day';
  SELECT id INTO p_starter  FROM plans WHERE slug = 'starter';
  SELECT id INTO p_lifetime FROM plans WHERE slug = 'lifetime';
  SELECT id INTO p_company  FROM plans WHERE slug = 'company';
  SELECT id INTO p_api_pers FROM plans WHERE slug = 'api-personal';

  -- ONE DAY PLAN: analytics:view, trading:basic
  IF p_one_day IS NOT NULL THEN
    INSERT INTO plan_permissions (plan_id, permission_id)
    SELECT p_one_day, id FROM permissions
    WHERE permission_string IN ('epsx:analytics:view', 'epsx:trading:basic')
    ON CONFLICT (plan_id, permission_id) DO NOTHING;
  END IF;

  -- STARTER PLAN: analytics:view/advanced, trading:basic, alerts:create
  IF p_starter IS NOT NULL THEN
    INSERT INTO plan_permissions (plan_id, permission_id)
    SELECT p_starter, id FROM permissions
    WHERE permission_string IN (
      'epsx:analytics:view', 'epsx:analytics:advanced',
      'epsx:trading:basic', 'epsx:alerts:create'
    )
    ON CONFLICT (plan_id, permission_id) DO NOTHING;
  END IF;

  -- LIFE TIME: analytics:view/advanced, trading:basic/pro, api:read
  IF p_lifetime IS NOT NULL THEN
    INSERT INTO plan_permissions (plan_id, permission_id)
    SELECT p_lifetime, id FROM permissions
    WHERE permission_string IN (
      'epsx:analytics:view', 'epsx:analytics:advanced',
      'epsx:trading:basic', 'epsx:trading:pro',
      'epsx:api:read'
    )
    ON CONFLICT (plan_id, permission_id) DO NOTHING;
  END IF;

  -- COMPANY PLAN: all trading, api read+write, data export, notifications
  IF p_company IS NOT NULL THEN
    INSERT INTO plan_permissions (plan_id, permission_id)
    SELECT p_company, id FROM permissions
    WHERE permission_string IN (
      'epsx:analytics:view', 'epsx:analytics:advanced',
      'epsx:trading:basic', 'epsx:trading:pro', 'epsx:trading:advanced',
      'epsx:api:read', 'epsx:api:write',
      'epsx:data:export', 'epsx:notifications:manage'
    )
    ON CONFLICT (plan_id, permission_id) DO NOTHING;
  END IF;

  -- API PERSONAL: analytics:view, api:read, data:export
  IF p_api_pers IS NOT NULL THEN
    INSERT INTO plan_permissions (plan_id, permission_id)
    SELECT p_api_pers, id FROM permissions
    WHERE permission_string IN (
      'epsx:analytics:view', 'epsx:api:read', 'epsx:data:export'
    )
    ON CONFLICT (plan_id, permission_id) DO NOTHING;
  END IF;
END $$;
```

**Notes:**
- `promotion.end_date` values above are estimates — update to actual sale end dates before running
- `promotion.price` = the sale price shown to users; `plans.price` = original/base price (shown as strikethrough)
- Custom plan has no fixed permissions (assigned manually per customer via admin)
- Run `SELECT name, price, plan_group, plan_category FROM plans ORDER BY display_order;` to verify after insert
