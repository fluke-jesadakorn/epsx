/**
 * Admin Frontend Shared Barrel Export  
 * Eliminates deep import paths by re-exporting shared utilities
 * 
 * BEFORE: import { cn } from '../../../../../shared/utils'
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

export {
  generateCodeVerifier,
  generateCodeChallenge, 
  generateRandomString
} from '../../../shared/auth/pkce';

// URL utilities
export {
  getBackendUrl,
  getFrontendUrl,
  getAdminUrl,
  oidcUrls,
  callbackUrls,
  URL,
  URLContext,
  Service
} from '../../../shared/utils/url-resolver';

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
⚠️  ADMIN FRONTEND PERMISSION MIGRATION WARNING

The admin-frontend is using DEPRECATED local permission validation functions.
These functions are INSECURE and will be removed in future versions.

DEPRECATED (Hackable):
- hasPermissionGranular()
- hasAnyPermissionGranular()
- hasAllPermissionsGranular()

SECURE REPLACEMENT (Unhackable):
- Use backend permission authority system
- All permission checks now go through backend API
- See backend-authority-client.ts for implementation

The admin-frontend has been upgraded to use backend permission authority.
Existing components should automatically use the new system.

For more information, see Phase 2.2 completion notes in the codebase.
`);
}

// Permission types
export type {
  EnhancedUserClaims,
  GranularPermissionClaim,
  PermissionHealth
} from '../../../shared/permissions/types/core';

// Date/time formatting utilities
export function formatDateTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  } catch {
    return dateString;
  }
}

export function formatRelativeTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMinutes / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMinutes < 1) return 'just now';
    if (diffMinutes < 60) return `${diffMinutes}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return formatDateTime(dateString);
  } catch {
    return dateString;
  }
}