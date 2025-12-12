/**
 * Admin Frontend Shared Barrel Export  
 * Eliminates deep import paths by re-exporting shared utilities
 * 
 * BEFORE: import { cn } from '../../../../../shared/utils'
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

// Date/time formatting utilities
/**
 *
 * @param dateString
 */
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

/**
 *
 * @param dateString
 */
export function formatRelativeTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMinutes / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMinutes < 1) { return 'just now'; }
    if (diffMinutes < 60) { return `${diffMinutes}m ago`; }
    if (diffHours < 24) { return `${diffHours}h ago`; }
    if (diffDays < 7) { return `${diffDays}d ago`; }
    return formatDateTime(dateString);
  } catch {
    return dateString;
  }
}