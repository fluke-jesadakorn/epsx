# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EPSX is a production-ready trading platform built with modern technologies, featuring analytics-focused design and mobile-first performance. The architecture consists of:

- **Frontend** (Port 3000): Next.js 15.5.0 + React 19.1.0 with comprehensive analytics
- **Admin Frontend** (Port 3001): Administrative dashboard for user/IAM management
- **Backend** (Port 8080): High-performance Rust API server with Axum + analytics engine
- **Unified Monorepo**: Streamlined structure optimized for development velocity

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
- **Analytics Engine**: High-performance market data processing and analysis
- **Architecture**: Domain-driven design with clean architecture patterns

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
- `user-basic-001`: Basic trading features
- `user-premium-002`: Premium features + advanced analytics  
- `moderator-standard-003`: User management capabilities
- `admin-full-004`: Full system access

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

### Key Features
- **Analytics Dashboard**: Advanced stock analysis with filtering and visualization
- **Mobile Performance**: Touch gestures, responsive layouts, and performance optimizations
- **Real-time Data**: Live market updates and streaming analytics
- **Authentication**: Multi-provider OIDC with Firebase integration

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

## 📊 Analytics Development Patterns

### 📊 Component Architecture

```typescript
// Pattern: Analytics-focused component structure
// components/analytics/AnalyticsCard.tsx
export function AnalyticsCard({ 
  data, 
  onFilter, 
  mobile = false,
  realTime = true 
}) {
  // 1. Mobile-first responsive design
  // 2. Real-time data integration
  // 3. Touch-optimized interactions
  // 4. Accessibility compliance
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

### 📈 Real-time Data Integration

```typescript
// Pattern: Real-time analytics data flow
const useAnalyticsData = (symbol: string) => {
  const { data, error } = useSWR(
    `/api/analytics/${symbol}`,
    fetcher,
    { refreshInterval: 1000 } // Real-time updates
  );
  
  // WebSocket for live market data
  useWebSocket(`/ws/market/${symbol}`, {
    onMessage: (event) => {
      // Update analytics state
    }
  });
};
```

### 🎨 Analytics Theme Integration

- **Color System**: Use analytics-focused color palette
- **Typography**: Optimized for data readability
- **Charts**: Recharts with custom analytics themes
- **Icons**: Lucide icons for consistency
- **Animations**: Framer Motion for smooth transitions

### 🚀 Development Workflow

1. **TDD First**: Write tests before analytics components
2. **Mobile-First**: Design for mobile, enhance for desktop
3. **Real-time Ready**: Build with WebSocket integration in mind
4. **Performance**: Use OrbStack for 15x faster development
5. **Analytics Focus**: Every component should support analytics integration