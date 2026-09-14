> Native production path (September 2026): Rust release binaries on the Mac Mini,
> PostgreSQL + Redis + MinIO, exposed through a named Cloudflare Tunnel. Follow
> `infrastructure/native/README.md` for build/package, explicit migrations,
> launchd, backups and rollback. Workers/D1 migration and container orchestration
> are not release prerequisites. Existing Kubernetes/Workers instructions below
> are historical rollback references. Never deploy or change production routes
> without a separate explicit deployment instruction.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Development (Cloudflare local — replaces Colima K8s)
- `cargo xtask cloudflare dev --local` - Full local Cloudflare (workerd/miniflare, `wrangler dev --local --persist-to=.wrangler/state`)
- `cargo xtask dev --all` - Legacy Colima K8s (deprecated, kept for rollback)
- `cargo xtask dev --frontend` / `--admin` / `--backend` - Individual Rust services
- `cargo xtask anvil-proxy` - Local Anvil chain and Rust RPC proxy (:8545)
- `cargo xtask setup-local` - Deploy contracts and tokens to the local chain

### Build
- `cargo xtask build --profile production` - All apps (Rust/Dioxus)
- `cargo xtask browser-runtime build` - WASM browser runtime (`wasm-bindgen 0.2.123`)
- `cargo xtask cloudflare build` - Wrangler-aware build (browser-runtime + production)

### Lint & Format
- `cargo fmt --all --check` - Rust format check
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo xtask audit no-node --strict` - Reject Node drift
- `cargo xtask assets verify` - Frozen CSS check

### Test
- `cargo xtask test --all` - Rust workspace tests
- `cargo test --workspace --locked` - Cargo tests
- `bunx wrangler dev --local --persist-to=.wrangler/state --config apps/frontend/wrangler.jsonc` - Local Cloudflare smoke (curl :8787/api/health)
- `cargo xtask e2e doctor|report|verify-artifacts` - E2E harness

### Backend (Rust)
- `cargo build` from `apps/backend/`
- `cargo test` from `apps/backend/`
- Binary: `apps/backend/src/bin/migrate.rs` for DB migrations
- Multiple sqlx migrations: `migrations/core`, `migrations/analytics`, `migrations/notifications`, `migrations/payments` (legacy `diesel.toml` deleted; run via `sqlx migrate run`)
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

docker build -f apps/frontend/Dockerfile -t epsx-frontend:prod .

docker build -f apps/admin/Dockerfile -t epsx-admin:prod .

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
