# EPSX Analytics Platform

EPSX is a high-performance production analytics platform built with a modern tech stack, designed for scalability and speed. It features a monorepo architecture managing a Next.js frontend, an admin dashboard, and a high-throughput Rust backend.

## 🚀 Quick Start

Ensure you have [Bun](https://bun.sh) and [Docker](https://www.docker.com/) installed.

```bash
# Install dependencies
bun install

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
- **DevOps**: Docker, Turbo Repo, Cloudflare Tunnel.

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

## 📦 Deployment

Deployment is handled via Docker containers and Cloud Run / Custom Server orchestration. See the `./scripts/deploy` directory for specific deployment strategies.

```bash
# Example: Build and deploy to development
bun deploy:dev
```
