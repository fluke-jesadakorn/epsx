# EPSX Trading Platform - Developer Context

## Project Overview

This is a comprehensive trading platform monorepo built with modern technologies, featuring:
- A Next.js frontend ecosystem (trading platform on port 3000, admin dashboard on port 3001)
- A high-performance Rust backend API server (Axum framework)
- A suite of shared packages for UI components, types, and utilities

The project uses Turborepo for build orchestration and pnpm for package management.

## Architecture

### Applications (`apps/`)

1.  **Frontend** (`apps/frontend`): Main trading platform interface (Port: 3000)
    -   Built with Next.js 15.4.5, React 19.1.0, and TypeScript.
    -   Features include real-time trading, payment processing, and performance analytics.
2.  **Admin Frontend** (`apps/admin-frontend`): Administrative dashboard (Port: 3001)
    -   Built with Next.js 15, for user management, IAM, role assignment, and system analytics.
3.  **Backend** (`apps/backend`): Rust API server
    -   Built with the Axum framework, PostgreSQL (via SQLx), and WebSocket support.
    -   Handles real-time data, authentication, trading logic, and payment processing.
    -   Includes CLI tools for admin operations.

### Shared Packages (`packages/`)

-   **@epsx/api-client**: Unified API client.
-   **@epsx/ui**: Shared UI components built with Radix UI and Tailwind CSS.
-   **@epsx/types**: Comprehensive TypeScript type definitions.
-   **@epsx/config**: Shared ESLint and TypeScript configurations.
-   **@epsx/theme**: Design system and theme provider.
-   **@epsx/auth-shared**: Cross-app authentication utilities.
-   **@epsx/server-actions**: Enhanced Next.js Server Actions.
-   **@epsx/server-providers**: Server-side providers.
-   **@epsx/shared-core**: Core shared logic.
-   **@epsx/shared-utils**: General utility functions.
-   **@epsx/firebase-analytics**: Firebase analytics integration.

## Key Technologies

-   **Frontend**: Next.js 15, React 19, TypeScript 5.8.3, Tailwind CSS 4, SWR, Zustand, React Hook Form, Zod.
-   **Backend**: Rust 2021, Axum, PostgreSQL, SQLx, tokio-cron-scheduler.
-   **Monorepo Tools**: Turborepo 2.5.5, pnpm 10.14.0.
-   **Code Quality**: ESLint 9.23.0, Prettier 3.6.2, TypeScript.
-   **Testing**: Jest (unit), Playwright (E2E).
-   **Containerization**: Docker.

## Development Workflow

### Prerequisites

-   Node.js >= 18.0.0
-   pnpm >= 8.0.0
-   Rust (latest stable)
-   PostgreSQL

### Setup

1.  Clone the repository.
2.  Run `pnpm install` to install dependencies.
3.  Copy `.env.example` to `.env.development` and configure your environment variables.

### Common Development Commands

-   **Start Development**: `pnpm dev` (starts frontend and admin frontend). Use `pnpm dev:all` to include the backend.
-   **Build**: `pnpm build` (builds everything). Use `pnpm build:packages` first if working on packages.
-   **Linting**: `pnpm lint`
-   **Type Checking**: `pnpm type-check`
-   **Testing**: `pnpm test` (runs all tests), `pnpm test:unit`, `pnpm test:e2e`.
-   **Formatting**: `pnpm format`

### Package Development

Shared packages are built with Turborepo. When modifying packages, you often need to rebuild them:
`pnpm build:packages` or `pnpm dev:packages` (watch mode).

### Docker Development

Use `make` commands for Docker workflows (e.g., `make dev`, `make docker-up ENV=dev`).

## Environment Variables

The project uses several `.env` files:
-   Root `.env.development` for general settings.
-   App-specific `.env.local` files within `apps/frontend` and `apps/admin-frontend`.
-   A `.env` file within `apps/backend`.

## Testing Strategy

-   **Frontend**: Unit tests with Jest, E2E tests with Playwright.
-   **Backend**: Unit, integration, and API tests organized by architectural layers (domain, application, infrastructure, presentation).
