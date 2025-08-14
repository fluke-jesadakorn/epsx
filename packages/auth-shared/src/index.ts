/**
 * EPSX Auth Shared Package - JWT-Based Authentication
 * Optimized exports for server-side-first JWT authentication
 */

// Essential types for JWT authentication
export type * from './types';

// Core JWT utilities (server-safe)
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

// JWT Cookie Management (server-safe)
export {
  JWTCookieManager,
  createCookieManager,
  extractJWTClaims,
  type CookieConfig,
  type TokenCookies
} from './cookies/cookie-manager';

// Server-side guards and utilities
export {
  SSRAuthGuard,
  SSRRoleContent,
  SSRUserInfo,
  SSRAdminGuard
} from './server/guards';

// ============================================================================
// IMPORT GUIDANCE FOR JWT-BASED ARCHITECTURE:
// 
// Server Components:   import { verifyJWT, signJWT } from '@epsx/auth-shared';
// JWT Utilities:       import { hasPermissionInJWT } from '@epsx/auth-shared';
// Types:              import type { EPSXJWTPayload } from '@epsx/auth-shared';
// 
// Client components should use local auth providers in each app.
// ============================================================================