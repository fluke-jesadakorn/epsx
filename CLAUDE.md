# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EPSX is a production-ready trading platform monorepo built with modern technologies. The architecture consists of:

- **Frontend** (Port 3000): Next.js 15 + React 19 trading platform
- **Admin Frontend** (Port 3001): Administrative dashboard for user/IAM management  
- **Backend** (Port 8080): High-performance Rust API server with Axum framework
- **12 Shared Packages**: Reusable components, utilities, and configurations

## Architecture & Technology Stack

### Monorepo Structure
- **Build System**: Turborepo 2.5.4 with dependency-aware builds and caching
- **Package Manager**: pnpm 10.13.1 with workspaces
- **Dependencies**: Packages must build before apps (`build:packages` → `build:apps`)

### Frontend Stack
- **Framework**: Next.js 15.4.2 with App Router
- **React**: 19.1.0 with Server Components and Server Actions
- **UI**: Radix UI + Tailwind CSS 4.0.15
- **State**: Zustand + SWR for data fetching
- **Forms**: React Hook Form + Zod validation
- **Auth**: NextAuth.js 4.24.11 integration

### Backend Stack  
- **Language**: Rust (Edition 2021)
- **Framework**: Axum 0.7 with WebSocket support
- **Database**: PostgreSQL with SQLx ORM and migrations
- **Cache**: Redis for session/performance data
- **Auth**: JWT tokens with role-based access control
- **Architecture**: Domain-driven design with clean architecture patterns

## Essential Development Commands

### Quick Start
```bash
# Install dependencies
pnpm install

# Start all development servers
pnpm dev  # Frontend + Admin (excludes backend)
pnpm dev:all  # All applications including backend

# Start individual applications
pnpm dev:frontend    # Port 3000
pnpm dev:admin       # Port 3001  
pnpm dev:backend     # Port 8080
pnpm dev:packages    # Watch mode for packages
```

### Build Commands
```bash
# IMPORTANT: Build packages first before apps
pnpm build:packages  # Required before building apps
pnpm build:apps      # Build frontend + admin apps
pnpm build:backend   # Build Rust backend

# Full build (packages → apps → backend)
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

## Test-Driven Development (TDD) Process

### TDD Workflow
1. **Red**: Write a failing test first
2. **Green**: Write minimal code to make the test pass
3. **Refactor**: Improve code while keeping tests passing

### Testing Strategy by Layer

#### Frontend Components (Jest + React Testing Library)
```bash
# Create test file alongside component
# Example: components/Button.tsx → components/Button.test.tsx

# TDD commands
pnpm test:watch      # Immediate feedback during development
pnpm test:unit       # Run unit tests only
```

#### E2E Testing (Playwright) 
```bash
# Test critical user flows
pnpm test:e2e        # Full E2E test suite
pnpm test:e2e:ui     # Interactive test runner
```

#### Backend Testing (Rust)
```bash
# From apps/backend/
cargo test --watch   # Watch mode for TDD
cargo test unit_tests -- --nocapture  # Detailed output
```

### TDD Best Practices
- **Write tests BEFORE implementation**
- **Test behavior, not implementation details**
- **Keep tests focused and independent** 
- **Use descriptive test names** that explain the expected behavior
- **Mock external dependencies** (APIs, databases, services)
- **Maintain test coverage** above 80% for critical paths

### Testing Patterns

#### Component Testing
```typescript
// Always test user interactions and state changes
describe('LoginForm', () => {
  it('should show validation error when email is invalid', () => {
    // Red: Test expected behavior first
    render(<LoginForm />);
    fireEvent.blur(screen.getByLabelText('Email'));
    expect(screen.getByText('Invalid email')).toBeInTheDocument();
  });
});
```

#### API Integration Testing  
```rust
// Test domain logic thoroughly
#[tokio::test] 
async fn test_user_creation_with_valid_data() {
    // Red: Define expected behavior
    let result = create_user(valid_user_data()).await;
    assert!(result.is_ok());
}
```

## Authentication Architecture

### Authentication Flow
1. **NextAuth.js** handles OAuth (Google) and credential login
2. **Frontend** receives session tokens and user data
3. **Backend** validates JWT tokens for API requests
4. **IAM System** determines user permissions based on profiles

### IAM Profiles & Permissions
- `user-basic-001`: Basic trading features
- `user-premium-002`: Premium features + advanced analytics  
- `moderator-standard-003`: User management capabilities
- `admin-full-004`: Full system access

### Session Management
- **Frontend Sessions**: NextAuth.js handles session state
- **API Authentication**: JWT tokens in Authorization headers
- **Cross-App Auth**: Shared session validation via backend

## API Communication Patterns

### Frontend ↔ Backend Communication
- **Unified API Client**: `@epsx/api-client` package handles all requests
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

## Package Development

### Shared Package Guidelines
- **Build Order**: Always build packages before apps
- **TypeScript**: All packages use strict TypeScript configuration  
- **Export Strategy**: ESM/CJS dual exports for compatibility
- **Dependencies**: Use workspace references (`workspace:*`)

### Key Packages
- `@epsx/ui`: Radix UI + Tailwind component library
- `@epsx/api-client`: Unified API client with authentication
- `@epsx/types`: Comprehensive TypeScript definitions
- `@epsx/server-actions`: Shared Next.js Server Actions

## Development Best Practices

### Code Organization
- **Feature-Based Structure**: Group related files together
- **Separation of Concerns**: Keep business logic separate from UI
- **TypeScript First**: Leverage type safety throughout
- **Error Boundaries**: Graceful error handling in React components

### Git Workflow
- **Pre-commit Hooks**: Husky enforces linting and type checking
- **Conventional Commits**: Follow commit message standards
- **Quality Gates**: All tests must pass before merging

### Container Environment (Apple Silicon Optimized)
- **Primary Engine**: OrbStack (recommended for 15x performance improvement)
- **Alternative**: Podman (enterprise-grade, daemonless)
- **Legacy**: Docker Desktop (migrate to OrbStack for better performance)
- **Build Target**: AMD64 containers for Google Cloud Run deployment
- **Performance**: Native ARM64 builds + cross-compilation optimization

### Container Management Commands
```bash
# Check current container engine and performance
./scripts/check-container-engine.sh

# Build optimized containers (OrbStack/Podman/Docker compatible)
./scripts/build.sh

# Benchmark build performance
./scripts/benchmark-builds.sh

# Analyze performance trends
./scripts/analyze-performance.sh

# Deploy to Google Cloud Run
./scripts/deploy-cloudrun.sh

# Clean build artifacts
./scripts/clean.sh
```

### Environment Management
- **Environment Files**: Separate configs for dev/test/prod
- **Container Engines**: OrbStack/Podman/Docker Desktop support
- **Migration Guide**: `./scripts/orbstack-migration-guide.md`
- **Admin Tools**: CLI tools for user management and IAM assignment

## Troubleshooting Common Issues

### Build Issues
- **Package Dependencies**: Run `pnpm build:packages` first
- **Type Errors**: Check `tsconfig.json` path mappings
- **Cache Issues**: Clear with `pnpm clean` and `pnpm clean:cache`

### Authentication Issues  
- **Session Persistence**: Check NextAuth.js configuration
- **JWT Validation**: Verify backend token validation
- **CORS Issues**: Check backend CORS settings for frontend domains

### Development Workflow Issues
- **Port Conflicts**: Frontend (3000), Admin (3001), Backend (8080)
- **Database Connection**: Ensure PostgreSQL is running and configured
- **Environment Variables**: Check `.env` files are properly configured

### Container Performance Issues
- **Slow Container Startup**: Check engine with `./scripts/check-container-engine.sh`
- **Build Performance**: Run `./scripts/benchmark-builds.sh` to identify bottlenecks
- **OrbStack Migration**: Follow `./scripts/orbstack-migration-guide.md` for 15x improvement
- **Cache Issues**: Clear build cache with `rm -rf /tmp/.buildx-cache-orbstack`
- **Platform Issues**: Ensure AMD64 builds for Cloud Run compatibility

### Performance Optimization Workflow
1. **Baseline**: Run `./scripts/benchmark-builds.sh` to establish current performance
2. **Migrate**: Install OrbStack following the migration guide
3. **Verify**: Re-run benchmarks to confirm improvements
4. **Monitor**: Use `./scripts/analyze-performance.sh` for ongoing optimization

When working on this codebase, always follow the TDD process, understand the authentication flow, respect the build dependencies between packages and applications, and use OrbStack for optimal development performance on Apple Silicon.