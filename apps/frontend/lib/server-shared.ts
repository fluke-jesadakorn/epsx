/**
 * Frontend Server-Only Shared Barrel Export
 * Server functions that use next/headers - only for API routes and Server Components
 */

// Server-only auth utilities  
export {
  processOAuthCallback as handleOAuthCallback
} from '../../../shared/auth/oauth-callback';

export {
  createOAuthInitiation,
  createFrontendOAuthConfig
} from '../../../shared/auth/oauth-initiate';

export {
  getSession,
  storeSession,
  clearSession,
  refreshSession
} from '../../../shared/auth/session';

// URL utilities (server-safe)
export {
  getBackendUrl,
  getFrontendUrl,
  getAdminUrl,
  oidcUrls,
  callbackUrls,
  URLContext,
  Service
} from '../../../shared/utils/url-resolver';