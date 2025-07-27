# EPSX Trading Platform 📈

A comprehensive trading platform monorepo built with modern technologies, featuring a Next.js frontend ecosystem with admin dashboard and a high-performance Rust backend.

[![TypeScript](https://img.shields.io/badge/TypeScript-5.8.3-007ACC?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-15.4.2-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![React](https://img.shields.io/badge/React-19.1.0-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-4.0.15-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

---

## 🏗️ Architecture

This production-ready monorepo is organized with clean separation of concerns:

### 🚀 Applications (`apps/`)

- **Frontend** (`apps/frontend`) - Main trading platform interface (Port: 3000)
  - Trading interface, user dashboard, payment system, analytics
  - Next.js 15.4.2 with React 19.1.0 and TypeScript 5.8.3
  - Features: Real-time trading, payment processing, performance analytics

- **Admin Frontend** (`apps/admin-frontend`) - Administrative dashboard (Port: 3001)
  - User management, IAM, role assignment, system analytics
  - Next.js 15.4.2 with comprehensive admin controls
  - Features: User promotion tools, access control, system monitoring

- **Backend** (`apps/backend`) - High-performance Rust API server
  - Axum framework with PostgreSQL and WebSocket support
  - Real-time data, authentication, trading logic, payment processing
  - Includes CLI tools for admin operations and user management

### 📦 Shared Packages (`packages/`)

- **@epsx/api-client** - Unified API client with cookie-based authentication
- **@epsx/ui** - Shared UI components built with Radix UI + Tailwind CSS
- **@epsx/types** - Comprehensive TypeScript type definitions
- **@epsx/config** - ESLint, TypeScript, and build configurations
- **@epsx/theme** - Design system and theme provider

---

## 🛠️ Tech Stack

### Frontend Technologies

- **Framework:** Next.js 15.4.2 with App Router
- **Language:** TypeScript 5.8.3
- **UI Components:** Radix UI + Tailwind CSS 4.0.15
- **State Management:** Zustand + SWR for data fetching
- **Forms:** React Hook Form with Zod validation
- **Authentication:** Firebase Auth integration
- **Charts:** Recharts for data visualization
- **Testing:** Jest (unit), Playwright (E2E), Lighthouse (performance)

### Backend Technologies

- **Language:** Rust (Edition 2021)
- **Web Framework:** Axum 0.7 with WebSocket support
- **Database:** PostgreSQL with SQLx ORM
- **Authentication:** JWT with role-based access control
- **Real-time:** WebSocket and Server-Sent Events
- **Job Scheduling:** tokio-cron-scheduler
- **Testing:** Unit, integration, and API tests

### Development Tools

- **Monorepo:** Turborepo 2.5.4 for build orchestration
- **Package Manager:** pnpm 10.13.1 with workspaces
- **Node.js:** >=18.0.0 requirement
- **Code Quality:** ESLint 9.23.0 + Prettier 3.6.2
- **Git Hooks:** Husky 9.1.7 with Commitlint
- **Containerization:** Docker with multi-platform support

---

## 🚀 Quick Start

### Prerequisites

- **Node.js** >= 18.0.0
- **pnpm** >= 10.13.1
- **Rust** (latest stable for backend development)
- **PostgreSQL** (for database)
- **Docker** (optional, for containerized development)

### Installation

```bash
# Clone the repository
git clone https://github.com/fluke-jesadakorn/epsx.git
cd epsx

# Install dependencies
pnpm install

# Setup environment variables
cp .env.example .env.development
# Edit .env.development with your configuration

# Setup database (PostgreSQL required)
# Configure your database connection in backend/.env

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

# Testing
pnpm test              # Run all tests
pnpm test:unit         # Unit tests only
pnpm test:e2e          # E2E tests with Playwright
pnpm test:e2e:ui       # E2E tests with UI mode

# Formatting
pnpm format            # Format all files
pnpm format:check      # Check formatting
```

### Admin & Management

```bash
# User management
pnpm promote-admin     # Promote user to admin role
pnpm assign-iam        # Assign IAM profile to user
pnpm list-profiles     # List available IAM profiles

# Docker operations
pnpm docker:dev        # Start development environment
pnpm docker:test       # Start test environment
pnpm docker:prod       # Start production environment
pnpm docker:dev:down   # Stop development environment
pnpm docker:dev:logs   # View development logs

# Performance & Analysis
pnpm lighthouse        # Run Lighthouse performance tests
pnpm analyze          # Analyze bundle sizes (frontend)
```

---

## 🐳 Docker Development

```bash
# Using Makefile (recommended)
make install              # Install dependencies
make dev                  # Start development servers
make build                # Build all applications
make docker-up ENV=dev    # Start development environment
make docker-down ENV=dev  # Stop development environment
make docker-logs ENV=dev  # View logs

# Using pnpm scripts
pnpm docker:dev           # Start development containers
pnpm docker:test          # Start test environment
pnpm docker:prod          # Start production containers
pnpm docker:dev:down      # Stop development containers
pnpm docker:dev:logs      # View development logs

# Container registry (Google Cloud)
make docker-build         # Build and tag images
make docker-push          # Push to registry
```

---

## 📁 Project Structure

```
epsx/
├── apps/
│   ├── frontend/          # Main trading platform (Port: 3000)
│   │   ├── app/           # Next.js App Router pages
│   │   ├── components/    # React components
│   │   ├── lib/           # Utilities and configurations
│   │   └── public/        # Static assets
│   ├── admin-frontend/    # Admin dashboard (Port: 3001)
│   │   ├── app/           # Admin pages and API routes
│   │   ├── components/    # Admin-specific components
│   │   ├── auth/          # Authentication context
│   │   └── services/      # Admin services
│   └── backend/           # Rust API server
│       ├── src/           # Rust source code
│       ├── migrations/    # Database migrations
│       └── bin/           # CLI tools (promote_admin, assign_iam)
├── packages/
│   ├── api-client/        # Unified API client with auth
│   ├── ui/                # Shared UI components (Radix + Tailwind)
│   ├── types/             # TypeScript type definitions
│   ├── config/            # ESLint, TypeScript configs
│   └── theme/             # Design system and theme provider
├── scripts/               # Admin and deployment scripts
├── .todo/                 # Task management
├── Makefile              # Build and Docker commands
└── turbo.json            # Turborepo configuration
```

---

## 🔧 Development Workflow

### 1. Initial Setup

```bash
# Install dependencies and setup environment
make install
cp .env.example .env.development
# Configure your environment variables
```

### 2. Package Development

Shared packages are built with Turborepo dependency management:

```bash
# Build packages first (required for apps)
pnpm build:packages

# Watch mode for package development
pnpm dev:packages
```

### 3. Application Development

```bash
# Start individual applications
pnpm dev:frontend    # Trading platform (Port: 3000)
pnpm dev:admin       # Admin dashboard (Port: 3001)
pnpm dev:backend     # Rust API server

# Start all frontend applications
pnpm dev             # Frontend + Admin

# Full stack development
pnpm dev:all         # All apps including backend
```

### 4. Administration & User Management

```bash
# Promote user to admin role
pnpm promote-admin

# Assign IAM profile to user
pnpm assign-iam

# View available IAM profiles
pnpm list-profiles
```

---

## 🏷️ Environment Variables

### Available Environment Files

- `.env.example` - Template for all environments
- `.env.development` - Development configuration
- `.env.prod.example` - Production template

### Root `.env.development`

```env
NODE_ENV=development
DATABASE_URL=postgresql://username:password@localhost:5432/epsx_dev
```

### Frontend/Admin `.env.local`

```env
# Firebase Configuration
NEXT_PUBLIC_FIREBASE_API_KEY=your_api_key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_domain
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project_id
NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=your_bucket
NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
NEXT_PUBLIC_FIREBASE_APP_ID=your_app_id

# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_ADMIN_API_URL=http://localhost:8080/admin

# Google OAuth
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret

# Authentication
NEXTAUTH_SECRET=your_nextauth_secret
NEXTAUTH_URL=http://localhost:3000
```

### Backend `.env`

```env
# Database
DATABASE_URL=postgresql://username:password@localhost:5432/epsx_dev
REDIS_URL=redis://localhost:6379

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
JWT_SECRET=your_jwt_secret

# External APIs
FIREBASE_SERVICE_ACCOUNT_KEY=path/to/service-account.json
```

---

## 🛡️ Security Features

### Authentication & Authorization

- **Multi-tier Role System**: Basic, Premium, Moderator, Admin roles
- **IAM Profiles**: 
  - `user-basic-001` - Basic user permissions
  - `user-premium-002` - Premium user features
  - `moderator-standard-003` - Moderation capabilities
  - `admin-full-004` - Full system access
- **JWT Authentication**: Secure token-based authentication
- **Firebase Integration**: Social login and user management
- **Permission Matrices**: Feature-based access control

### Real-time Features

- **WebSocket Support**: Live trading data and notifications
- **Server-Sent Events**: Real-time updates for admin dashboard
- **Payment Tracking**: Real-time payment status monitoring
- **Live Data Streaming**: Market data and user activity streams

## 🧪 Testing Strategy

### Frontend Testing
- **Unit Tests**: Jest with React Testing Library
- **E2E Tests**: Playwright with parallel execution
- **Performance Tests**: Lighthouse integration
- **SSR Testing**: Server-side rendering validation

### Backend Testing
- **Unit Tests**: Rust native testing framework
- **Integration Tests**: Database and API endpoint testing
- **Load Testing**: Performance and stress testing
- **Security Testing**: Authentication and authorization validation

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Quality Requirements

- All code must pass linting (`pnpm lint`)
- All tests must pass (`pnpm test`)
- Type checking must pass (`pnpm type-check`)
- Code must be formatted (`pnpm format`)
- Husky pre-commit hooks must pass

---

## 📚 Additional Resources

### Documentation
- [Turborepo Documentation](https://turbo.build/repo/docs)
- [pnpm Workspaces](https://pnpm.io/workspaces)
- [Next.js 15 Documentation](https://nextjs.org/docs)
- [Rust Axum Framework](https://docs.rs/axum/)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [Radix UI Components](https://www.radix-ui.com/)

### Performance & Monitoring
- [Lighthouse Performance](https://web.dev/lighthouse/)
- [Firebase Analytics](https://firebase.google.com/docs/analytics)
- [PostgreSQL Performance](https://www.postgresql.org/docs/)

---

## 🚀 Deployment

### Google Cloud Registry
- **Registry**: `us-central1-docker.pkg.dev/epsx-449804/epsx`
- **Multi-platform**: linux/amd64, linux/arm64 support
- **Environments**: Development, test, production configurations

### Build Pipeline
```bash
# Build for production
make build

# Build and push Docker images
make docker-build
make docker-push

# Deploy to environment
make docker-up ENV=prod
```

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## 👥 Team

- **Fluke Jesadakorn** - _Lead Developer_ - [@fluke-jesadakorn](https://github.com/fluke-jesadakorn)

---

## 🙏 Acknowledgments

Built with modern technologies for enterprise-grade trading platforms:

- [Next.js 15](https://nextjs.org/) - React framework with App Router
- [React 19](https://react.dev/) - Latest React with concurrent features
- [Tailwind CSS 4](https://tailwindcss.com/) - Utility-first CSS framework
- [Radix UI](https://www.radix-ui.com/) - Accessible component primitives
- [Rust Axum](https://docs.rs/axum/) - High-performance web framework
- [PostgreSQL](https://www.postgresql.org/) - Advanced relational database
- [Firebase](https://firebase.google.com/) - Authentication and analytics
- [Turborepo](https://turbo.build/) - Build system and monorepo tools

---

_🔥 Built with ❤️ for modern trading platforms_
