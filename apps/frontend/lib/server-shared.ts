/**
 * Frontend Server-Only Shared Barrel Export
 * Server functions that use next/headers - only for API routes and Server Components
 */

// Server-only auth utilities (OAuth callback removed - Web3 migration)

// OAuth initiation removed - migrated to Web3 wallet connection flow

// Session functions removed - migrated to Web3-based session management
// Use /api/v1/auth/session endpoints instead for Web3 authentication

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