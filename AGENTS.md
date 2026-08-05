# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Common Commands

### Development
- `cargo xtask dev --all` - All Rust/Dioxus services
- `cargo xtask dev --frontend` / `--admin` / `--backend` - Individual apps
- `cargo xtask anvil-proxy` - Local Anvil chain and Rust RPC proxy (:8545)
- `cargo xtask setup-local` - Deploy contracts and tokens to the local chain

### Build
- `cargo xtask build --profile development`
- `cargo xtask build --profile production`

### Lint & Format
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo xtask audit no-node --strict`
- `cargo xtask assets verify`

### Test
- `cargo xtask test --all`
- `cargo test --workspace --locked`
- `cargo xtask e2e doctor|report|verify-artifacts`
- `cargo xtask e2e run --group 0 --browser chromium`

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

docker build -f apps/frontend/Dockerfile -t epsx-frontend:prod .
docker build -f apps/admin/Dockerfile -t epsx-admin:prod .
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
All business logic related to permissions, plan access, ranking offsets, feature flags, and subscription rules **must be implemented in the Rust backend only**. Frontend (`apps/frontend`) and admin (`apps/admin`) are UI-only layers.

## Architecture

### Monorepo Structure
Cargo workspace with Dioxus applications, Axum microservices, shared Rust crates, and Foundry contracts.

### Infrastructure
- **Host**: Local Mac Mini (arm64) via **Colima Kubernetes** + Cloudflare Tunnel
- **DB**: PostgreSQL (`epsx_prod`, `epsx_analytics_prod`, etc.), Redis (Password: `epsx`)
- **Prod Domains**: epsx.io / admin.epsx.io / api.epsx.io
