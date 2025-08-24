# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EPSX is a production-ready analytics platform built with modern technologies, featuring enterprise-grade security and mobile-first performance. The architecture consists of:

- **Frontend** (Port 3000): Next.js 15.5.0 + React 19.1.0 with EPS analytics dashboard
- **Admin Frontend** (Port 3001): Administrative dashboard for user/IAM management and system monitoring
- **Backend** (Port 8080): High-performance Rust API server with EPS analytics engine, TradingView integration, and enterprise security
- **Unified Monorepo**: Streamlined structure optimized for analytics performance and development velocity

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

### Backend Stack  
- **Language**: Rust (Edition 2021)
- **Framework**: Axum 0.7 with WebSocket support for real-time analytics
- **Database**: PostgreSQL with SQLx ORM and migrations for analytics data
- **Cache**: Redis for session/performance data and analytics caching
- **Auth**: Multi-provider JWT with role-based access control
- **EPS Analytics Engine**: Production-ready stock analysis with TradingView data integration and real-time caching
- **Enterprise Security**: Complete GDPR/SOX/HIPAA compliance, brute force detection, and audit systems
- **Architecture**: Domain-driven design with clean architecture patterns and comprehensive security framework

## Essential Development Commands

### Quick Start
```bash
# Install dependencies
pnpm install

# Start development (choose one)
pnpm dev             # Frontend + Admin only
pnpm dev:all         # All applications including backend
pnpm docker:dev      # Full environment with HTTPS

# Individual applications
pnpm dev:frontend    # Trading platform (3000)
pnpm dev:admin       # Admin dashboard (3001)
pnpm dev:backend     # Rust API server (8080)
```

### Build Commands
```bash
# Build individual components
pnpm build:apps      # Build frontend + admin apps  
pnpm build:backend   # Build Rust backend
pnpm build:frontend  # Build frontend only
pnpm build:admin     # Build admin only

# Full build (apps + backend)
pnpm build
```

### Testing Commands
```bash
# Run all tests
pnpm test

# Specific test types
pnpm test:unit       # Jest unit tests
pnpm test:e2e        # Playwright E2E tests
pnpm test:watch      # Watch mode for development

# Backend tests (from apps/backend/)
cargo test           # All Rust tests
cargo test --test unit_tests
cargo test --test integration_tests
```

### Quality Assurance
```bash
pnpm lint            # ESLint check
pnpm lint:fix        # Auto-fix issues
pnpm type-check      # TypeScript compilation check
pnpm format          # Prettier formatting
```

## Test-Driven Development (TDD)

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

#### Backend (Rust)
```bash
# From apps/backend/
cargo test --watch           # TDD watch mode
cargo test -- --nocapture    # Detailed output
cargo test integration       # Integration tests
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

#### Rust API Testing
```rust
// Pattern: Test domain logic with clear scenarios
#[tokio::test]
async fn should_create_user_with_valid_data() {
    let result = create_user(valid_user_data()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().email, "test@example.com");
}
```

## Authentication Architecture

### Authentication Flow
1. **Custom JWT System** handles OAuth (Google) and credential login
2. **Frontend** receives session tokens and user data
3. **Backend** validates JWT tokens for API requests
4. **IAM System** determines user permissions based on profiles

### IAM Profiles & Permissions
- `user-basic-001`: Basic EPS analytics access with standard filtering
- `user-premium-002`: Premium analytics features with advanced filtering and export capabilities
- `moderator-standard-003`: User management and content moderation capabilities
- `admin-full-004`: Full system access including security monitoring and compliance management

### Session Management
- **Frontend Sessions**: Custom JWT system handles session state
- **API Authentication**: JWT tokens in Authorization headers
- **Cross-App Auth**: Shared session validation via backend

## API Communication Patterns

### Frontend ↔ Backend Communication
- **Unified API Client**: Integrated API client handles all requests
- **Server Actions**: Next.js Server Actions for form submissions
- **SWR Integration**: Data fetching with caching and revalidation
- **WebSocket**: Real-time data updates for trading interface

### Error Handling Strategy
- **Network Errors**: Automatic retry with exponential backoff
- **Auth Errors**: Redirect to login page
- **Validation Errors**: Display inline form errors
- **Server Errors**: Show user-friendly error messages

## Database Patterns

### Migration Workflow
```bash
# From apps/backend/
sqlx migrate add create_users_table
sqlx migrate run  # Apply migrations
```

### Repository Pattern
- **Domain Layer**: Pure business logic
- **Infrastructure Layer**: Database implementations
- **Dependency Injection**: AppContainer manages dependencies

## Performance Optimization

### Next.js Optimization
- **Bundle Analysis**: `pnpm analyze` for bundle size monitoring
- **Code Splitting**: Automatic chunk optimization configured
- **Caching Strategy**: Multi-tier caching (static, dynamic, API)
- **Image Optimization**: AVIF/WebP with long-term caching

### Backend Performance
- **Connection Pooling**: SQLx connection pool management
- **Redis Caching**: Session and query result caching
- **Async Processing**: Tokio for concurrent request handling

## Analytics & Mobile Development

### Analytics Development Guidelines
- **Theme Integration**: All applications use unified analytics theme
- **Mobile-First**: Touch interactions and responsive design throughout
- **TypeScript**: Comprehensive type definitions for analytics data
- **Real-time Updates**: WebSocket integration for live market data

### Core Implemented Features
- **EPS Growth Analytics**: Complete stock ranking system with multi-dimensional filtering and TradingView data
- **Enterprise Security**: ML-powered brute force detection, security alerting, and comprehensive compliance framework
- **Real-time Performance**: Advanced caching, WebSocket support, and optimized database queries
- **Production Authentication**: Multi-provider OIDC, IAM profiles, and secure session management
- **Admin Dashboard**: User management, security monitoring, and system analytics

## Development Best Practices

### Code Organization
- **Feature-Based Structure**: Group related files together (analytics/, auth/, touch/)
- **Analytics-First Design**: All components designed with analytics integration in mind
- **Mobile-Optimized Components**: Touch interactions and responsive layouts
- **Separation of Concerns**: Keep business logic separate from UI
- **TypeScript First**: Leverage type safety throughout with comprehensive analytics types
- **Error Boundaries**: Graceful error handling in React components

### Git Workflow
- **Pre-commit Hooks**: Husky enforces linting and type checking
- **Conventional Commits**: Follow commit message standards
- **Quality Gates**: All tests must pass before merging

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

# Deployment
./scripts/build.sh           # Build optimized containers
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

### 🔐 Authentication Issues

| Problem | Solution |
|---------|----------|
| **Session Loss** | Check custom JWT configuration |
| **JWT Validation** | Verify backend token validation |
| **CORS Errors** | Check backend CORS for frontend domains |
| **Redirect Loops** | Verify `APP_URL`/`ADMIN_URL` in env files |

### 🌐 Development Environment

| Problem | Solution |
|---------|----------|
| **Port Conflicts** | Frontend (3000), Admin (3001), Backend (8080) |
| **Database Connection** | Ensure PostgreSQL running & configured |
| **Environment Variables** | Check `.env` files are properly set |
| **Domain Resolution** | Verify `/etc/hosts` has `*.epsx.io` entries |

### 📦 Container Performance

| Issue | Command | Expected Result |
|-------|---------|----------------|
| **Slow Startup** | `./scripts/check-container-engine.sh` | Engine status & recommendations |
| **Build Performance** | `./scripts/benchmark-builds.sh` | Performance metrics |
| **OrbStack Migration** | Follow `./scripts/orbstack-migration-guide.md` | 15x performance improvement |
| **Cache Cleanup** | `rm -rf /tmp/.buildx-cache-orbstack` | Fresh build cache |

### 🚀 Performance Optimization Workflow

1. **Baseline**: `./scripts/benchmark-builds.sh`
2. **Migrate**: Install OrbStack (see migration guide)
3. **Verify**: Re-run benchmarks
4. **Monitor**: `./scripts/analyze-performance.sh`

## 📊 EPS Analytics Development Patterns

### 📊 Component Architecture

```typescript
// Pattern: EPS Analytics component structure  
// components/analytics/EPSAnalyticsCard.tsx
export function EPSAnalyticsCard({ 
  epsData, 
  onFilter, 
  mobile = false,
  realTime = true 
}) {
  // 1. EPS-focused data visualization
  // 2. Real-time TradingView data integration
  // 3. Advanced filtering capabilities
  // 4. Mobile-optimized performance
}
```

### 📱 Mobile Performance Best Practices

| Practice | Implementation |
|----------|----------------|
| **Touch Interactions** | Enhanced touch wrappers for mobile optimization |
| **Responsive Design** | Mobile/tablet/desktop compatibility |
| **React 19 Features** | Concurrent features for smooth scrolling |
| **Data Loading** | SWR with proper loading states |
| **Image Optimization** | Next.js Image with AVIF/WebP |
| **Code Splitting** | Dynamic imports for analytics modules |

### 📈 Real-time EPS Data Integration

```typescript
// Pattern: Real-time EPS analytics data flow
const useEPSAnalyticsData = (filters: EPSFilters) => {
  const { data, error } = useSWR(
    `/api/v1/analytics/eps-rankings`,
    fetcher,
    { refreshInterval: 30000 } // 30-second updates for EPS data
  );
  
  // WebSocket for live EPS updates
  useWebSocket(`/ws/eps-updates`, {
    onMessage: (event) => {
      // Update EPS ranking state with TradingView data
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

### 🚀 Production Development Workflow

1. **Security First**: All components include proper authentication and audit logging
2. **EPS-Focused**: Components designed specifically for stock earnings analysis
3. **Real-time Ready**: Built with TradingView WebSocket integration and caching
4. **Performance**: Optimized for large datasets with pagination and virtualization
5. **Compliance**: GDPR/SOX/HIPAA compliance built into data handling and user interactions