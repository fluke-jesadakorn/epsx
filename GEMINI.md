# EPSX - Project Overview & Engineering Guidelines

EPSX is a high-performance production analytics platform for stock and financial data. It uses a monorepo architecture managed by **Turbo** and **Bun**, integrating a high-throughput **Rust** backend with **Next.js** frontends and **Web3** authentication.

## 🏗 Architecture & Monorepo Structure

The project follows a modular monorepo structure where logic is centralized in `shared/` and business rules are strictly enforced in the backend.

- **`apps/frontend`**: Main analytics platform (Next.js 16, React 19).
- **`apps/admin-frontend`**: Internal management portal (Next.js 16, React 19).
- **`apps/backend`**: High-performance API (Rust, Axum, Diesel ORM, CQRS).
- **`apps/contracts`**: BSC Smart Contracts (Solidity, Foundry).
- **`shared/`**: Single source of truth for UI components, API clients, and domain types.
- **`infrastructure/`**: Kubernetes manifests (**Colima**), Docker, and Cloudflare Tunnel configs.

### Technical Stack
- **Frontend**: Next.js 16 (App Router), React 19, Tailwind CSS v4, Zustand, TanStack Query v5.
- **Backend**: Rust (Axum), Diesel ORM (Async), PostgreSQL, Redis (Streams for CQRS), SIWE.
- **Infrastructure**: **Colima Kubernetes (k3s)**, Cloudflare Tunnel, MinIO (Bare Metal).

## 🛠 Core Development Commands

### Environment Setup
- `bun env:setup`: Create `.env` files from examples.
- `bun env:validate`: Validate current environment variables.
- `bun setup:local`: Deploy contracts and tokens to a local Anvil chain.

### Execution
- `bun dev`: Run all services (Frontend :3000, Admin :3001, Backend :8080).
- `bun dev:web`: Run both frontends without the backend.
- `bun dev:anvil`: Start local Anvil chain (:8545).
- `bun dev:up`: Spin up infrastructure (Postgres, Redis) via Docker.

### Maintenance
- `bun lint`: ESLint all apps (Ultra-strict rules enforced).
- `bun type-check`: TypeScript checking across the monorepo.
- `bun format`: Prettier format all files.
- `bun test`: Run Jest and Cargo tests.
- `bun test:e2e`: Playwright end-to-end tests.

## ⚠️ Architectural Constraints & Rules

### Permissions & Plan Logic (CRITICAL)
- **Backend Only**: All business logic for permissions, plan access, ranking offsets, and feature flags **must be implemented in the Rust backend**.
- **UI-Only Frontends**: Frontends must never compute, derive, or parse permission strings. They only display data returned by the API.

### Backend (Rust/Axum) - Clean Architecture + CQRS
- **Domain-Driven Design (DDD)**: Logic split into `domain`, `application`, `infrastructure`, and `web`.
- **Session Persistence**: Uses persistent RSA keys (secret `epsx-backend-keys`) to maintain user sessions across restarts.
- **Dependency Injection**: Use `DomainContainer` passed to all routes.

### Infrastructure & Networking
- **Host Connectivity**: Pods reach host services (DB/Redis) via `host.docker.internal`, mapped to `192.168.5.1` via `hostAliases`.
- **Port Bridging**: `socat` bridges map legacy Docker ports (4700, 4701, 9180) to Kubernetes NodePorts (30000, 30001, 30080) for Cloudflare Tunnel compatibility.

## 🔑 Authentication & Security
- **Web3-First**: Sign-In with Ethereum (SIWE). No email/password.
- **Session**: Secure HttpOnly cookies with JWT (`epsx.` prefix).
- **IAM Permissions**: Format `"platform:resource:action"` (e.g., `"admin:users:manage"`).

## 🚀 Deployment (Colima K8s + Cloudflare Tunnel)

**CRITICAL**: Never deploy to production without explicit user confirmation.

Production runs on local **Colima Kubernetes (k3s)** with Cloudflare Tunnels for ingress.
- **Frontend NodePort**: 30000 (Legacy Bridge: 4700)
- **Admin NodePort**: 30001 (Legacy Bridge: 4701)
- **Backend NodePort**: 30080 (Legacy Bridge: 9180)

### Quick Redeploy
```bash
kubectl apply -k infrastructure/kubernetes/overlays/prod
kubectl rollout restart deployment -n epsx-prod
```

## 📂 Key Shared Modules (`shared/`)
- `api/`: Domain API clients built on `UnifiedApiClient`.
- `auth/`: Web3 auth, SIWE challenge/verify, cookie management.
- `config/`: Route constants, IAM permission strings, feature flags.
- `hooks/`: `useApiClient`, `useSSENotifications`, `useWatchlist`.
- `state/`: TanStack Query factory and providers.
- `utils/`: Unified logging, currency/date formatting, and response handlers.
