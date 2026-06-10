# EPSX Analytics Platform

EPSX is a high-performance production analytics platform built with a modern tech stack, designed for scalability and speed. It features a monorepo architecture managing a Next.js frontend, an admin dashboard, and a high-throughput Rust backend.

## 🚀 Quick Start

Ensure you have [Bun](https://bun.sh) and [Docker](https://www.docker.com/) installed.

```bash
# Install dependencies
bun install

# Configure local env stack
# Shared secrets/common defaults: .env or .env.local
# Environment overlays: .env.development / .env.staging / .env.production

# Start development environment (all services)
bun dev
```

### Services Access

| Service | Local URL | description |
|---------|-----------|-------------|
| **Frontend** | [http://localhost:3000](http://localhost:3000) | Main analytics dashboard (Next.js 16 + React 19) |
| **Admin** | [http://localhost:3001](http://localhost:3001) | Admin management portal (Next.js 16) |
| **Backend** | [http://localhost:8080](http://localhost:8080) | API Server (Rust + Axum) |

## 🏗 Architecture

The project is structured as a monorepo using **Turbo** and **Bun**.

```text
├── apps/
│   ├── frontend/        # Analytics platform (Next.js App Router)
│   ├── admin-frontend/  # Internal admin tool (Next.js App Router)
│   └── backend/         # High-performance API (Rust/Axum)
├── packages/            # Shared internal NPM packages
├── shared/              # Shared configuration and types
└── scripts/             # DevOps and utility scripts
```

### Technology Stack

- **Frontend**: Next.js 16.0, React 19.2, TailwindCSS, Zustand, SWR.
- **Web3**: Wagmi, RainbowKit (SIWE compatible).
- **Backend**: Rust (Axum), Diesel ORM (Async), PostgreSQL, Redis.
- **DevOps**: Vercel (frontend/admin), local Kubernetes + Cloudflare Tunnel (backend), Turbo Repo.

## 🔑 Authentication

The platform uses a Web3-first authentication system (Sign-In with Ethereum).

- **Wallet-Only**: No email/password required.
- **Multi-Chain**: Supports BSC Mainnet (56) and Testnet (97).
- **Session Management**: Secure HttpOnly cookies.

## 🛠 Development Commands

Defined in `package.json`, here are the most common commands:

- `bun dev`: Run all apps in parallel.
- `bun dev:frontend`: Run only the main frontend.
- `bun dev:admin`: Run only the admin frontend.
- `bun dev:backend`: Run only the backend API.
- `bun build`: Build all applications.
- `bun test`: Run test suite.
- `bun lint`: Lint all codebases.
- `bun format`: Format code with Prettier.

## 🌍 Environment Files

Root environment loading now follows a layered model:

- `.env`: shared secrets and legacy common values
- `.env.development`, `.env.staging`, `.env.production`: tracked non-secret overlays
- `.env.local`, `.env.development.local`, `.env.staging.local`, `.env.production.local`: untracked overrides

The app and backend scripts resolve the active stack automatically from `ENV`, `DEPLOYMENT_ENV`, `RUST_ENV`, or `NODE_ENV`.

## 📦 Deployment

The target deployment split is now:

- `apps/frontend` -> Vercel
- `apps/admin-frontend` -> Vercel
- `apps/backend` -> local Kubernetes
- `apps/contracts` -> local Foundry workflows and on-chain deployment

The repo migration note is in [docs/plans/2026-04-16-vercel-hybrid-deployment.md](/Users/fluke/Desktop/Work/epsx/docs/plans/2026-04-16-vercel-hybrid-deployment.md). Legacy Docker-based frontend deployment files still exist and should be treated as transitional until the Vercel projects are fully live.
