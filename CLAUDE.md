# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

```bash
# Development
pnpm dev              # Start frontend (3000) + admin (3001)
pnpm dev:all          # Start all including backend
pnpm dev:frontend     # Frontend only on port 3000
pnpm dev:admin        # Admin only on port 3001
pnpm dev:packages     # Watch mode for all packages

# Building
pnpm build            # Build everything
pnpm build:packages   # Build shared packages first
pnpm build:apps       # Build applications
pnpm build:frontend   # Build frontend app
pnpm build:admin      # Build admin app

# Quality Assurance
pnpm lint             # Lint all projects
pnpm lint:fix         # Fix auto-fixable issues
pnpm type-check       # TypeScript checking
pnpm test             # Run all tests
pnpm test:unit        # Unit tests only
pnpm test:e2e         # End-to-end tests

# Testing specific apps
cd apps/frontend && pnpm test:e2e:ui    # Playwright UI mode
cd apps/admin-frontend && pnpm test:e2e:headed  # Headed mode

# Utilities
pnpm clean            # Clean build artifacts
pnpm format           # Format all code
make help             # Show Makefile commands
```

## High-Level Architecture

### Monorepo Structure
- **Turborepo + PNPM workspaces** for build orchestration and dependency management
- **Apps**: Frontend (3000), Admin Frontend (3001), Backend (Rust)
- **Shared Packages**: `@epsx/auth`, `@epsx/types`, `@epsx/ui`, `@epsx/utils`, `@epsx/config`, `@epsx/api-client`, `@epsx/shared`

### Authentication & Authorization (Critical Pattern)
- **Firebase Authentication** for user identity + custom IAM system for permissions
- **Role hierarchy**: `user` � `premium_user` � `moderator` � `admin` � `super_admin`
- **Permission system**: Granular permissions like `read:own_data`, `write:all`, `manage:users`
- **Context providers**: `AuthProvider`, `IAMContext` for global auth state
- **Middleware protection**: Route-level authorization in both apps
- **Admin permissions**: Stored in Firestore with role templates

### State Management Patterns
- **Server state**: SWR for data fetching with automatic revalidation
- **Client state**: Zustand stores (e.g., theme management in `lib/store/theme.ts`)
- **Authentication**: React Context with Firebase integration
- **Forms**: React Hook Form + Zod validation

### Component Architecture
- **Design system**: Custom PancakeSwap-inspired theme with Tailwind
- **UI foundation**: Radix UI primitives in `@epsx/ui` package
- **App-specific components**: Feature-based organization in `components/features/`
- **Shared patterns**: `components/ui/` for base components, `components/common/` for utilities

### API & Data Patterns
- **Frontend**: Next.js API routes + Server Actions for forms
- **Admin**: Enhanced APIs for user management and IAM operations
- **Backend**: Rust server with WebSocket support for real-time stock data
- **Caching**: Multi-level strategy (browser, server, Redis)
- **Error handling**: Error boundaries and consistent error response patterns

### Payment System Architecture
- **Multi-currency support**: USDT across different networks (ERC20, TRC20, BEP20, etc.)
- **Payment tiers**: Bronze/Silver/Gold/Platinum with feature gates
- **Transaction tracking**: Firestore-based with status monitoring
- **QR code payments**: Crypto payment integration

### Build System Dependencies
- **Critical**: Packages must build before apps (`pnpm build:packages` runs first)
- **Development**: Packages run in watch mode, apps consume live changes
- **TypeScript**: Monorepo path mapping with `@epsx/*` aliases
- **Environment variables**: Global env in `turbo.json`, app-specific in `.env.local`

## Important File Patterns

### Configuration Files
- `turbo.json`: Build orchestration and caching configuration
- `tsconfig.base.json`: Shared TypeScript configuration with path mapping
- `pnpm-workspace.yaml`: Workspace definition
- `package.json` (root): Scripts and dev dependencies

### Authentication Flow
- `apps/frontend/context/auth-context.tsx`: Main auth provider
- `apps/frontend/middleware.ts`: Route protection
- `packages/auth/src/`: Shared auth utilities and types
- `apps/admin-frontend/context/shared-admin-auth-provider.tsx`: Admin-specific auth

### Type System
- `packages/types/src/`: Shared TypeScript definitions
- Apps import from `@epsx/types` instead of local type files
- Payment types, auth types, API types centralized

### IAM System (Admin Dashboard)
- `apps/admin-frontend/components/iam/`: User management components
- `apps/admin-frontend/services/iamService.ts`: IAM operations
- `config/iam/default-roles.ts`: Permission templates
- Firestore-based with real-time updates

## Development Workflow

### Working with Packages
1. Start package development: `pnpm dev:packages`
2. Apps automatically consume changes
3. Build packages before production builds

### Authentication Development
- Test with different user roles in Firestore
- Use admin dashboard for user management
- Check middleware protection on new routes

### Adding New Features
1. Define types in `@epsx/types` if shared
2. Add UI components to `@epsx/ui` if reusable
3. Implement in app-specific `components/features/`
4. Add proper permission checks for admin features

### Database Schema (Firestore)
- `users/`: User profiles and permissions
- `payments/`: Transaction history
- `iamRoles/`: Role definitions
- `iamPermissions/`: Permission templates

### Environment Setup
- Copy `.env.example` to `.env.local` in apps
- Firebase config required for auth
- Google OAuth credentials needed
- Payment system env vars for crypto integration

## Common Issues

### Build Problems
- Always run `pnpm build:packages` before building apps
- Clear `.turbo` cache if builds fail: `pnpm clean:cache`
- Check TypeScript path mappings in `tsconfig.base.json`

### Authentication Issues
- Verify Firebase config in environment variables
- Check IAM permissions in Firestore
- Ensure middleware runs on protected routes

### Package Dependencies
- Use `workspace:*` for internal package dependencies
- Avoid circular dependencies between packages
- Install external deps in correct package/app location

### Type Imports
- Import shared types from `@epsx/types`, not local files
- Payment types, auth types are in shared package
- Local types only for app-specific interfaces

## Memory Bank System

### Project Continuity
When working on complex tasks that may span multiple sessions due to token limits:

1. **Create Memory Bank Files**: Store in `.ai_memory/` directory with versioned filenames
2. **Track Progress**: Update todo lists and implementation status in memory files
3. **Document Context**: Include environment context, file locations, and next steps
4. **Version Control**: Use semantic versioning (v1.0, v1.1, etc.) for major updates

### Memory Bank Structure
```
.ai_memory/
├── project_name_v1.0.md     # Initial planning and setup
├── project_name_v1.1.md     # Progress updates
└── feature_name_v1.0.md     # Specific feature implementation
```

### When to Use Memory Bank
- Multi-step implementations requiring 5+ tasks
- Complex features spanning multiple apps/packages
- Tasks that may hit token limits
- Projects requiring context preservation across sessions

### Memory Bank Template
Include in each memory file:
- Project overview and goals
- Current status and completed tasks
- Pending tasks with priorities
- Key architecture decisions
- File locations and modifications
- Next steps when resuming
- Environment context
- Version history

### Resuming from Memory
1. Read the latest version memory file in `.ai_memory/`
2. Review current status and pending tasks
3. Continue from the next priority task
4. Update memory file with progress
5. Create new version if significant changes