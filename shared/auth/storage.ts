// ============================================================================
// UNIFIED OPENID STANDARD LOCALSTORAGE KEYS
// Same keys for both frontend and admin-frontend applications
// ============================================================================

/**
 * Unified OpenID-compliant localStorage keys
 *
 * Format: oidc.{key} (no clientId suffix)
 *
 * Rationale:
 * - Only ONE user logged in per browser session
 * - True OpenID standard doesn't use clientId in localStorage keys
 * - Frontend and admin share same authentication state
 * - Simpler architecture, less complexity
 */
export const OIDC_KEYS = {
  /** User information object (replaces epsx_web3_user) */
  USER: 'oidc.user',

  /** Access token for API authentication (replaces *_access_token variants) */
  ACCESS_TOKEN: 'oidc.access_token',

  /** Refresh token for token renewal */
  REFRESH_TOKEN: 'oidc.refresh_token',

  /** Token expiry timestamp in milliseconds (replaces *_token_expiry) */
  EXPIRES_AT: 'oidc.expires_at',

  /** Authentication timestamp in milliseconds (replaces epsx_web3_auth_timestamp) */
  AUTH_TIME: 'oidc.auth_time',
} as const;

/**
 * Type-safe localStorage helper for OIDC keys
 */
export type OidcKeyType = typeof OIDC_KEYS[keyof typeof OIDC_KEYS];
