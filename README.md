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
  - **Blockchain**: Automatic payment monitoring and subscription activation via BSC (blockchain-monitor binary)
  - **Enterprise Security**: Brute force detection, audit logging, webhook system
  - **Architecture**: Clean Architecture with 210+ Rust files and production-ready migration system
  - Includes CLI tools for admin operations, user management, and blockchain payment monitoring

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
| **Ethers-rs** | Latest | Ethereum/BSC blockchain integration |
| **Alloy** | Latest | Modern Ethereum library |
| **SIWE** | Latest | Sign-In with Ethereum authentication |

### 🛠️ Development Tools

| Tool | Version | Purpose |
|------|---------|----------|
| **Turborepo** | 2.5.5 | Monorepo build system |
| **Bun** | 1.1.42 | Package manager & runtime |
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
- **Bun** >= 1.1.0
- **Rust** (latest stable)
- **PostgreSQL** (database)
- **Docker** (recommended for full environment)

### Installation & Setup

```bash
# 1. Clone and install dependencies
git clone https://github.com/fluke-jesadakorn/epsx.git
cd epsx
bun install

# 2. Configure local domains (add to /etc/hosts)
echo "127.0.0.1 epsx.io admin.epsx.io api.epsx.io" >> /etc/hosts

# 3. Setup environment
cp .env.example .env.development
# Edit .env.development with your database and API keys

# 4. Start development (choose one)
bun docker:dev     # Full environment with HTTPS
bun dev           # Frontend + Admin only
bun dev:all       # All applications including backend
```

### Access Points

- **Frontend**: https://epsx.io (or http://localhost:3000)
- **Admin**: https://admin.epsx.io (or http://localhost:3001)
- **API**: https://api.epsx.io (or http://localhost:8080)

---

## 🌐 Development Environment

### Local Domain Setup

EPSX uses custom *.epsx.io domains for production-like development:

- **Frontend**: https://epsx.io (trading platform)
- **Admin**: https://admin.epsx.io (dashboard)
- **API**: https://api.epsx.io (backend)

**Quick Setup:**
```bash
# Add domains to hosts file
echo "127.0.0.1 epsx.io admin.epsx.io api.epsx.io" >> /etc/hosts

# Start with HTTPS (recommended)
bun docker:dev

# Or start without HTTPS
bun dev:all
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
bun dev               # Frontend + Admin (ports 3000, 3001)
bun dev:all           # All applications including backend
bun docker:dev        # Full environment with HTTPS

# Individual applications
bun dev:frontend      # Analytics platform (port 3000)
bun dev:admin         # Admin dashboard (port 3001)
bun dev:backend       # Rust API server (port 8080)
```

### 🛠️ Build & Quality

```bash
# Building
bun build             # Build all applications
bun build:frontend    # Frontend only
bun build:admin       # Admin only
bun build:backend     # Rust backend only

# Quality checks
bun lint              # ESLint all projects
bun lint:fix          # Auto-fix issues
bun type-check        # TypeScript validation
bun format            # Prettier formatting

# Testing
bun test              # All tests
bun test:unit         # Jest unit tests
bun test:e2e          # Playwright E2E
bun test:e2e:ui       # E2E with UI mode
```

### 👥 Admin & Management

```bash
# User management
bun promote-admin     # Promote user to admin
bun assign-iam        # Assign IAM profile
bun list-profiles     # List IAM profiles

# Environment management
bun docker:dev        # Start development
bun docker:dev:down   # Stop development
bun docker:dev:logs   # View logs

# Performance analysis
bun lighthouse        # Performance tests
bun analyze          # Bundle analysis
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

# Using bun scripts
bun docker:dev           # Start development containers
bun docker:test          # Start test environment
bun docker:prod          # Start production containers
bun docker:dev:down      # Stop development containers
bun docker:dev:logs      # View development logs

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

## 💰 Blockchain Payment System

### Overview

EPSX includes a production-ready blockchain payment monitoring system that automatically processes cryptocurrency payments on Binance Smart Chain (BSC) and activates user subscriptions without manual intervention.

**Key Features:**
- **Automatic User Onboarding**: Creates wallet users automatically on first payment
- **Instant Activation**: Subscriptions active within 5-10 seconds of blockchain confirmation
- **Multi-Token Support**: Accepts USDT and USDC stablecoin payments
- **Duplicate Prevention**: Idempotent processing with unique transaction tracking
- **Multi-Chain**: Supports both BSC Mainnet and Testnet environments

### Architecture

```
BSC Blockchain → Event Listener → Payment Verifier → User Creation → Subscription Activation → PostgreSQL
```

The `blockchain-monitor` standalone binary polls the BSC blockchain every 3 seconds for PaymentReceived events from the deployed PaymentEscrow smart contract, verifies payment amounts and tokens, creates wallet users if needed, and activates subscriptions automatically.

### Quick Start

```bash
# 1. Build the blockchain monitor binary
cd apps/backend
DATABASE_URL=postgresql://... cargo build --release --bin blockchain-monitor

# 2. Configure environment variables
cat >> .env << 'EOF'
# Blockchain Payment Monitor
BLOCKCHAIN_NETWORK=testnet
BLOCKCHAIN_START_BLOCK=0
BLOCKCHAIN_POLL_INTERVAL_SECONDS=3
BSC_TESTNET_RPC_URL=https://data-seed-prebsc-1-s1.binance.org:8545
PAYMENT_ESCROW_CONTRACT_TESTNET=0x...  # After smart contract deployment
EOF

# 3. Verify setup
./scripts/verify-blockchain-setup.sh

# 4. Run the monitor
./target/release/blockchain-monitor
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `BLOCKCHAIN_NETWORK` | Yes | Network: `testnet` or `mainnet` |
| `BSC_TESTNET_RPC_URL` | Yes (testnet) | BSC testnet RPC endpoint |
| `BSC_MAINNET_RPC_URL` | Yes (mainnet) | BSC mainnet RPC endpoint |
| `PAYMENT_ESCROW_CONTRACT_TESTNET` | Yes (testnet) | Deployed contract address (testnet) |
| `PAYMENT_ESCROW_CONTRACT_MAINNET` | Yes (mainnet) | Deployed contract address (mainnet) |
| `BLOCKCHAIN_START_BLOCK` | No | Starting block number (default: 0) |
| `BLOCKCHAIN_POLL_INTERVAL_SECONDS` | No | Polling interval (default: 3) |

### Database Schema

The system uses the following tables:
- **processed_blockchain_events**: Event tracking and duplicate prevention
- **wallet_users**: Automatic user creation on first payment
- **active_subscriptions**: Subscription management with JSONB metadata
- **pricing_plans**: Plan definitions and pricing validation

### Deployment Documentation

For complete deployment instructions including:
- Smart contract deployment (PaymentEscrow.sol)
- Production deployment options (systemd, Docker, Cloud Run)
- End-to-end testing procedures
- Security considerations and fund management

See: [BLOCKCHAIN_DEPLOYMENT_GUIDE.md](BLOCKCHAIN_DEPLOYMENT_GUIDE.md)

### Implementation Status

- **Smart Contract**: PaymentEscrow.sol - Ready for deployment
- **Backend**: blockchain-monitor binary - Production-ready (8.5MB optimized)
- **Database**: Migration applied and verified
- **Documentation**: Complete deployment and testing guides

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

- All code must pass linting (`bun lint`)
- All tests must pass (`bun test`)
- Type checking must pass (`bun type-check`)
- Code must be formatted (`bun format`)
- Husky pre-commit hooks must pass

---

## 📚 Additional Resources

### Documentation
- [Turborepo Documentation](https://turbo.build/repo/docs)
- [Bun Documentation](https://bun.sh/docs)
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
