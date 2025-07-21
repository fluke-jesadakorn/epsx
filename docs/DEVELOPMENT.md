# Development Guide

This guide provides comprehensive instructions for developing in the EPSX monorepo.

## Table of Contents

1. [Setup and Installation](#setup-and-installation)
2. [Project Structure](#project-structure)
3. [Development Workflow](#development-workflow)
4. [Package Development](#package-development)
5. [Application Development](#application-development)
6. [Testing Strategy](#testing-strategy)
7. [Deployment](#deployment)
8. [Troubleshooting](#troubleshooting)

## Setup and Installation

### Prerequisites

Ensure you have the following installed:

- **Node.js** >= 18.0.0
- **PNPM** >= 8.0.0
- **Git**
- **Docker** (optional, for containerized development)
- **Rust** (for backend development)

### Installation Steps

```bash
# Clone the repository
git clone https://github.com/fluke-jesadakorn/epsx.git
cd epsx

# Install dependencies
pnpm install

# Build packages first
pnpm build:packages

# Setup environment variables
cp .env.example .env
# Edit .env with your configuration
```

## Project Structure

### Monorepo Organization

```
epsx/
├── apps/                   # Applications
│   ├── frontend/          # Main trading platform (Next.js)
│   ├── admin-frontend/    # Admin dashboard (Next.js)
│   └── backend/           # API server (Rust)
├── packages/              # Shared packages
│   ├── types/            # TypeScript type definitions
│   ├── utils/            # Utility functions
│   ├── ui/               # React UI components
│   ├── config/           # ESLint, Prettier configs
│   ├── auth/             # Authentication utilities
│   ├── api-client/       # API client library
│   └── shared/           # Shared business logic
├── docs/                 # Documentation
├── scripts/              # Build and deployment scripts
└── [config files]       # Monorepo configuration
```

### Package Naming Convention

All packages follow the `@epsx/package-name` naming convention:

- `@epsx/frontend` - Frontend application
- `@epsx/admin-frontend` - Admin application
- `@epsx/types` - Shared types
- `@epsx/utils` - Utilities
- `@epsx/ui` - UI components
- etc.

## Development Workflow

### 1. Starting Development

```bash
# Start all development servers
pnpm dev

# Or start specific applications
pnpm dev:frontend    # Frontend only
pnpm dev:admin       # Admin only
pnpm dev:backend     # Backend only
```

### 2. Package Development

When working on shared packages:

```bash
# Start package development (watch mode)
pnpm dev:packages

# Or work on specific package
cd packages/your-package
pnpm dev
```

### 3. Code Quality

Before committing, ensure code quality:

```bash
# Check everything
pnpm lint
pnpm type-check
pnpm test
pnpm format:check

# Fix issues
pnpm lint:fix
pnpm format
```

## Package Development

### Creating a New Package

1. Create package directory:

```bash
mkdir packages/new-package
cd packages/new-package
```

2. Initialize package.json:

```json
{
  "name": "@epsx/new-package",
  "version": "0.1.0",
  "private": true,
  "main": "./dist/index.js",
  "module": "./dist/index.mjs",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "scripts": {
    "build": "tsup",
    "build:packages": "tsup",
    "dev": "tsup --watch",
    "dev:packages": "tsup --watch",
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "type-check": "tsc --noEmit",
    "clean": "rm -rf dist .turbo"
  },
  "devDependencies": {
    "@epsx/config": "workspace:*",
    "tsup": "^8.0.2",
    "typescript": "^5.4.2"
  }
}
```

3. Create tsconfig.json:

```json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "outDir": "dist",
    "declarationDir": "dist"
  },
  "include": ["src/**/*"],
  "exclude": ["dist", "node_modules"]
}
```

4. Create tsup.config.ts:

```typescript
import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  sourcemap: true,
  clean: true,
  splitting: false,
  treeshake: true,
});
```

### Package Best Practices

1. **Export Everything from index.ts**
2. **Use Proper TypeScript Types**
3. **Include README.md**
4. **Add Unit Tests**
5. **Follow Semantic Versioning**

## Application Development

### Frontend Development

The frontend is a Next.js application with:

- **App Router** (Next.js 13+)
- **TypeScript**
- **Tailwind CSS**
- **Radix UI Components**
- **Firebase Authentication**

#### Key Directories:

```
apps/frontend/
├── app/                # App Router pages
├── components/         # React components
├── lib/               # Utility libraries
├── hooks/             # Custom React hooks
├── context/           # React contexts
├── services/          # API services
├── types/             # Local type definitions
└── utils/             # Utility functions
```

#### Development Commands:

```bash
# Start frontend development
pnpm dev:frontend

# Build frontend
pnpm build:frontend

# Run tests
pnpm test:unit
pnpm test:e2e
```

### Admin Frontend

Similar structure to the main frontend but focused on administrative tasks.

### Backend Development

The backend is built with Rust and provides:

- **RESTful API**
- **WebSocket connections**
- **Database integration**
- **Authentication middleware**

## Testing Strategy

### Unit Testing

- **Framework:** Jest
- **Location:** `*.test.ts` files alongside source code
- **Command:** `pnpm test:unit`

### Integration Testing

- **Framework:** Jest
- **Location:** `__tests__/` directories
- **Command:** `pnpm test`

### End-to-End Testing

- **Framework:** Playwright
- **Location:** `e2e/` directories in applications
- **Command:** `pnpm test:e2e`

### Testing Best Practices

1. Write tests for all new features
2. Maintain test coverage above 80%
3. Use descriptive test names
4. Test edge cases and error conditions
5. Mock external dependencies

## Deployment

### Build Process

```bash
# Build everything
pnpm build

# Build specific targets
pnpm build:packages
pnpm build:frontend
pnpm build:admin
pnpm build:backend
```

### Docker Deployment

```bash
# Build Docker images
make docker-build

# Deploy to development
pnpm docker:dev

# Deploy to production
pnpm docker:prod
```

### Environment Configuration

Ensure proper environment variables are set for each environment:

- **Development:** `.env.local`
- **Staging:** Environment-specific configuration
- **Production:** Secure environment variables

## Troubleshooting

### Common Issues

#### 1. Build Failures

```bash
# Clean everything and rebuild
pnpm clean
pnpm install
pnpm build:packages
pnpm build
```

#### 2. Type Errors

```bash
# Check TypeScript configuration
pnpm type-check

# Rebuild packages
pnpm build:packages
```

#### 3. Dependency Issues

```bash
# Clear PNPM cache
pnpm store prune

# Reinstall dependencies
rm -rf node_modules
pnpm install
```

#### 4. Docker Issues

```bash
# Rebuild containers
docker-compose down
docker-compose build --no-cache
docker-compose up
```

### Getting Help

1. Check this documentation
2. Review error messages carefully
3. Check the Issues tab in the repository
4. Ask team members for assistance

### Performance Tips

1. **Use Turbo Cache:** Commands are cached automatically
2. **Incremental Builds:** Only modified packages rebuild
3. **Parallel Execution:** Multiple tasks run simultaneously
4. **Watch Mode:** Use for active development

### Code Organization Tips

1. **Keep packages focused:** Each package should have a single responsibility
2. **Use barrel exports:** Export everything through index files
3. **Consistent naming:** Follow established conventions
4. **Document APIs:** Add JSDoc comments for public APIs
5. **Type everything:** Use TypeScript for all code

## Contributing

When contributing to the monorepo:

1. Create feature branches for new work
2. Write tests for new functionality
3. Ensure all quality checks pass
4. Update documentation as needed
5. Create clear, descriptive commit messages
6. Submit pull requests for review

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
Scopes: `frontend`, `admin`, `backend`, `types`, `utils`, etc.

Example:

```
feat(frontend): add stock analysis dashboard

- Implement real-time stock data visualization
- Add filtering and sorting capabilities
- Include responsive design for mobile devices

Closes #123
```
