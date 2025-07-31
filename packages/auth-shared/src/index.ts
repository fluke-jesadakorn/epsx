// ============================================================================
// CONSOLIDATED AUTH - Single Entry Point for All Auth Features
// ============================================================================

// Types (always safe to import)
export * from './types';

// Core auth utilities (server-safe only)
export { 
  getSessionToken,
  checkPermissionAccess,
  checkRoleHierarchy,
  clearPermissionCache,
  addSecurityHeaders,
  addAdminSecurityHeaders,
  createUnifiedMiddleware,
  createFrontendMiddleware,
  createAdminMiddleware
} from './middleware/index';

// NOTE: Client-side components (providers, hooks, guards) are exported 
// from separate entry points to avoid server/client bundling conflicts:
// 
// - Use '@epsx/auth-shared/client' for React providers and hooks
// - Use '@epsx/auth-shared/middleware' for middleware utilities
// - Use '@epsx/auth-shared' for types and server-safe utilities

// ============================================================================
// IMPORT GUIDANCE - Use these imports to minimize dependencies:
// 
// Types only:      import type { AuthConfig, AuthState } from '@epsx/auth-shared';
// Middleware:      import { createFrontendMiddleware } from '@epsx/auth-shared';
// Providers:       import { UnifiedAuthProvider } from '@epsx/auth-shared';
// Hooks:           import { useAuth, usePermissions } from '@epsx/auth-shared';
// Guards:          import { AuthGuard } from '@epsx/auth-shared';
// 
// Avoid importing everything: import * from '@epsx/auth-shared' (discouraged)
// ============================================================================