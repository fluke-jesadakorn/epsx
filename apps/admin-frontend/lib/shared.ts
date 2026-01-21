/**
 * Admin Frontend Shared Barrel Export  
 * Eliminates deep import paths by re-exporting shared utilities
 * 
 * BEFORE: import { cn } from '@/shared/utils'
 * AFTER:  import { cn } from '@/lib/shared'
 */

// Utilities
export { cn, logger, safeError } from '@/shared/utils';

// Auth utilities
export {
  getJWTTimeToExpiry, isJWTExpired, verifyJWT, type EPSXJWTPayload, type JWTUser
} from '@/shared/auth/jwt';

// PKCE functions removed - migrated to Web3 authentication (no OIDC PKCE needed)

// URL utilities
export {
  Service, URL,
  URLContext, callbackUrls, getAdminUrl, getBackendUrl,
  getFrontendUrl, oidcUrls
} from '@/shared/utils/url-resolver';

// ============================================================================
// PERMISSION SYSTEM: PURE BACKEND AUTHORITY
// ============================================================================
// ✅ ALL permission validation handled by backend
// ✅ Admin frontend only displays errors from backend
// ✅ Use fetchWithPermissionHandling() for API calls with automatic error handling

// Date/time formatting utilities - from shared
export { formatDateTime, formatRelativeTime } from '@/shared/utils/formatting/date';
