# EPSX Trading Platform 📈

A comprehensive trading platform monorepo built with modern technologies, featuring an analytics-focused theme with enhanced mobile performance. The Next.js frontend ecosystem includes comprehensive analytics dashboards and high-performance Rust backend. Configured for local development with custom *.epsx.io domains for production-like environment simulation.

[![TypeScript](https://img.shields.io/badge/TypeScript-5.8.3-007ACC?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-15.5.0-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![React](https://img.shields.io/badge/React-19.1.0-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-4.0.15-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

---

## 🏗️ Architecture

This production-ready monorepo is organized with clean separation of concerns:

### 🚀 Applications (`apps/`)

- **Frontend** (`apps/frontend`) - Main trading platform interface
  - **URL**: https://epsx.io (local development)
  - Analytics dashboard, trading interface, user dashboard, payment system
  - Next.js 15.5.0 with React 19.1.0 and TypeScript 5.8.3
  - Features: Advanced analytics, mobile-optimized trading, payment processing, real-time market data

- **Admin Frontend** (`apps/admin-frontend`) - Administrative dashboard
  - **URL**: https://admin.epsx.io (local development)
  - User management, IAM, role assignment, analytics administration
  - Next.js 15.5.0 with comprehensive admin controls
  - Features: User promotion tools, access control, analytics management, system monitoring

- **Backend** (`apps/backend`) - High-performance Rust API server
  - **URL**: https://api.epsx.io (local development)
  - Axum framework with PostgreSQL and WebSocket support
  - Analytics processing, real-time market data, authentication, trading logic, payment processing
  - Includes CLI tools for admin operations and user management

### 🎨 Analytics Theme & Features

- **Analytics Dashboard** - Comprehensive stock analysis and market insights with advanced filtering
- **Mobile-First Design** - Touch-optimized interface with enhanced mobile performance optimizations
- **Real-time Data Streaming** - Live market data with WebSocket integration and data visualization
- **Responsive Components** - Adaptive layouts optimized for all device sizes and touch interactions
- **Advanced Charts** - Interactive financial data visualization with Recharts and custom chart components

---

## 🛠️ Tech Stack

### Frontend Technologies

- **Framework:** Next.js 15.5.0 with App Router
- **Language:** TypeScript 5.8.3
- **UI Components:** Radix UI + Tailwind CSS 4.0.15
- **State Management:** Zustand + SWR for data fetching
- **Forms:** React Hook Form with Zod validation
- **Authentication:** Multi-provider authentication (Firebase, OIDC)
- **Charts:** Recharts for data visualization
- **Testing:** Jest (unit), Playwright (E2E), Lighthouse (performance)

### Backend Technologies

- **Language:** Rust (Edition 2021)
- **Web Framework:** Axum 0.7 with WebSocket support
- **Database:** PostgreSQL with SQLx ORM and comprehensive migrations
- **Authentication:** Multi-provider JWT with RBAC, OIDC, and Firebase integration
- **Analytics Engine:** High-performance stock analysis and market data processing
- **Real-time:** WebSocket and Server-Sent Events for live market updates
- **Job Scheduling:** tokio-cron-scheduler for automated analytics processing
- **Testing:** Unit, integration, and API tests with comprehensive E2E coverage

### Development Tools

- **Monorepo:** Turborepo 2.5.5 for build orchestration
- **Package Manager:** pnpm 10.14.0 with workspaces
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
- **Local DNS Configuration** (for *.epsx.io domains)

### Installation

```bash
# Clone the repository
git clone https://github.com/fluke-jesadakorn/epsx.git
cd epsx

# Install dependencies
pnpm install

# Setup local DNS for *.epsx.io domains
# Add to /etc/hosts (macOS/Linux) or C:\Windows\System32\drivers\etc\hosts (Windows):
127.0.0.1 epsx.io
127.0.0.1 admin.epsx.io
127.0.0.1 api.epsx.io

# Setup environment variables
cp .env.example .env.development
# Edit .env.development with your configuration

# Setup database (PostgreSQL required)
# Configure your database connection in backend/.env

# Start development servers (with HTTPS via Docker/Traefik)
pnpm docker:dev
# OR for local development without HTTPS
pnpm dev
```

---

## 🌐 Local Domain Configuration

EPSX is configured to use custom *.epsx.io domains for local development, providing a production-like environment with proper subdomain separation.

### Domain Architecture

- **Main Application**: https://epsx.io - Trading platform frontend
- **Admin Dashboard**: https://admin.epsx.io - Administrative interface  
- **API Server**: https://api.epsx.io - Backend API and authentication

### Setting Up Local DNS

#### Option 1: Manual /etc/hosts Configuration

Add the following entries to your hosts file:

**macOS/Linux**: `/etc/hosts`
```bash
# Add these lines to /etc/hosts
127.0.0.1 epsx.io
127.0.0.1 admin.epsx.io  
127.0.0.1 api.epsx.io
```

**Windows**: `C:\Windows\System32\drivers\etc\hosts`
```bash
# Add these lines to hosts file
127.0.0.1 epsx.io
127.0.0.1 admin.epsx.io
127.0.0.1 api.epsx.io
```

#### Option 2: Using Docker Development Environment

The Docker development environment includes Traefik reverse proxy that automatically handles SSL certificates and domain routing:

```bash
# Start the complete development environment with HTTPS
pnpm docker:dev

# Access applications:
# https://epsx.io - Frontend
# https://admin.epsx.io - Admin  
# https://api.epsx.io - Backend API
```

### Authentication Integration

The authentication system is fully integrated across all domains:

- **Single Sign-On**: Login once, access all applications
- **Secure Sessions**: JWT tokens with secure cookie handling
- **Cross-Domain Auth**: Session sharing between epsx.io and admin.epsx.io
- **API Authentication**: Bearer tokens for api.epsx.io

### Development Benefits

- **Production Parity**: Mirrors production domain structure
- **CORS Testing**: Proper cross-origin request testing
- **SSL Development**: HTTPS in development environment
- **Subdomain Testing**: Validate subdomain-specific features
- **Authentication Flow**: Test complete OAuth/OIDC flows

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
pnpm dev:all          # All applications including backend

# Start all including backend
pnpm dev:all
```

### Building

```bash
# Build everything
pnpm build

# Build specific targets  
pnpm build:apps        # Build all applications
pnpm build:frontend    # Build frontend app
pnpm build:admin       # Build admin app
pnpm build:backend     # Build Rust backend
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
│   │   ├── app/           # Next.js App Router with analytics pages
│   │   ├── components/    # React components including analytics dashboard
│   │   │   ├── analytics/ # Advanced analytics and market data components
│   │   │   ├── auth/      # Multi-provider authentication components
│   │   │   ├── touch/     # Mobile-optimized touch interaction components
│   │   │   └── ui/        # Shared UI components (Radix + Tailwind)
│   │   ├── hooks/         # Custom hooks for analytics and mobile performance
│   │   ├── lib/           # Utilities, API clients, and authentication helpers
│   │   └── styles/        # Analytics theme CSS and mobile optimizations
│   ├── admin-frontend/    # Admin dashboard (Port: 3001)
│   │   ├── app/           # Admin pages including analytics management
│   │   ├── components/    # Admin components with analytics administration
│   │   │   ├── admin/     # Core admin management components
│   │   │   ├── analytics/ # Analytics administration interface
│   │   │   └── auth/      # Enhanced authentication with role management
│   │   ├── lib/           # Admin utilities and API integration
│   │   └── types/         # Admin-specific TypeScript definitions
│   └── backend/           # High-performance Rust API server
│       ├── src/           # Rust source code with analytics engine
│       │   ├── web/analytics/ # Analytics API handlers and data processing
│       │   ├── auth/      # Multi-provider authentication system
│       │   └── stock/     # Market data and financial calculations
│       ├── migrations/    # Database migrations for analytics data
│       ├── templates/analytics/ # Analytics theme HTML templates
│       └── tests/         # Comprehensive testing including analytics
├── scripts/               # Deployment and development scripts
├── types/                 # Shared TypeScript definitions
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

### 2. Analytics Development

The platform features an analytics-focused theme with mobile optimizations:

```bash
# Start analytics development environment
pnpm dev:frontend    # Frontend with comprehensive analytics dashboard
pnpm dev:admin       # Admin with analytics administration

# Mobile-optimized development with touch interactions
pnpm dev             # All applications with enhanced mobile performance
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

# API Configuration (Local *.epsx.io domains)
NEXT_PUBLIC_API_URL=https://api.epsx.io
NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io
NEXT_PUBLIC_APP_URL=https://epsx.io
NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io

# Google OAuth
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret

# Authentication
NEXTAUTH_SECRET=your_nextauth_secret
NEXTAUTH_URL=https://epsx.io

# For Admin Frontend
NEXTAUTH_URL=https://admin.epsx.io
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

# Frontend URLs for CORS
FRONTEND_URL=https://epsx.io
ADMIN_FRONTEND_URL=https://admin.epsx.io
OIDC_ISSUER=https://api.epsx.io

# External APIs
FIREBASE_SERVICE_ACCOUNT_KEY=path/to/service-account.json
```

---

## 🛡️ Security Features

### Authentication & Authorization

- **Multi-Provider Authentication**: Comprehensive OIDC and Firebase integration
- **Multi-tier Role System**: Basic, Premium, Moderator, Admin roles with database role management
- **IAM Profiles**: 
  - `user-basic-001` - Basic user permissions
  - `user-premium-002` - Premium user features
  - `moderator-standard-003` - Moderation capabilities
  - `admin-full-004` - Full system access
- **Advanced Security Features**:
  - JWT token management with secure rotation
  - Threat protection and rate limiting
  - WebAuthn security integration
  - Cross-app session synchronization
  - CSRF protection and security headers
- **Enterprise Features**:
  - SCIM protocol support for user provisioning
  - Audit events and compliance logging
  - Tenant resolution and session federation
  - Casbin-based authorization engine

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
- **Integration Tests**: Comprehensive authentication flow testing, database and API endpoint testing
- **Firebase Integration Tests**: End-to-end authentication provider testing
- **OIDC Comprehensive Tests**: Multi-provider authentication validation
- **Load Testing**: Performance and stress testing
- **Security Testing**: Advanced authentication, authorization, and compliance validation

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
