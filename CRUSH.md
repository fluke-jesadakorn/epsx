# CRUSH Agent Configuration

## Build Commands
- `pnpm build` - Build all apps and packages
- `pnpm build:apps` - Build all applications
- `pnpm build:packages` - Build all packages
- `pnpm build:frontend` - Build frontend app
- `pnpm build:admin` - Build admin frontend app
- `pnpm build:backend` - Build backend app

## Lint Commands
- `pnpm lint` - Run linting on all projects
- `pnpm lint:fix` - Run linting and fix issues

## Test Commands
- `pnpm test` - Run all tests
- `pnpm test:unit` - Run unit tests
- `pnpm test:e2e` - Run end-to-end tests
- `pnpm test:watch` - Run tests in watch mode

## Code Style Guidelines
- Semi-colons required
- Single quotes preferred
- Trailing commas for ES5 compatibility
- 2-space indentation
- 80 character line width
- Bracket spacing enabled
- Arrow parentheses avoided when possible

## Import/Export Conventions
- Consistent type imports using `import type`
- Import ordering: built-in, external, internal, parent, sibling, index, object, type
- Alphabetical ordering within groups
- No duplicate imports

## Naming Conventions
- CamelCase for variables and functions
- PascalCase for components and types
- UPPER_CASE for constants

## Error Handling Patterns
- TypeScript warnings for unused variables (with ignore pattern for prefixed underscores)
- Explicit any usage discouraged but allowed with warnings
- Non-null assertions discouraged but allowed with warnings
- React-specific error prevention rules enabled