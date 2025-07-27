// Main entry point - export types only to avoid server/client conflicts
export * from './types';

// Re-export from specific environments
// Note: Consumers should import from specific subpaths to avoid bundling issues
// e.g., import { SSRAuthGuard } from '@epsx/auth-shared/server'
//       import { ClientAuthGuard } from '@epsx/auth-shared/client'
//       import { checkPermissionAccess } from '@epsx/auth-shared/middleware'