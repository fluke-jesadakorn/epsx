// ============================================================================
// MODERN AUTH.JS - Single Entry Point for Auth.js v5 Features
// ============================================================================

// Types (always safe to import)
export * from './types';

// Modern Auth.js v5 configuration
export { default as authConfig } from './config/auth.config';

// Modern JWT utilities with jose library
export {
  signJWT,
  verifyJWT,
  decodeJWT,
  getPermissionsFromJWT,
  hasPermissionInJWT,
  getAdminModulesFromJWT,
  hasAdminModuleInJWT,
  getPackageTierFromJWT,
  hasPackageTierInJWT,
  isJWTExpired,
  getJWTTimeToExpiry,
  createJWTClaims,
  type EPSXJWTPayload
} from './jwt/jwt-utils';

// Modern auth hooks (client-side only)
export {
  useModernAuth,
  useAuth,
  useAdminAuth,
  useFeatureAccess,
  type EPSXUser
} from './hooks/useModernAuth';

// Modern permission gate components (client-side only)
export {
  PermissionGate,
  AdminGate,
  SystemAdminGate,
  PackageTierGate,
  PremiumGate
} from './components/PermissionGate';

// Legacy middleware (will be removed in Phase 6)
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

// Shared cookie and cross-app sync utilities
export {
  getSharedCookieConfig,
  getCrossAppAuthConfig,
  SharedCookieManager,
  sharedCookieManager
} from './config/shared-cookie';

export {
  CrossAppAuthSync,
  createCrossAppSync,
  getCrossAppSync
} from './sync/cross-app-sync';

export {
  useSharedAuth,
  type SharedAuthUser,
  type SharedAuthState,
  type SharedAuthActions,
  type UseSharedAuthReturn
} from './hooks/useSharedAuth';

// CSRF protection utilities
export {
  CSRFProtection,
  CSRFProtectedRequest,
  getCSRFProtection,
  getCSRFProtectedRequest,
  type CSRFToken,
  type CSRFConfig
} from './security/csrf-protection';

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