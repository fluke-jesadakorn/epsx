# GEMINI.md

This file provides guidance to Gemini when working with this repository.

## Project Overview

EPSX is a production analytics platform:

| Service | Port | Stack |
|---------|------|-------|
| **Frontend** | 3000 | Next.js 15.5 + React 19.1 |
| **Admin Frontend** | 3001 | Next.js 15 + OIDC Auth |
| **Backend** | 8080 | Rust + Axum + Diesel + PostgreSQL |

## Quick Start

```bash
# Install & run all services
bun install && bun dev

# Individual services
bun dev:frontend     # Port 3000
bun dev:admin        # Port 3001
bun dev:backend      # Port 8080

# Build & test
bun build            # All applications
bun test             # All tests
bun lint && bun type-check && bun format  # QA
```

## File Structure

```
├── apps/
│   ├── frontend/           # Main analytics platform
│   ├── admin-frontend/     # Admin dashboard
│   └── backend/           # Rust API server
├── infrastructure/        # Docker & Cloudflare config
├── shared/                # Shared packages & config
├── packages/              # NPM packages
└── scripts/              # Build & deployment scripts
```

## Technology Stack

### Frontend
- **Framework**: Next.js 15 (App Router, Server Components)
- **UI**: Tailwind CSS
- **State**: Zustand + SWR
- **Forms**: React Hook Form + Zod
- **Web3**: WAGMI + RainbowKit

### Backend
- **Language**: Rust + Axum
- **Database**: PostgreSQL + Diesel (async)
- **Cache**: Redis
- **Architecture**: Clean architecture with repository pattern

## Authentication

### Web3-First System (SIWE)
- **Wallet-Only**: No email/password - Sign-In with Ethereum
- **Multi-Chain**: BSC Mainnet (56) and Testnet (97)
- **Session**: Web3 tokens stored in HttpOnly cookies
- **Flow**: Challenge → Sign → Verify → Bearer Token

### Permission System
```
Format: "platform:resource:action"
Examples: "admin:users:manage", "epsx:analytics:read"

Platforms: epsx:*, admin:*, epsx-pay:*, epsx-token:*
Temporal: "platform:resource:action:unix_timestamp" for expiring permissions
```

## API Structure

All endpoints use `/api/v1/` prefix. Routes defined in `/shared/config/route-constants.ts`.

| Category | Endpoint | Description |
|----------|----------|-------------|
| Public | `/api/v1/public/*` | No auth required |
| Auth | `/api/v1/auth/*` | Web3 SIWE authentication |
| Users | `/api/v1/users/*` | User management |
| Analytics | `/api/v1/analytics/*` | Market data |
| Admin | `/api/v1/admin/*` | Admin endpoints (permission required) |
| Permissions | `/api/v1/permissions/*` | Permission authority |
| Plans | `/api/v1/plans/*` | Subscription management |

**Route Constants Example:**
```typescript
API_ROUTES.AUTH.WEB3_CHALLENGE     // '/api/v1/auth/web3/challenge'
API_ROUTES.ANALYTICS.RANKINGS      // '/api/v1/analytics/rankings'
API_ROUTES.USERS.PROFILE           // '/api/v1/users/profile'
```

## Deployment

### Local Docker Build → Cloud Run

```bash
# Build all images locally
bun build:all

# Deploy with local testing (recommended)
bun deploy:frontend
bun deploy:admin
bun deploy:backend

# Or push only
bun deploy:all:prod

# Monitor
bun env:status

### Remote Server Deployment (Docker Compose)

For servers with limited RAM or network bandwidth (e.g., 100.109.131.15), use this "Build Local, Deploy Remote" strategy to avoid `bun install` or `cargo build` on the server.

**1. Build & Package Locally (Mac)**
```bash
# Build images for server architecture (linux/amd64)
export DOCKER_DEFAULT_PLATFORM=linux/amd64
docker compose -f infrastructure/docker/docker-compose.yml build frontend admin-frontend backend

# Save images to a compressed tarball
docker save epsx-frontend:latest epsx-admin-frontend:latest epsx-backend:latest | gzip > deploy.tar.gz
```

### Infrastructure Overview
- **Host**: Remote Server (100.109.131.15)
- **Containerization**: Docker Compose
- **Ingress**: Cloudflare Tunnel (Zero Trust)
- **Database**: Shared PostgreSQL instance (DBs: `epsx_prod`, `epsx_dev`)
- **Cache**: Shared Redis instance (DB 0: Dev, DB 1: Prod)

### Environment URLs
| Environment | Frontend | Admin | Backend |
|-------------|----------|-------|---------|
| **Production** | [epsx.io](https://epsx.io) | [admin.epsx.io](https://admin.epsx.io) | [api.epsx.io](https://api.epsx.io) |
| **Development** | [dev.epsx.io](https://dev.epsx.io) | [dev-admin.epsx.io](https://dev-admin.epsx.io) | [dev-api.epsx.io](https://dev-api.epsx.io) |

### Deployment Workflow (Docker Compose)

**1. Build & Package Locally (Mac)**
```bash
# Build images for server architecture (linux/amd64)
export DOCKER_DEFAULT_PLATFORM=linux/amd64

# Production Build (Baked-in Prod Env Vars)
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=04e0a500abfa1e095bf8f64b15fa2812 \
  --build-arg NEXT_PUBLIC_APP_URL=https://epsx.io \
  --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  --build-arg NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend \
  -f apps/frontend/Dockerfile -t epsx-frontend:prod .

docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=04e0a500abfa1e095bf8f64b15fa2812 \
  --build-arg NEXT_PUBLIC_APP_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  --build-arg NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin \
  -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .

# Backend (Reusable)
docker build -f apps/backend/Dockerfile -t epsx-backend:latest .

# Save images to a compressed tarball
docker save epsx-frontend:prod epsx-admin-frontend:prod epsx-backend:latest | gzip > deploy-prod.tar.gz
```

**2. Transfer to Server**
```bash
# Upload compressed images
scp deploy-prod.tar.gz USER@SERVER_IP:~/epsx/
```

**3. Deploy on Server**
```bash
ssh USER@SERVER_IP
cd ~/epsx

# Load images
gzip -d -c deploy-prod.tar.gz | docker load

# Deploy Production
cd prod
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d
```

**4. Running Migrations (Important)**
The Docker image currently does not contain the migration binary. Run migrations locally via SSH tunnel.
Note: Backend uses multiple databases. You must specify the config for each.

```bash
# 1. Open Tunnel to Remote DB
ssh -L 5434:localhost:5433 USER@SERVER_IP -Nf

# 2. Run Migration Locally (using diesel_cli)
# For Production:
export DATABASE_URL=postgres://epsx_user:password@localhost:5434/epsx_prod

# Core Migrations
diesel migration run --config apps/backend/diesel.toml

# Analytics Migrations
diesel migration run --config apps/backend/diesel_analytics.toml

# Notifications Migrations
diesel migration run --config apps/backend/diesel_notifications.toml

# Payments Migrations
diesel migration run --config apps/backend/diesel_payments.toml
```

## Environment Variables

### Server-Only (Essential 15 vars)
```bash
# Core Infrastructure
DATABASE_URL=postgresql://epsx_user:password@postgres:5432/epsx_prod
BACKEND_URL=https://api.epsx.io
FRONTEND_URL=https://epsx.io
ADMIN_FRONTEND_URL=https://admin.epsx.io

# Cloudflare Tunnel
CLOUDFLARE_TUNNEL_TOKEN=ey...

# Authentication
# ... (Same as before)
```

# ... (Original content preserved) ...

### Remote Server Deployment (Docker Compose)

**1. Build & Package Locally (Mac)**
```bash
# Build images for server architecture (linux/amd64)
export DOCKER_DEFAULT_PLATFORM=linux/amd64
docker compose -f docker-compose.yml build frontend admin-frontend backend

# Save images to a compressed tarball
docker save epsx-frontend:latest epsx-admin-frontend:latest epsx-backend:latest | gzip > deploy.tar.gz
```

**2. Transfer to Server**
```bash
# Upload compressed images
scp deploy.tar.gz USER@SERVER_IP:~/epsx/
```

**3. Deploy on Server**
```bash
ssh USER@SERVER_IP
cd ~/epsx

# Load images
gzip -d -c deploy.tar.gz | docker load

# Deploy Dev
docker compose --env-file .env.docker up -d

# Deploy Prod (Joined to same network)
cd prod
docker compose -f docker-compose.prod.yml --env-file .env.prod up -d
```

**4. Running Migrations (Important)**
The Docker image currently does not contain the migration binary. Run migrations locally via SSH tunnel:
```bash
# 1. Open Tunnel to Remote DB
ssh -L 5434:localhost:5433 USER@SERVER_IP -Nf

# 2. Run Migration Locally
export DATABASE_URL=postgres://epsx_user:password@localhost:5434/epsx_prod
diesel migration run
```

## Backend Commands
# ...

---

**Status**: Production Live on Docker Compose + Cloudflare Tunnel. Web3-first auth. Shared DB infrastructure for resource optimization.

