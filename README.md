# EPSX Monorepo 📈

A comprehensive trading platform built with modern technologies, featuring a Next.js frontend, admin dashboard, and Rust backend in a well-organized monorepo structure.

[![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

---

## 🏗️ Architecture

This monorepo is organized into three main categories:

### 🚀 Applications (`apps/`)

- **Frontend** (`@epsx/frontend`) - Main trading platform interface
- **Admin Frontend** (`@epsx/admin-frontend`) - Administrative dashboard
- **Backend** (`@epsx/backend`) - Rust-based API server

### 📦 Packages (`packages/`)

- **@epsx/types** - Shared TypeScript type definitions
- **@epsx/utils** - Common utility functions
- **@epsx/ui** - Reusable UI components
- **@epsx/config** - Shared configuration (ESLint, etc.)
- **@epsx/auth** - Authentication utilities
- **@epsx/api-client** - API client library
- **@epsx/shared** - Shared business logic

---

## 🛠️ Tech Stack

### Frontend Technologies

- **Framework:** Next.js 15 with App Router
- **Language:** TypeScript
- **Styling:** Tailwind CSS + Radix UI
- **State Management:** SWR + Zustand
- **Authentication:** Firebase Auth
- **Testing:** Jest + Playwright

### Backend Technologies

- **Language:** Rust
- **Framework:** [Your Rust web framework]
- **Database:** [Your database choice]

### Development Tools

- **Monorepo:** Turborepo for build orchestration
- **Package Manager:** PNPM with workspaces
- **Code Quality:** ESLint + Prettier
- **Containerization:** Docker + Docker Compose

---

## 🚀 Quick Start

### Prerequisites

- **Node.js** >= 18.0.0
- **PNPM** >= 8.0.0
- **Rust** (for backend development)
- **Docker** (optional, for containerized development)

### Installation

```bash
# Clone the repository
git clone https://github.com/fluke-jesadakorn/epsx.git
cd epsx

# Install dependencies
pnpm install

# Setup environment variables
cp .env.example .env
# Edit .env with your configuration

# Start development servers
pnpm dev
```

---

## 📜 Available Scripts

### Development

```bash
# Start all development servers (frontend + admin)
pnpm dev

# Start specific applications
pnpm dev:frontend      # Frontend only (port 3000)
pnpm dev:admin         # Admin only (port 3001)
pnpm dev:backend       # Backend only
pnpm dev:packages      # Watch mode for packages

# Start all including backend
pnpm dev:all
```

### Building

```bash
# Build everything
pnpm build

# Build specific targets
pnpm build:packages    # Build all packages
pnpm build:apps        # Build all applications
pnpm build:frontend    # Build frontend app
pnpm build:admin       # Build admin app
pnpm build:backend     # Build backend
```

### Quality Assurance

```bash
# Linting
pnpm lint              # Check all projects
pnpm lint:fix          # Fix auto-fixable issues

# Type checking
pnpm type-check        # Check all projects
pnpm type-check:packages
pnpm type-check:apps

# Testing
pnpm test              # Run all tests
pnpm test:unit         # Unit tests only
pnpm test:e2e          # E2E tests only

# Formatting
pnpm format            # Format all files
pnpm format:check      # Check formatting
```

### Utilities

```bash
# Cleanup
pnpm clean             # Clean build artifacts
pnpm clean:cache       # Clean all caches

# Docker
pnpm docker:dev        # Start development environment
pnpm docker:prod       # Start production environment
pnpm docker:dev:down   # Stop development environment

# Workspace management
pnpm workspace:graph   # Show dependency graph
pnpm workspace:outdated # Check outdated dependencies
```

---

## 🐳 Docker Development

```bash
# Using Docker Compose
make docker-up ENV=dev    # Start development environment
make docker-down ENV=dev  # Stop development environment
make docker-logs ENV=dev  # View logs

# Using PNPM scripts
pnpm docker:dev           # Start development containers
pnpm docker:dev:down      # Stop development containers
pnpm docker:dev:logs      # View development logs
```

---

## 📁 Project Structure

```
epsx/
├── apps/
│   ├── frontend/          # Main trading platform
│   ├── admin-frontend/    # Admin dashboard
│   └── backend/           # Rust API server
├── packages/
│   ├── types/             # Shared TypeScript types
│   ├── utils/             # Utility functions
│   ├── ui/                # UI components
│   ├── config/            # Configuration packages
│   ├── auth/              # Authentication utilities
│   ├── api-client/        # API client
│   └── shared/            # Shared business logic
├── docs/                  # Documentation
├── scripts/               # Build and deployment scripts
└── config files...        # Monorepo configuration
```

---

## 🔧 Development Workflow

### 1. Package Development

When developing shared packages:

```bash
# Start package in watch mode
cd packages/your-package
pnpm dev

# Or from root
pnpm dev:packages
```

### 2. Application Development

Applications automatically use the latest package builds:

```bash
# Frontend development
pnpm dev:frontend

# Admin development
pnpm dev:admin
```

### 3. Full Stack Development

```bash
# Start everything
pnpm dev:all
```

---

## 🏷️ Environment Variables

Create `.env` files in relevant directories:

### Root `.env`

```env
NODE_ENV=development
```

### Frontend `.env.local`

```env
# Firebase Configuration
NEXT_PUBLIC_FIREBASE_API_KEY=your_api_key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_domain
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project_id

# Google OAuth
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret

# Other configurations...
```

---

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Quality

- All code must pass linting (`pnpm lint`)
- All tests must pass (`pnpm test`)
- Type checking must pass (`pnpm type-check`)
- Code must be formatted (`pnpm format`)

---

## 📚 Additional Resources

- [Turborepo Documentation](https://turbo.build/repo/docs)
- [PNPM Workspaces](https://pnpm.io/workspaces)
- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS](https://tailwindcss.com/docs)

---

## 📄 License

This project is licensed under the [Your License] - see the LICENSE file for details.

---

## 👥 Team

- **Fluke Jesadakorn** - _Lead Developer_ - [@fluke-jesadakorn](https://github.com/fluke-jesadakorn)

---

_Built with ❤️ for modern trading platforms_

- `pnpm dev` — Start development server
- `pnpm build` — Build for production
- `pnpm lint` — Lint code
- `pnpm format` — Format codebase
- `pnpm type-check` — Type check

---

## Contributing

- Use `pnpm` for dependencies
- Run `pnpm lint` and `pnpm format` before committing
- Follow code style enforced by Prettier and ESLint

---

## License

MIT (or specify your license)

---

## Acknowledgments

Built with [Next.js](https://nextjs.org/), [React](https://react.dev/), [Tailwind CSS](https://tailwindcss.com/), [Firebase](https://firebase.google.com/), and [Radix UI](https://www.radix-ui.com/).
