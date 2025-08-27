# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EPSX is a production-ready analytics platform built with modern technologies, featuring enterprise-grade security and mobile-first performance. The architecture consists of:

- **Frontend** (Port 3000): Next.js 15.5.0 + React 19.1.0 with EPS analytics dashboard
- **Admin Frontend** (Port 3001): Administrative dashboard for user/IAM management and system monitoring
- **Backend** (Port 8080): High-performance Rust API server with **complete Diesel ORM migration**, EPS analytics engine, TradingView integration, and enterprise security
- **Unified Monorepo**: Streamlined structure optimized for analytics performance and development velocity

## 🚀 **Major Update: Complete SQLx to Diesel ORM Migration**

**Migration Status: 100% Complete** ✅

The EPSX backend has successfully completed its comprehensive migration from SQLx to Diesel ORM, providing enhanced type safety, performance, and maintainability.

### Migration Components Completed:
- ✅ **Database Schema**: All production tables migrated to Diesel with auto-generated schema
- ✅ **Core Models**: Complete Diesel model definitions for all entities (210+ Rust files)
- ✅ **Repository Layer**: Full Diesel repository implementation with bb8 connection pooling
- ✅ **Notification System**: Complete async CRUD operations with Diesel
- ✅ **Security System**: Complete models and repositories for security events, alerts, and audit trails
- ✅ **Migration Infrastructure**: Consolidated production schema migration (2025-01-15)
- ✅ **Testing Framework**: Updated test suite with Diesel integration

### Key Migration Benefits:
- **Compile-time SQL Validation**: Catch database errors at compile time
- **Type Safety**: Strong typing prevents runtime database errors  
- **Performance**: Connection pooling with bb8-diesel
- **Schema Management**: Automated migrations with diesel_migrations
- **Developer Experience**: Rich IDE support and query builder

## Architecture & Technology Stack

### Monorepo Structure
- **Build System**: Turborepo 2.5.5 with dependency-aware builds and caching
- **Package Manager**: pnpm 10.14.0 with workspaces
- **Architecture**: Unified structure with 3 applications (frontend, admin-frontend, backend)
- **Analytics Focus**: All applications integrated with analytics theme and mobile optimizations

### Frontend Stack
- **Framework**: Next.js 15.5.0 with App Router and analytics integration
- **React**: 19.1.0 with Server Components and Server Actions
- **UI**: Radix UI + Tailwind CSS 4.0.15 with analytics theme
- **State**: Zustand + SWR for data fetching and analytics state management
- **Forms**: React Hook Form + Zod validation
- **Auth**: Multi-provider OIDC with Firebase integration
- **Mobile**: Enhanced touch interactions and performance optimizations

### Backend Stack - **Now Fully Diesel-Powered**
- **Language**: Rust (Edition 2021)
- **Framework**: Axum 0.7 with WebSocket support for real-time analytics
- **Database**: PostgreSQL with **Diesel ORM 2.2** and automated migrations
- **Connection Pool**: bb8-diesel for high-performance connection management
- **Cache**: Redis for session/performance data and analytics caching
- **Auth**: Multi-provider JWT with role-based access control
- **EPS Analytics Engine**: Production-ready stock analysis with TradingView data integration and real-time caching
- **Security System**: Complete ML-powered brute force detection, security alerts, and audit systems
- **Notification System**: Multi-channel notification delivery with tracking and analytics
- **Architecture**: Domain-driven design with clean architecture patterns and comprehensive security framework

## Essential Development Commands

### Quick Start
```bash
# Install dependencies
pnpm install

# Start development (choose one)
pnpm dev             # All applications (Frontend + Admin + Backend) - RECOMMENDED ✅
pnpm dev:all         # Alternative command for all applications
pnpm docker:dev      # Full environment with HTTPS

# Individual applications
pnpm dev:frontend    # Trading platform (3000)
pnpm dev:admin       # Admin dashboard (3001)
pnpm dev:backend     # Rust API server (8080) with Diesel

# Troubleshooting: If you encounter client-side JavaScript errors or cache issues
# Kill all processes and restart clean:
pkill -f "pnpm dev" && pnpm dev
```

### Diesel-Specific Commands ✅ **Production Ready**
```bash
# Install Diesel CLI (required for development)
cargo install diesel_cli --no-default-features --features postgres

# Database setup and migrations
diesel setup                                    # Initialize Diesel
cargo run --bin migrate up --features cli-tools # Run migrations  
cargo run --bin migrate status --features cli-tools # Check migration status

# Schema management
diesel print-schema > src/infra/db/diesel/schema.rs # Regenerate schema

# Development commands (no SQLX_OFFLINE needed - fully Diesel)
cargo run                                       # Run backend
cargo test                                      # Run tests
cargo build --release                          # Production build
```

### Build Commands
```bash
# Build individual components
pnpm build:apps      # Build frontend + admin apps  
pnpm build:backend   # Build Rust backend with Diesel
pnpm build:frontend  # Build frontend only
pnpm build:admin     # Build admin only

# Full build (apps + backend)
pnpm build
```

### Testing Commands - **Updated for Diesel**
```bash
# Run all tests
pnpm test

# Specific test types
pnpm test:unit       # Jest unit tests
pnpm test:e2e        # Playwright E2E tests
pnpm test:watch      # Watch mode for development

# Backend tests with Diesel (from apps/backend/)
SQLX_OFFLINE=true cargo test                    # All Rust tests with Diesel
SQLX_OFFLINE=true cargo test --test unit_tests
SQLX_OFFLINE=true cargo test --test integration_tests
SQLX_OFFLINE=true cargo test infra::db::diesel  # Test Diesel repositories
```

### Quality Assurance
```bash
pnpm lint            # ESLint check
pnpm lint:fix        # Auto-fix issues
pnpm type-check      # TypeScript compilation check
pnpm format          # Prettier formatting
```

## Test-Driven Development (TDD) with Diesel

### 🔄 TDD Workflow

1. **🔴 Red**: Write a failing test first
2. **🟢 Green**: Write minimal code to make test pass
3. **🟡 Refactor**: Improve code while keeping tests passing

### 🧪 Testing Commands by Layer

#### Frontend (Jest + React Testing Library)
```bash
pnpm test:watch      # TDD watch mode
pnpm test:unit       # All unit tests
pnpm test:coverage   # Coverage report
```

#### E2E (Playwright)
```bash
pnpm test:e2e        # Full E2E suite
pnpm test:e2e:ui     # Interactive mode
pnpm test:e2e:debug  # Debug mode
```

#### Backend (Rust with Diesel) ✅ **Production Ready**
```bash
# From apps/backend/ - All tests now use Diesel (no SQLX_OFFLINE needed)
cargo test --watch           # TDD watch mode with Diesel
cargo test -- --nocapture    # Detailed output with logs
cargo test integration       # Integration tests  
cargo test notification      # Test notification system
cargo test security          # Test security components
cargo test domain            # Test domain layer
cargo test infrastructure    # Test infrastructure layer
```

### ✅ TDD Best Practices

| Practice | Description |
|----------|-------------|
| **Test First** | Always write tests before implementation |
| **Behavior Focus** | Test what the code does, not how |
| **Independence** | Tests should not depend on each other |
| **Descriptive Names** | Test names should explain expected behavior |
| **Mock Dependencies** | Isolate units under test |
| **Coverage Goals** | Maintain >80% coverage for critical paths |

### 📝 Testing Patterns

#### React Component Testing
```typescript
// Pattern: Test user interactions and state
describe('LoginForm', () => {
  it('should show validation error for invalid email', () => {
    render(<LoginForm />);
    fireEvent.blur(screen.getByLabelText('Email'));
    expect(screen.getByText('Invalid email')).toBeInTheDocument();
  });
});
```

#### Rust API Testing with Diesel ✅ **Updated**
```rust
// Pattern: Test domain logic with Diesel repositories
#[tokio::test]
async fn should_create_notification_with_diesel() {
    let pool = create_test_pool().await;
    let repo = PostgresNotificationRepo::new(pool);
    
    let notification = create_test_notification();
    let result = repo.create_notification(&notification).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string().len(), 36); // UUID length
}
```

## Authentication Architecture - **Enhanced with Diesel**

### Authentication Flow
1. **Custom JWT System** handles OAuth (Google) and credential login
2. **Frontend** receives session tokens and user data  
3. **Backend** validates JWT tokens for API requests using Diesel-backed session storage
4. **Role System** determines user permissions based on simple role-based access control (admin/user/guest) stored in Diesel models

### Role-Based Access Control
- **`admin`**: Full system access with all features: EPS analytics, data export, real-time data, profile management, notifications, billing, advanced filters, plus user management and security monitoring
- **`user`**: Premium access with full feature set: EPS analytics, data export, real-time data, profile management, notifications, billing, and advanced filters
- **`guest`**: Basic read-only access to EPS analytics viewing only

### Session Management - **Now Diesel-Backed**
- **Frontend Sessions**: Custom JWT system handles session state
- **API Authentication**: JWT tokens in Authorization headers validated against Diesel models
- **Firebase Sessions**: Firebase session data stored in `firebase_sessions` table via Diesel
- **Cross-App Auth**: Shared session validation via Diesel-powered backend

## API Communication Patterns

### Frontend ↔ Backend Communication
- **Unified API Client**: Integrated API client handles all requests to Diesel-powered backend
- **Server Actions**: Next.js Server Actions for form submissions
- **SWR Integration**: Data fetching with caching and revalidation
- **WebSocket**: Real-time data updates for trading interface and notifications

### Error Handling Strategy
- **Network Errors**: Automatic retry with exponential backoff
- **Auth Errors**: Redirect to login page
- **Validation Errors**: Display inline form errors
- **Database Errors**: Diesel compile-time validation prevents most issues
- **Server Errors**: Show user-friendly error messages

## Database Patterns - **Fully Diesel-Powered**

### Migration Workflow ✅ **Updated**
```bash
# From apps/backend/ - All migrations now use Diesel
diesel migration generate migration_name         # Create migration
diesel migration run                            # Apply migrations
diesel migration revert                         # Rollback migrations
cargo run --bin migrate up --features cli-tools # Alternative migration runner
cargo run --bin migrate status --features cli-tools # Check migration status
```

### Repository Pattern with Diesel ✅ **New**
- **Domain Layer**: Pure business logic with domain entities
- **Infrastructure Layer**: Diesel repository implementations
- **Models**: Diesel-generated models with proper type mapping
- **Schema**: Auto-generated Diesel schema from database
- **Connection Pool**: bb8-diesel connection management
- **Dependency Injection**: AppContainer manages Diesel repositories

### Diesel Development Patterns ✅ **New**
```rust
// Repository implementation with Diesel
impl NotificationRepository for PostgresNotificationRepo {
    async fn create_notification(&self, notification: &DomainNotification) -> Result<Uuid, NotificationError> {
        let mut conn = self.pool.get().await?;
        
        let new_notification = NewDieselNotification::from(notification);
        
        let result = diesel::insert_into(notifications::table)
            .values(&new_notification)
            .execute(&mut conn)
            .await?;
            
        Ok(new_notification.id)
    }
}
```

## Performance Optimization

### Next.js Optimization
- **Bundle Analysis**: `pnpm analyze` for bundle size monitoring
- **Code Splitting**: Automatic chunk optimization configured
- **Caching Strategy**: Multi-tier caching (static, dynamic, API)
- **Image Optimization**: AVIF/WebP with long-term caching

### Backend Performance - **Enhanced with Diesel**
- **Connection Pooling**: bb8-diesel connection pool management with configurable sizes
- **Compile-time Queries**: Diesel provides zero-overhead query compilation
- **Prepared Statements**: Automatic statement preparation and caching
- **Redis Caching**: Session and query result caching with Diesel-powered cache invalidation
- **Async Processing**: Tokio for concurrent request handling with Diesel async support

## Analytics & Mobile Development

### Analytics Development Guidelines
- **Theme Integration**: All applications use unified analytics theme
- **Mobile-First**: Touch interactions and responsive design throughout
- **TypeScript**: Comprehensive type definitions for analytics data
- **Real-time Updates**: WebSocket integration for live market data
- **Diesel Analytics**: EPS growth analytics powered by type-safe Diesel queries

### Core Implemented Features - **Enhanced with Diesel**
- **EPS Growth Analytics**: Complete stock ranking system with multi-dimensional filtering and TradingView data, powered by Diesel queries
- **Notification System**: Multi-channel notification delivery with Diesel-backed tracking and analytics ✅ **New**
- **Enterprise Security**: ML-powered brute force detection, security alerting, and comprehensive compliance framework with Diesel audit trails
- **Real-time Performance**: Advanced caching, WebSocket support, and optimized Diesel database queries
- **Production Authentication**: Multi-provider OIDC, role-based access control, and secure Diesel-backed session management
- **Admin Dashboard**: User management, security monitoring, and system analytics with Diesel-powered data access

## Development Best Practices

### Code Organization
- **Feature-Based Structure**: Group related files together (analytics/, auth/, notifications/, security/)
- **Analytics-First Design**: All components designed with analytics integration in mind
- **Mobile-Optimized Components**: Touch interactions and responsive layouts
- **Separation of Concerns**: Keep business logic separate from UI
- **TypeScript First**: Leverage type safety throughout with comprehensive analytics types
- **Diesel First**: Use Diesel for all database operations with compile-time safety
- **Error Boundaries**: Graceful error handling in React components

### Diesel Development Guidelines ✅ **New**
- **Schema-First**: Always update database schema through migrations
- **Type Safety**: Leverage Diesel's compile-time query validation
- **Repository Pattern**: Implement clean repository interfaces with Diesel
- **Model Conversion**: Implement proper conversion between Diesel and domain models
- **Connection Management**: Use bb8-diesel connection pooling properly
- **Testing**: Write comprehensive tests for all Diesel repositories

### Git Workflow
- **Pre-commit Hooks**: Husky enforces linting and type checking
- **Conventional Commits**: Follow commit message standards
- **Quality Gates**: All tests must pass before merging (including Diesel tests)
- **Migration Safety**: Always test migration rollbacks

### 📦 Container Environment (Apple Silicon Optimized)

| Engine | Performance | Use Case |
|--------|-------------|----------|
| **OrbStack** | 15x faster | Recommended for development |
| **Podman** | Enterprise | Daemonless, security-focused |
| **Docker Desktop** | Standard | Legacy support |

### 🚀 Container Commands

```bash
# Environment management
pnpm docker:dev              # Start development environment
pnpm docker:dev:down         # Stop development
pnpm docker:dev:logs         # View logs

# Performance optimization
./scripts/check-container-engine.sh    # Check current engine
./scripts/benchmark-builds.sh          # Performance analysis
./scripts/orbstack-migration-guide.md  # Migration guide

# Deployment with Diesel
./scripts/build.sh           # Build optimized containers with Diesel
./scripts/deploy-cloudrun.sh # Deploy to Google Cloud Run
./scripts/clean.sh           # Clean build artifacts
```

## 🔧 Troubleshooting Guide

### 🛠️ Build Issues

| Problem | Solution |
|---------|----------|
| **Dependencies** | `pnpm build:apps` before development |
| **Type Errors** | Check `tsconfig.json` path mappings |
| **Cache Issues** | `pnpm clean && pnpm clean:cache` |
| **Module Resolution** | Verify `package.json` workspace configs |
| **Diesel Errors** ✅ **New** | Run `SQLX_OFFLINE=true` for all cargo commands |
| **Schema Issues** ✅ **New** | Regenerate with `diesel print-schema` |

### 🔐 Authentication Issues

| Problem | Solution |
|---------|----------|
| **Session Loss** | Check custom JWT configuration and Diesel session storage |
| **JWT Validation** | Verify backend token validation with Diesel session queries |
| **CORS Errors** | Check backend CORS for frontend domains |
| **Redirect Loops** | Verify `APP_URL`/`ADMIN_URL` in env files |
| **Firebase Sessions** ✅ **New** | Check `firebase_sessions` table via Diesel |

### 🌐 Development Environment

| Problem | Solution |
|---------|----------|
| **Port Conflicts** | Frontend (3000), Admin (3001), Backend (8080) |
| **Database Connection** | Ensure PostgreSQL running & Diesel configured properly |
| **Environment Variables** | Check `.env` files are properly set, include `SQLX_OFFLINE=true` |
| **Domain Resolution** | Verify `/etc/hosts` has `*.epsx.io` entries |
| **Diesel Setup** ✅ **New** | Run `diesel setup` for first-time configuration |

### 📦 Container Performance

| Issue | Command | Expected Result |
|-------|---------|----------------|
| **Slow Startup** | `./scripts/check-container-engine.sh` | Engine status & recommendations |
| **Build Performance** | `./scripts/benchmark-builds.sh` | Performance metrics |
| **OrbStack Migration** | Follow `./scripts/orbstack-migration-guide.md` | 15x performance improvement |
| **Cache Cleanup** | `rm -rf /tmp/.buildx-cache-orbstack` | Fresh build cache |
| **Diesel Container** ✅ **New** | `docker exec epsx-backend diesel migration run` | Apply migrations in container |

### 🚀 Performance Optimization Workflow

1. **Baseline**: `./scripts/benchmark-builds.sh`
2. **Migrate**: Install OrbStack (see migration guide)
3. **Verify**: Re-run benchmarks
4. **Monitor**: `./scripts/analyze-performance.sh`

## 📊 EPS Analytics Development Patterns - **Enhanced with Diesel**

### 📊 Component Architecture ✅ **Updated**

```typescript
// Pattern: EPS Analytics component with Diesel-powered backend  
// components/analytics/EPSAnalyticsCard.tsx
export function EPSAnalyticsCard({ 
  epsData, 
  onFilter, 
  mobile = false,
  realTime = true 
}) {
  // 1. EPS-focused data visualization
  // 2. Real-time TradingView data integration via Diesel-backed API
  // 3. Advanced filtering capabilities with type-safe queries
  // 4. Mobile-optimized performance
}
```

### 📱 Mobile Performance Best Practices

| Practice | Implementation |
|----------|----------------|
| **Touch Interactions** | Enhanced touch wrappers for mobile optimization |
| **Responsive Design** | Mobile/tablet/desktop compatibility |
| **React 19 Features** | Concurrent features for smooth scrolling |
| **Data Loading** | SWR with Diesel-powered API endpoints |
| **Image Optimization** | Next.js Image with AVIF/WebP |
| **Code Splitting** | Dynamic imports for analytics modules |

### 📈 Real-time EPS Data Integration - **Diesel-Powered** ✅ **Updated**

```typescript
// Pattern: Real-time EPS analytics data flow with Diesel backend
const useEPSAnalyticsData = (filters: EPSFilters) => {
  const { data, error } = useSWR(
    `/api/v1/analytics/eps-rankings`, // Diesel-powered endpoint
    fetcher,
    { refreshInterval: 30000 } // 30-second updates for EPS data
  );
  
  // WebSocket for live EPS updates from Diesel-backed streaming
  useWebSocket(`/ws/eps-updates`, {
    onMessage: (event) => {
      // Update EPS ranking state with TradingView data
      // Data fetched via type-safe Diesel queries
    }
  });
};
```

### 🎨 EPS Analytics Theme Integration

- **Color System**: EPS growth-focused color palette (green for growth, red for decline)
- **Typography**: Optimized for financial data readability and number formatting
- **Charts**: Recharts with custom EPS visualization themes and TradingView styling
- **Icons**: Lucide icons for financial metrics and filtering controls
- **Animations**: Smooth transitions for ranking updates and data loading states

### 🚀 Production Development Workflow - **Diesel-Enhanced**

1. **Security First**: All components include proper authentication and Diesel-backed audit logging
2. **EPS-Focused**: Components designed specifically for stock earnings analysis with Diesel queries
3. **Real-time Ready**: Built with TradingView WebSocket integration and Diesel-powered caching
4. **Performance**: Optimized for large datasets with pagination, virtualization, and Diesel connection pooling
5. **Compliance**: GDPR/SOX/HIPAA compliance built into data handling via Diesel audit trails and user interactions
6. **Notification Integration**: Real-time notifications via Diesel-powered notification system ✅ **New**

## 🔄 Migration Status Summary

### ✅ Completed (100% of total migration):
- **Database Schema**: All production tables migrated to Diesel with consolidated schema
- **Core Models**: Complete Diesel model definitions (210+ Rust files)
- **Repository Layer**: Full Diesel repository implementation with bb8 connection pooling
- **Notification System**: Complete async CRUD operations with Diesel
- **Security System**: Complete models and repositories for security events, alerts, and audit trails  
- **Authentication**: Firebase integration with Diesel-backed session storage
- **Migration Infrastructure**: Production-ready migration system (2025-01-15)
- **Testing Framework**: Updated test suite with comprehensive Diesel integration
- **Development Workflow**: All commands optimized for Diesel (no SQLX_OFFLINE flags needed)
- **Production Deployment**: Successfully deployed with Diesel ORM

### 🚀 Current Status:
- **Architecture**: Clean Architecture with Diesel at infrastructure layer
- **Performance**: Optimized connection pooling and compile-time query validation
- **Type Safety**: Complete compile-time SQL validation prevents runtime errors
- **Maintainability**: Schema auto-generation and migration management
- **Production Ready**: Successfully running in production environment

---

**🎉 EPSX has successfully completed 100% migration from SQLx to Diesel ORM, providing enhanced type safety, performance, and maintainability with full production deployment!**

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.

      
      IMPORTANT: this context may or may not be relevant to your tasks. You should not respond to this context unless it is highly relevant to your task.