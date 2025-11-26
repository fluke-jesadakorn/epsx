# EPSX Analytics Platform 📊

A production-ready stock analytics platform built with modern technologies, featuring comprehensive EPS growth analysis and mobile-first performance. Built with Next.js 15, React 19, and high-performance Rust backend, configured for seamless local development with production-like *.epsx.io domain simulation.

[![TypeScript](https://img.shields.io/badge/TypeScript-5.8.3-007ACC?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-15.5.0-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![React](https://img.shields.io/badge/React-19.1.0-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-4.0.15-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

---

## 🏗️ Architecture

This production-ready monorepo is organized with clean separation of concerns:

### 🚀 Applications (`apps/`)

- **Frontend** (`apps/frontend`) - Main analytics platform interface
  - **URL**: https://epsx.io (local development)
  - Analytics dashboard, EPS rankings, user dashboard, subscription system
  - Next.js 15.5.0 with React 19.1.0 and TypeScript 5.8.3
  - Features: Advanced analytics, mobile-optimized data visualization, subscription management, real-time market data

- **Admin Frontend** (`apps/admin-frontend`) - Administrative dashboard
  - **URL**: https://admin.epsx.io (local development)
  - User management, IAM, role assignment, analytics administration
  - Next.js 15.5.0 with comprehensive admin controls
  - Features: User promotion tools, access control, analytics management, system monitoring

- **Backend** (`apps/backend`) - High-performance Rust API server with SQLx
  - **URL**: https://api.epsx.io (local development)
  - Axum framework with PostgreSQL (SQLx) and WebSocket support
  - **Database**: Async SQLx integration with compile-time SQL validation and built-in connection pooling
  - **Core Features**: EPS ranking analytics engine, TradingView integration, authentication/IAM, payment processing, security monitoring
  - **Enterprise Security**: Brute force detection, audit logging, webhook system
  - **Architecture**: Clean Architecture with 210+ Rust files and production-ready migration system
  - Includes CLI tools for admin operations and user management

### 📊 Core Analytics Features

- **EPS Growth Analytics** - Production-ready EPS ranking system with TradingView data integration
- **Real-time Market Data** - Live stock data streaming with WebSocket support and advanced caching
- **Advanced Filtering** - Multi-dimensional filtering by country, sector, growth metrics, and custom criteria
- **Mobile-Optimized** - Touch-friendly interface with responsive design for all device sizes
- **Enterprise Security** - Complete audit logging and threat monitoring system
- **High Performance** - Redis caching, connection pooling, and optimized database queries

---

## 🛠️ Technology Stack

### 🎨 Frontend Stack

| Technology | Version | Purpose |
|------------|---------|----------|
| **Next.js** | 15.5.0 | React framework with App Router |
| **React** | 19.1.0 | UI library with Server Components |
| **TypeScript** | 5.8.3 | Type-safe development |
| **Tailwind CSS** | 4.0.15 | Utility-first styling |
| **Radix UI** | Latest | Accessible component primitives |
| **Zustand** | Latest | State management |
| **SWR** | Latest | Data fetching & caching |
| **React Hook Form** | Latest | Form handling |
| **Zod** | Latest | Schema validation |

### ⚙️ Backend Stack

| Technology | Version | Purpose |
|------------|---------|----------|
| **Rust** | 2021 | High-performance systems language |
| **Axum** | 0.7 | Web framework with WebSocket support |
| **PostgreSQL** | Latest | Primary database |
| **SQLx** | 0.8 | Async-first SQL toolkit with compile-time validation |
| **Redis** | Latest | Caching & sessions |
| **JWT** | Latest | Authentication tokens |
| **Tokio** | Latest | Async runtime |

### 🛠️ Development Tools

| Tool | Version | Purpose |
|------|---------|----------|
| **Turborepo** | 2.5.5 | Monorepo build system |
| **pnpm** | 10.14.0 | Package manager |
| **ESLint** | 9.23.0 | Code linting |
| **Prettier** | 3.6.2 | Code formatting |
| **Husky** | 9.1.7 | Git hooks |
| **Jest** | Latest | Unit testing |
| **Playwright** | Latest | E2E testing |
| **Docker** | Latest | Containerization |

---

## 🚀 Quick Start

### Prerequisites

- **Node.js** >= 18.0.0
- **pnpm** >= 10.14.0
- **Rust** (latest stable)
- **PostgreSQL** (database)
- **Docker** (recommended for full environment)

### Installation & Setup

```bash
# 1. Clone and install dependencies
git clone https://github.com/fluke-jesadakorn/epsx.git
cd epsx
pnpm install

# 2. Configure local domains (add to /etc/hosts)
echo "127.0.0.1 epsx.io admin.epsx.io api.epsx.io" >> /etc/hosts

# 3. Setup environment
cp .env.example .env.development
# Edit .env.development with your database and API keys

# 4. Start development (choose one)
pnpm docker:dev     # Full environment with HTTPS
pnpm dev           # Frontend + Admin only
pnpm dev:all       # All applications including backend
```

### Access Points

- **Frontend**: https://epsx.io (or http://localhost:3000)
- **Admin**: https://admin.epsx.io (or http://localhost:3001)
- **API**: https://api.epsx.io (or http://localhost:8080)

---

## 🌐 Development Environment

### Local Domain Setup

EPSX uses custom *.epsx.io domains for production-like development:

- **Frontend**: https://epsx.io (data analytics platform)
- **Admin**: https://admin.epsx.io (dashboard)
- **API**: https://api.epsx.io (backend)

**Quick Setup:**
```bash
# Add domains to hosts file
echo "127.0.0.1 epsx.io admin.epsx.io api.epsx.io" >> /etc/hosts

# Start with HTTPS (recommended)
pnpm docker:dev

# Or start without HTTPS
pnpm dev:all
```

### Authentication Features

- **Single Sign-On**: Seamless cross-application authentication
- **JWT Security**: Secure token-based API access
- **Multi-Provider**: Google OAuth, Firebase, OIDC support
- **Role-Based Access**: IAM profiles with granular permissions

---

## 📜 Development Commands

### 🚀 Development Servers

```bash
# Quick start options
pnpm dev               # Frontend + Admin (ports 3000, 3001)
pnpm dev:all           # All applications including backend
pnpm docker:dev        # Full environment with HTTPS

# Individual applications
pnpm dev:frontend      # Analytics platform (port 3000)
pnpm dev:admin         # Admin dashboard (port 3001)
pnpm dev:backend       # Rust API server (port 8080)
```

### 🛠️ Build & Quality

```bash
# Building
pnpm build             # Build all applications
pnpm build:frontend    # Frontend only
pnpm build:admin       # Admin only
pnpm build:backend     # Rust backend only

# Quality checks
pnpm lint              # ESLint all projects
pnpm lint:fix          # Auto-fix issues
pnpm type-check        # TypeScript validation
pnpm format            # Prettier formatting

# Testing
pnpm test              # All tests
pnpm test:unit         # Jest unit tests
pnpm test:e2e          # Playwright E2E
pnpm test:e2e:ui       # E2E with UI mode
```

### 👥 Admin & Management

```bash
# User management
pnpm promote-admin     # Promote user to admin
pnpm assign-iam        # Assign IAM profile
pnpm list-profiles     # List IAM profiles

# Environment management
pnpm docker:dev        # Start development
pnpm docker:dev:down   # Stop development
pnpm docker:dev:logs   # View logs

# Performance analysis
pnpm lighthouse        # Performance tests
pnpm analyze          # Bundle analysis
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
│   ├── frontend/          # Analytics platform (Next.js 15 + React 19)
│   │   ├── app/           # App Router pages & layouts
│   │   ├── components/    # React components (analytics, auth, touch, ui)
│   │   ├── hooks/         # Custom hooks
│   │   └── lib/           # Utilities & API clients
│   ├── admin-frontend/    # Admin dashboard (Next.js 15)
│   │   ├── app/           # Admin pages
│   │   ├── components/    # Admin components (admin, analytics, auth)
│   │   └── lib/           # Admin utilities
│   └── backend/           # Rust API server (Axum + PostgreSQL)
│       ├── src/           # Rust source (web, auth, analytics)
│       ├── migrations/    # Database migrations
│       └── tests/         # Rust tests
├── scripts/               # Development & deployment scripts
├── types/                 # Shared TypeScript definitions
└── turbo.json            # Monorepo configuration
```


---

## 🏷️ Environment Configuration

### Quick Setup

```bash
# Copy environment template
cp .env.example .env.development

# Configure key variables
echo "DATABASE_URL=postgresql://username:password@localhost:5432/epsx_dev" >> .env.development
echo "NEXT_PUBLIC_API_URL=https://api.epsx.io" >> .env.development
```

### Required Variables

| Application | Variable | Purpose |
|-------------|----------|----------|
| **Database** | `DATABASE_URL` | PostgreSQL connection |
| **Frontend** | `NEXT_PUBLIC_API_URL` | Backend API endpoint |
| **Auth** | `NEXTAUTH_SECRET` | JWT signing secret |
| **Firebase** | `NEXT_PUBLIC_FIREBASE_*` | Firebase configuration |
| **OAuth** | `GOOGLE_CLIENT_ID/SECRET` | Google authentication |

### Environment Files

- `.env.development` - Local development
- `.env.prod.example` - Production template
- `apps/frontend/.env.local` - Frontend-specific
- `apps/admin-frontend/.env.local` - Admin-specific
- `apps/backend/.env` - Backend configuration

---

## 🛡️ Enterprise Security

### Production-Grade Security Systems

- **Advanced Threat Detection**: ML-powered brute force detection with pattern analysis and automated response
- **Multi-Provider Authentication**: OIDC, Firebase, and JWT with secure token rotation and session management
- **Structured Permission System**: Modern permission architecture with platform-scoped access control
  - **Format**: `"platform:resource:action"` (e.g., `"epsx:users:manage"`, `"epsx-pay:transactions:read"`)
  - **Platforms**: `epsx` (main platform), `epsx-pay` (payments), `epsx-token` (crypto), `admin` (cross-platform)
  - **Migration Status**: 100% complete from legacy admin_modules system with 50% faster queries
  - **Examples**: `["admin:users:manage"]` - User management, `["epsx:analytics:view"]` - Analytics access
- **Security Monitoring**: Real-time security event correlation, alerting system, and comprehensive audit logging
- **Data Protection**: Encryption at rest and in transit, automated data classification, and retention management

### Production-Ready Systems

- **EPS Analytics Engine**: Complete stock analysis with TradingView integration and real-time data
- **Enterprise Security**: Brute force detection, security alerts, threat monitoring with ML-based analysis
- **Performance Monitoring**: Advanced caching, connection pooling, and system health monitoring
- **WebSocket Support**: Real-time data streaming for analytics and admin notifications
- **Payment Integration**: Complete subscription management with webhook processing

## 🧪 Testing Strategy

### Frontend Testing
- **Unit Tests**: Jest with React Testing Library
- **E2E Tests**: Playwright with parallel execution
- **Performance Tests**: Lighthouse integration
- **SSR Testing**: Server-side rendering validation

### Backend Testing
- **Unit Tests**: Rust native testing framework for core business logic with SQLx integration
- **Integration Tests**: EPS analytics, authentication flows, and database operations using SQLx queries
- **Architecture Testing**: Clean Architecture layers (domain, application, infrastructure, presentation)
- **Security Testing**: Brute force detection and threat monitoring with SQLx audit trails
- **Performance Testing**: Load testing for analytics endpoints, caching systems, and SQLx connection pooling
- **Database Testing**: SQLx validation, migration testing, and compile-time SQL verification
- **API Testing**: Comprehensive endpoint validation for analytics, auth, and admin functions

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
- **Registry**: `us-central1-docker.pkg.dev/epsx-469400/epsx`
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

Built with modern technologies for enterprise-grade data analytics platforms:

- [Next.js 15](https://nextjs.org/) - React framework with App Router
- [React 19](https://react.dev/) - Latest React with concurrent features
- [Tailwind CSS 4](https://tailwindcss.com/) - Utility-first CSS framework
- [Radix UI](https://www.radix-ui.com/) - Accessible component primitives
- [Rust Axum](https://docs.rs/axum/) - High-performance web framework
- [PostgreSQL](https://www.postgresql.org/) - Advanced relational database
- [Firebase](https://firebase.google.com/) - Authentication and analytics
- [Turborepo](https://turbo.build/) - Build system and monorepo tools

---

_🔥 Built with ❤️ for modern data analytics platforms_
