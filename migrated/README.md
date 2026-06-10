# EPSX — Web3 Commerce Platform

EPSX is a production-grade web3 commerce platform: visual page builder, on-chain payments, programmable subscriptions, and paymaster-sponsored gas — all running as Rust microservices on BSC.

This repository contains the migrated monorepo, replacing the original Vercel Next.js + monolithic Axum setup with a clean Dioxus-style microservice architecture.

## Highlights

- **9 Rust services + 4 BFFs** (Axum + SQLx + Alloy)
- **8 shared crates** for kernel, auth, web3, observability, events, client, renderer, config
- **Solidity contracts** for PaymentEscrow, SubscriptionVault, Paymaster, TokenRegistry
- **8 block types** for the visual builder (hero, features, pricing, testimonial, cta-banner, blog-list, custom-html, rich-text)
- **Dynamic block registry** with hot-reload from `content/blocks/*/manifest.json`
- **Git-tracked MDX content** with automatic theme + nav sync
- **Local-first monitoring** (Prometheus, Loki, Tempo, Grafana)
- **Cloudflare Tunnel** integration for zero-trust exposure

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  BFF Layer (SSR-rendered HTML, plain axum + tailwind CDN)   │
│  ┌───────────┐ ┌──────────┐ ┌──────┐ ┌──────────┐          │
│  │ frontend  │ │  admin   │ │ pay  │ │ preview  │          │
│  │  :3000    │ │  :3001   │ │:3002 │ │  :3003   │          │
│  └─────┬─────┘ └────┬─────┘ └──┬───┘ └────┬─────┘          │
│        └────────────┴──────────┴──────────┘                 │
│                  ▼                                            │
│            ┌───────────┐                                     │
│            │  gateway  │  JWT validation, RBAC, routing     │
│            │   :8080   │                                     │
│            └─────┬─────┘                                     │
└──────────────────┼──────────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  Service Layer (Rust microservices, axum + sqlx)            │
│  ┌─────────┐ ┌────────┐ ┌────────┐ ┌────────────┐          │
│  │identity │ │ wallet │ │payment │ │subscription│          │
│  │  :8101  │ │  :8102 │ │  :8103 │ │   :8104    │          │
│  └─────────┘ └────────┘ └────────┘ └────────────┘          │
│  ┌─────────┐ ┌────────────┐ ┌─────────┐                   │
│  │ content │ │notification│ │analytics│ ┌────────┐         │
│  │  :8105  │ │   :8106    │ │  :8107  │ │indexer │         │
│  │         │ │            │ │         │ │ :8108  │         │
│  └─────────┘ └────────────┘ └─────────┘ └────────┘         │
└─────────────────────────────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  Data Layer                                                  │
│  PostgreSQL (8 DBs) · Redis (event streams) · BSC RPC       │
│  Foundry Contracts · MDX content in git · Tailwind CDN      │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Tech |
|------|------|
| Language | Rust 1.91 (edition 2021) |
| HTTP | axum 0.8, tower, tower-http |
| Async | tokio 1 |
| Web3 | alloy 1.0 (BSC) |
| Database | sqlx 0.8 (PostgreSQL 16) |
| Cache | redis 0.28 |
| Auth | jsonwebtoken, siwe 0.6, argon2 |
| Email | lettre 0.11 (SMTP) |
| Frontend | Plain axum SSR + tailwind CDN + vanilla JS (no Dioxus) |
| Contracts | Solidity + OpenZeppelin + Foundry |
| Monitoring | Prometheus + Loki + Tempo + Grafana |
| Deployment | Cloudflare Tunnel + Colima K8s |

## Repository Layout

```
epsx/
├── Cargo.toml                # Workspace root
├── proto/                    # Protobuf definitions
├── shared/                   # 8 shared crates
│   ├── kernel/              # Domain types (ChainId, Token, Money, Address)
│   ├── crypto/              # JWT, SIWE
│   ├── database/            # sqlx PgPool + Repository trait
│   ├── web3/                # Alloy BSC providers
│   ├── events/              # DomainEvent + Redis streams
│   ├── observability/       # tracing + Prometheus
│   ├── config/              # AppConfig loader
│   ├── client/              # ServiceClient for BFF→service calls
│   ├── auth/                # JWT middleware + AuthUser extractor
│   └── renderer/            # Block/Page rendering
├── services/                 # 9 microservices
│   ├── gateway/             # :8080 — JWT + service proxy
│   ├── identity/            # :8101 — SIWE auth, user CRUD
│   ├── wallet/              # :8102 — accounts, balance, signing
│   ├── payment/             # :8103 — intents, escrow
│   ├── subscription/        # :8104 — plans, vault config
│   ├── content/             # :8105 — pages, blocks, themes, file watcher
│   ├── notification/        # :8106 — templates, email/in-app
│   ├── analytics/           # :8107 — events, metrics
│   └── indexer/             # :8108 — BSC blocks, transactions
├── apps/                     # 4 BFFs
│   ├── frontend/            # :3000 — epsx.io with visual builder
│   ├── admin/               # :3001 — admin.epsx.io
│   ├── pay/                 # :3002 — pay.epsx.io
│   └── preview/             # :3003 — preview.epsx.io
├── contracts/                # Foundry project
│   ├── src/PaymentEscrow.sol
│   ├── src/SubscriptionVault.sol
│   ├── src/Paymaster.sol
│   ├── src/TokenRegistry.sol
│   ├── script/Deploy.s.sol
│   └── test/Contracts.t.sol
├── content/                  # Git-tracked MDX content
│   ├── blocks/{hero,features,...}/manifest.json + schema.json
│   ├── themes/{default,dark,light}.json
│   ├── navigation/main.json
│   ├── settings/{site,edit-mode}.json
│   ├── pages/*.mdx
│   └── tailwind.css
├── infrastructure/
│   ├── cloudflare/          # Tunnel configs
│   ├── monitoring/          # Prometheus, Loki, Tempo, Grafana
│   ├── scripts/             # start, stop, migrate, build, deploy
│   ├── migrations/          # SQL migrations
│   └── config.example.toml
├── .github/workflows/ci.yml
└── README.md
```

## Quick Start

### Prerequisites

- **Rust 1.91+** (`rustup install stable`)
- **PostgreSQL 16** (running on `:5432`, user `epsx`, password `epsx`)
- **Redis 7+** (running on `:6379`, password `epsx`)
- **Foundry** (for contracts: `curl -L https://foundry.paradigm.xyz | bash && foundryup`)
- **Cloudflared** (for tunnels: `brew install cloudflared`)

### Setup

```bash
# 1. Clone and enter
cd epsx

# 2. Set up databases
./infrastructure/scripts/db-migrate.sh apply

# 3. Build all binaries
cargo build --workspace --release

# 4. Install Foundry dependencies (for contracts)
cd contracts && forge install foundry-rs/forge-std --no-commit && forge install OpenZeppelin/openzeppelin-contracts --no-commit && cd ..

# 5. Start all services
./infrastructure/scripts/start.sh

# 6. Verify
./infrastructure/scripts/health.sh
```

Services come up on:
- Frontend: http://localhost:3000
- Admin: http://localhost:3001
- Pay: http://localhost:3002
- Preview: http://localhost:3003
- API Gateway: http://localhost:8080

### Test

```bash
cargo test --workspace          # Run all tests
cargo test -p epsx-renderer     # Renderer tests
cargo test -p epsx-kernel       # Kernel tests
```

### Lint & Format

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Service Reference

| Service | Port | DB | Purpose |
|---------|------|-----|---------|
| gateway | 8080 | — | JWT validation, RBAC, reverse proxy |
| identity | 8101 | epsx_identity | SIWE auth, user CRUD |
| wallet | 8102 | epsx_wallet | Accounts, balance, signing |
| payment | 8103 | epsx_payment | Payment intents, escrow |
| subscription | 8104 | epsx_subscription | Plans, vault config |
| content | 8105 | epsx_content | Pages, blocks, themes, file watcher |
| notification | 8106 | epsx_notification | Templates, email, in-app |
| analytics | 8107 | epsx_analytics | Events, metrics |
| indexer | 8108 | epsx_indexer | BSC blocks, transactions |

## API Reference

All services expose a JSON REST API. Authentication uses SIWE-signed JWT tokens (issued by `identity` service, validated by `gateway`).

### Authentication

```bash
# 1. Get nonce + message
curl -X POST http://localhost:8101/api/v1/identity/auth/nonce \
  -H "content-type: application/json" \
  -d '{"address":"0x..."}'

# 2. Sign with wallet (MetaMask)
# personal_sign(message, address)

# 3. Verify signature, get JWT
curl -X POST http://localhost:8101/api/v1/identity/auth/siwe \
  -H "content-type: application/json" \
  -d '{"message":"...","signature":"0x...","chain_id":"56"}'

# Response: { "access_token": "eyJ...", "user": {...} }
```

### Content

```bash
# List blocks
curl http://localhost:8105/api/v1/content/blocks

# Get page
curl http://localhost:8105/api/v1/content/pages/welcome

# Save page (Editor+ role)
curl -X PUT http://localhost:8105/api/v1/content/pages/welcome \
  -H "authorization: Bearer $JWT" \
  -d '{"title":"...","blocks_json":"[...]","seo_json":"{...}"}'

# Publish
curl -X POST http://localhost:8105/api/v1/content/pages/welcome/publish \
  -H "authorization: Bearer $JWT"
```

### Payments

```bash
# Create intent
curl -X POST http://localhost:8103/api/v1/payment/intents \
  -H "authorization: Bearer $JWT" \
  -d '{"payer":"0x...","payee":"0x...","amount":"1000000","token":"USDC"}'

# Confirm (after on-chain tx)
curl -X POST http://localhost:8103/api/v1/payment/intents/$ID/confirm \
  -H "authorization: Bearer $JWT" \
  -d '{"tx_hash":"0x..."}'

# Release escrow
curl -X POST http://localhost:8103/api/v1/payment/escrows/$ID/release \
  -H "authorization: Bearer $JWT"
```

## Content Model

### Pages

Pages are MDX files in `content/pages/*.mdx`. They are parsed on content-service startup and exposed via `/api/v1/content/pages/{slug}`.

### Blocks

Each block type lives in `content/blocks/<name>/` with two files:
- `schema.json` — JSON schema for validation
- `manifest.json` — metadata (version, category, defaultProps)

```json
// content/blocks/hero/manifest.json
{
  "name": "hero",
  "version": "1.0.0",
  "category": "marketing",
  "defaultProps": {
    "title": "Welcome",
    "subtitle": "Subtitle here",
    "ctaText": "Get Started",
    "ctaUrl": "/"
  }
}
```

### Themes

Themes in `content/themes/*.json` define CSS variables (colors, fonts, spacing, breakpoints, radius):

```json
// content/themes/dark.json
{
  "name": "dark",
  "variables": {
    "--bg": "#030712",
    "--text": "#f9fafb",
    "--primary": "#3b82f6"
  }
}
```

### RBAC

`content/settings/edit-mode.json` controls which roles can edit which resources:

```json
{
  "roles": {
    "admin": { "canEdit": true, "canPublish": true, "canDelete": true },
    "editor": { "canEdit": true, "canPublish": false, "canDelete": false },
    "content_manager": { "canEdit": true, "canPublish": true, "canDelete": true }
  }
}
```

## Smart Contracts

### PaymentEscrow

```solidity
function create(address token, address payee, uint256 amount, uint256 feeBps) external returns (uint256 escrowId);
function release(uint256 escrowId) external;
function refund(uint256 escrowId, string reason) external;
function dispute(uint256 escrowId, string reason) external;
function resolve(uint256 escrowId, bool toPayee) external;
```

Default fee: 0.3% (30 bps), capped at 10% (1000 bps).

### SubscriptionVault

```solidity
function createPlan(string name, uint256 amount, address token, uint256 periodSeconds) external returns (uint256 planId);
function subscribe(uint256 planId) external;
function cancel(uint256 subscriptionId) external;
function withdraw(uint256 planId) external;
```

Per-merchant isolated vault. Grace period: 7 days default.

### Paymaster (ERC-4337)

```solidity
function depositUSDC(uint256 amount) external;
function withdrawUSDC(uint256 amount, address to) external;
function validatePaymasterUserOp(UserOp calldata op, bytes32 hash, uint256 maxCost) external returns (bytes context, uint256 deadline);
function postOp(...) external;
```

Sponsor gas, charge USDC with configurable markup (default 1%).

### TokenRegistry

```solidity
function registerToken(uint256 chainId, address token, string symbol, uint8 decimals) external;
function resolve(uint256 chainId, address token) external view returns (TokenInfo memory);
```

### Deploy

```bash
cd contracts
forge build
forge test -vv
forge script script/Deploy.s.sol:Deploy --rpc-url $BSC_RPC_URL --broadcast --private-key $PRIVATE_KEY
```

## Visual Builder

The frontend BFF (`apps/frontend`) serves `/edit/{slug}` with a drag-and-drop visual builder:

- **Block palette** (left): 8 block types
- **Canvas** (center): drop zone with block reorder
- **Inspector** (right): property editor with auto-save
- **Hot-reload** of block registry when `content/blocks/*/manifest.json` changes
- **RBAC enforced** server-side via `auth` crate

Built with vanilla JS + Tailwind CDN (no Dioxus, no React). State is per-page; last-write-wins.

## Monitoring

The `infrastructure/monitoring/` directory contains:

- **prometheus.yml** — scrapes all 13 services
- **loki.yml** — log aggregation
- **tempo.yml** — distributed tracing
- **grafana.ini** — auto-provisioned datasources + custom EPSX dashboard

```bash
# Start monitoring stack
docker compose -f infrastructure/monitoring/docker-compose.yml up -d
# Grafana: http://localhost:3000
```

## Deployment (Production)

Production runs locally via **Colima Kubernetes** + **Cloudflare Tunnel**:

```bash
# 1. Build images
docker build -f apps/frontend/Dockerfile -t epsx-frontend:prod .
docker build -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .
docker build -f apps/backend/Dockerfile -t epsx-backend:prod .

# 2. Create secrets
./infrastructure/scripts/create-secrets.sh prod

# 3. Apply K8s manifests
kubectl apply -k infrastructure/kubernetes/overlays/prod

# 4. Rollout
kubectl rollout restart deployment -n epsx-prod
```

**CRITICAL: Never deploy to production unless explicitly instructed by the user.**

## Roles

| Role | Edit | Publish | Delete | Manage Payments |
|------|------|---------|--------|-----------------|
| admin | ✓ | ✓ | ✓ | ✓ |
| content_manager | ✓ | ✓ | ✓ | — |
| editor | ✓ | — | — | — |
| designer | ✓ | — | — | — |
| merchant | — | — | — | ✓ |
| user | — | — | — | — |

## Development Workflow

1. **Edit content** in `content/pages/*.mdx` or `content/blocks/*/manifest.json` → file watcher picks it up
2. **Edit theme** in `content/themes/*.json` → hot-reloaded
3. **Add a service**:
   - `cargo new --bin services/<name>`
   - Add to `Cargo.toml` `[workspace] members`
   - Add to gateway proxy routes
4. **Add a block**:
   - Create `content/blocks/<name>/{manifest,schema}.json`
   - Add renderer in `shared/renderer/src/lib.rs`
   - Add BFF component in `apps/preview/src/main.rs`
5. **Run tests**: `cargo test --workspace`
6. **Commit**: `git commit -am "feat: ..."` (publishing to `main` triggers K8s rollout)

## Security

- **JWT** validation in gateway only (services trust gateway)
- **SIWE** for wallet auth (EIP-4361)
- **RBAC** enforced server-side
- **No secrets** in code; all loaded from environment / K8s secrets
- **CSP headers** in BFFs
- **Rate limiting** via tower middleware
- **Input validation** via JSON schemas

## License

Proprietary. All rights reserved.
