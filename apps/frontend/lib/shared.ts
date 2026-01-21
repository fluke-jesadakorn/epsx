/**
 * Frontend Shared Barrel Export
 * Eliminates deep import paths by re-exporting shared utilities
 * 
 * BEFORE: import { cn } from '@/lib/utils'
 * AFTER:  import { cn } from '@/lib/shared'
 */

// Utilities
export { cn, logger, safeError } from '@/shared/utils';

// Auth utilities
export {
  getJWTTimeToExpiry, isJWTExpired, verifyJWT, type EPSXJWTPayload, type JWTUser
} from '@/shared/auth/jwt';

// PKCE functions removed - migrated to Web3 authentication (no OIDC PKCE needed)

// Note: Server-only utilities moved to @/lib/server-shared

// Client-safe URL utilities
export {
  Service, URLContext, callbackUrls, getAdminUrl, getBackendUrl,
  getFrontendUrl
} from '@/shared/utils/url-resolver';

