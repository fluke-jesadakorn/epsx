# AGENTS.md

## Build/Test Commands
```bash
# Single test: cd apps/frontend && pnpm test:unit -- -t "test name"
# E2E single: cd apps/frontend && pnpm test:e2e -- --grep "test name"
pnpm build:packages && pnpm build:apps
pnpm lint && pnpm type-check
pnpm test:unit
```

## Code Style
- **Imports**: Use `@epsx/*` for packages, relative for app files
- **Types**: Define in `@epsx/types` if shared, local types for app-specific
- **Naming**: camelCase variables, PascalCase components, UPPER_CASE constants
- **Error handling**: Use try/catch with proper error boundaries
- **Formatting**: Prettier with 2 spaces, semicolons, single quotes
- **Firebase**: Always use AuthProvider + IAMContext for permissions
- **Components**: Feature-based in `components/features/`, shared in `components/ui/`
- **State**: SWR for server state, Zustand for client state
- **Forms**: React Hook Form + Zod validation