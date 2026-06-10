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

### Deployment (Colima K8s + Cloudflare Tunnel)
**CRITICAL: Never deploy to production unless explicitly instructed by the user. Making code changes locally is always safe; deploying to prod requires explicit user confirmation each time.**

Production runs locally via **Colima Kubernetes** (profile `epsx`) with Cloudflare Tunnel exposing services via NodePorts and `socat` bridges.

**Quick deploy (restart with existing images):**
```bash
kubectl apply -k infrastructure/kubernetes/overlays/prod
kubectl rollout restart deployment -n epsx-prod
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
  -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .

# Build backend
docker build -f apps/backend/Dockerfile -t epsx-backend:prod .

# Create/update K8s secrets
./infrastructure/kubernetes/scripts/create-secrets.sh prod

# Deploy to K8s
kubectl apply -k infrastructure/kubernetes/overlays/prod
```

**Networking & Bridging:**
Cloudflare Tunnel is remotely managed and expects services on ports 4700, 4701, and 9180. To bridge these to Kubernetes NodePorts, use the `com.epsx.port-bridge` LaunchAgent:
```bash
cp infrastructure/scripts/com.epsx.port-bridge.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.epsx.port-bridge.plist
```

**Services & ports:**
| Service | K8s Deployment | NodePort | Legacy Bridge |
|---------|---------------|----------|---------------|
| Frontend | epsx-frontend | 30000 | 4700 |
| Admin | epsx-admin | 30001 | 4701 |
| Backend | epsx-backend | 30080 | 9180 |
| PostgreSQL | bare metal (brew) | 5432 | — |
| Redis | bare metal (brew) | 6379 | — |
| MinIO | bare metal (launchctl) | 9100 | — |

**Database Setup (Host):**
- **PostgreSQL**: Must set `listen_addresses = '*'` in `postgresql.conf` and allow `192.168.0.0/16` in `pg_hba.conf`.
- **Redis**: Port 6379, password `epsx`.
- **K8s Access**: Pods reach host via `host.docker.internal` (aliased to `192.168.5.1` via `hostAliases` in deployments).

**Session Persistence:**
Persistent RSA keys are mounted into the backend pod via secret `epsx-backend-keys` from `.env.prod`. Do not let the backend generate new keys on restart or sessions will expire.

**Troubleshooting:**
```bash
# Check pod status
kubectl get pods -n epsx-prod

# Check pod logs
kubectl logs -n epsx-prod deployment/epsx-backend

# Check socat bridges
ps aux | grep socat
```

## Architecture Constraints

### Permissions & Plan Logic — Backend Only
All business logic related to permissions, plan access, ranking offsets, feature flags, and subscription rules **must be implemented in the Rust backend only**. Frontend (`apps/frontend`) and admin-frontend (`apps/admin-frontend`) are UI-only layers.

## Architecture

### Monorepo Structure
Bun workspaces + Turborepo. Four apps share code via `shared/`.

### Infrastructure
- **Host**: Local Mac Mini (arm64) via **Colima Kubernetes** + Cloudflare Tunnel
- **DB**: PostgreSQL (`epsx_prod`, `epsx_analytics_prod`, etc.), Redis (Password: `epsx`)
- **Prod Domains**: epsx.io / admin.epsx.io / api.epsx.io
