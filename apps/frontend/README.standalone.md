# EPSX Frontend - Standalone Deployment

This directory contains a standalone version of the EPSX Frontend that has been extracted from the monorepo structure.

## Files

- `package.standalone.json` - A self-contained package.json with all dependencies resolved
- `README.standalone.md` - This file explaining the standalone setup

## Usage

To use the standalone version:

1. Copy `package.standalone.json` to `package.json`:
   ```bash
   cp package.standalone.json package.json
   ```

2. Install dependencies:
   ```bash
   npm install
   # or
   yarn install
   # or
   pnpm install
   ```

3. Run the development server:
   ```bash
   npm run dev
   ```

4. Build for production:
   ```bash
   npm run build
   npm start
   ```

## Workspace Dependencies Resolved

The following workspace dependencies have been flattened and their dependencies included directly:

- `@epsx/api-client` (v1.0.0) - API client functionality
- `@epsx/auth-shared` (v0.1.0) - Authentication utilities  
- `@epsx/config` (v0.1.0) - Configuration utilities
- `@epsx/server-actions` (v0.1.0) - Server action utilities
- `@epsx/server-providers` (v0.1.0) - Server provider components
- `@epsx/shared-core` (v0.0.1) - Core shared utilities
- `@epsx/shared-utils` (v0.1.0) - Shared utility functions
- `@epsx/theme` (v0.1.0) - Theme provider
- `@epsx/types` (v0.1.0) - TypeScript types
- `@epsx/ui` (v0.1.0) - UI component library

## Important Notes

⚠️ **Missing Source Code**: This package.json only resolves the dependencies but does not include the actual source code from the workspace packages. You will need to either:

1. **Copy the source code** from the `packages/` directory into your standalone project
2. **Rebuild the functionality** using the included dependencies
3. **Publish the workspace packages** to npm and use them as regular dependencies

## Missing Workspace Package Source Files

To make this truly standalone, you would need to copy these directories:

```
packages/api-client/src/ → src/lib/api-client/
packages/auth-shared/src/ → src/lib/auth-shared/  
packages/config/src/ → src/lib/config/
packages/server-actions/src/ → src/lib/server-actions/
packages/server-providers/src/ → src/lib/server-providers/
packages/shared-core/src/ → src/lib/shared-core/
packages/shared-utils/src/ → src/lib/shared-utils/
packages/theme/ → src/lib/theme/
packages/types/src/ → src/lib/types/
packages/ui/src/ → src/lib/ui/
```

Then update import statements throughout the codebase from:
```typescript
import { something } from '@epsx/package-name'
```

To:
```typescript
import { something } from './lib/package-name'
```

## Dependencies Added

The following dependencies were added from the workspace packages:

- `@casl/ability`: ^6.7.0 (from auth-shared)
- Additional TypeScript types for workspace functionality

## Build Configuration

You may need to update your build configuration files (tsconfig.json, next.config.ts, etc.) to account for the local source files instead of workspace references.