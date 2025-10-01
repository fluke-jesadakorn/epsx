/**
 * Frontend Shared Barrel Export
 * Eliminates deep import paths by re-exporting shared utilities
 * 
 * BEFORE: import { cn } from '@/lib/utils'
 * AFTER:  import { cn } from '@/lib/shared'
 */

// Utilities
export { cn, logger, safeError } from '../../../shared/utils';

// Auth utilities
export { 
  verifyJWT, 
  isJWTExpired, 
  getJWTTimeToExpiry,
  type JWTUser,
  type EPSXJWTPayload 
} from '../../../shared/auth/jwt';

// PKCE functions removed - migrated to Web3 authentication (no OIDC PKCE needed)

// Note: Server-only utilities moved to @/lib/server-shared

// ============================================================================
// DEPRECATED PERMISSION UTILITIES (Phase 2.3 Migration Warning)
// ============================================================================
// ⚠️  WARNING: Local permission validation functions are DEPRECATED
// 🔒 Use backend permission authority system instead for security
// 📊 These functions will be removed in future versions

export {
  hasPermissionGranular,
  hasAnyPermissionGranular,
  hasAllPermissionsGranular,
  canViewAnalytics,
  canExportData,
  canAccessRealtime,
  canUseAdvancedFilters,
  isAdmin
} from '../../../shared/permissions/utils/checking';

export {
  derivePackageTierFromPermissions,
  deriveAccessiblePlatformsFromPermissions
} from '../../../shared/permissions/utils/platform';

// ============================================================================
// MIGRATION WARNING (Runtime Alert)
// ============================================================================

if (typeof console !== 'undefined' && process.env.NODE_ENV === 'development') {
  console.warn(`
⚠️  FRONTEND PERMISSION MIGRATION WARNING

The frontend is using DEPRECATED local permission validation functions.
These functions are INSECURE and will be removed in future versions.

DEPRECATED (Hackable):
- hasPermissionGranular()
- hasAnyPermissionGranular()
- hasAllPermissionsGranular()

SECURE REPLACEMENT (Unhackable):
- Use backend permission authority system
- All permission checks should go through backend API
- Frontend should only handle permission error responses

MIGRATION STATUS:
Phase 2.1 (Frontend Permission Logic Removal) has been completed.
Some components may still use local validation temporarily.

For more information, see Phase 2.4 for remaining frontend cleanup tasks.
`);
}

// Permission types
export type {
  EnhancedUserClaims,
  GranularPermissionClaim,
  PermissionHealthInfo
} from '../../../shared/permissions/types/core';

// Client-safe URL utilities
export {
  getBackendUrl,
  getFrontendUrl,
  getAdminUrl,
  callbackUrls,
  URLContext,
  Service
} from '../../../shared/utils/url-resolver';