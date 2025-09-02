/**
 * Admin OIDC Client Interface
 * Client-side interface for admin OIDC authentication
 * Separated from server actions to avoid Next.js build constraints
 */

import {
  getOIDCTokensFromCookies,
  validateOIDCTokenWithBackend,
  getAdminSessionFromOIDC,
  isValidAdminSession,
  getAdminUserInfo,
  hasAdminPermission,
  getAdminPermissions,
  hasLegacyAdminSession,
  cleanupLegacyAdminCookies
} from './admin-oidc-auth'

export const adminOIDCAuth = {
  // Main session functions
  getSession: getAdminSessionFromOIDC,
  isValidSession: isValidAdminSession,
  getUserInfo: getAdminUserInfo,
  
  // Permission helpers
  hasPermission: hasAdminPermission,
  getPermissions: getAdminPermissions,
  
  // Token management
  getTokens: getOIDCTokensFromCookies,
  validateToken: validateOIDCTokenWithBackend,
  
  // Migration helpers
  hasLegacySession: hasLegacyAdminSession,
  cleanupLegacy: cleanupLegacyAdminCookies,
}

export default adminOIDCAuth