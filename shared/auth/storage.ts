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
  // Deprecated: Use cookie management instead
} as const;

/**
 * Type-safe localStorage helper for OIDC keys
 */
export type OidcKeyType = typeof OIDC_KEYS[keyof typeof OIDC_KEYS];
