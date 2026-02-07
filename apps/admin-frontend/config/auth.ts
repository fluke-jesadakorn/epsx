/**
 * ADMIN FRONTEND - AUTH CONFIGURATION COMPATIBILITY LAYER
 * Web3 wallet-first authentication system using consolidated shared configuration
 * Provides admin-specific auth configuration and backward compatibility
 */

import { ROUTES } from '../lib/route-compatibility';

import {
  AuthFeatures,
  ProgressiveAuthState,
  SessionConfig,
  // Web3 Auth configuration types
  AuthConfig as SharedAuthConfig,
  Web3Config,
  buildLogoutUrl,
  // Web3 URL builders
  buildWeb3ChallengeUrl,
  getJWTExpiry,
  // Progressive auth helpers
  getRequiredAuthLevel,
  getSessionConfig,
  // Configuration getters
  getAdminAuthConfig as getSharedAdminAuthConfig,
  getWeb3Config,
  isJWTExpired,
  meetsAuthRequirements,

  // Utility functions
  parseJWTPayload,
  // Legacy compatibility
  web3Config as sharedWeb3Config,
  // Web3 Validation functions
  validateWeb3Signature
} from '@/shared/config/auth';

// Extend Shared AuthConfig for backward compatibility
export interface AuthConfig extends SharedAuthConfig {
  name?: string;
  version?: string;
  endpoints?: {
    authorize: string;
    token: string;
    userinfo: string;
    logout: string;
  };
}

// Re-export types
export type {
  AuthFeatures,
  ProgressiveAuthState, SessionConfig, Web3Config
};

// Re-export Web3 auth utilities
export {
  buildLogoutUrl, buildWeb3ChallengeUrl, getJWTExpiry, getRequiredAuthLevel, getSessionConfig, getWeb3Config, isJWTExpired, meetsAuthRequirements,
  parseJWTPayload, validateWeb3Signature
};

/**
 * Admin Web3 auth configuration
 * Uses shared configuration with admin context and legacy extensions
 */
export const ADMIN_AUTH_CONFIG: AuthConfig = {
  ...getSharedAdminAuthConfig(),
  name: 'EPSX Admin Web3',
  version: '2.0.0',
  endpoints: {
    authorize: ROUTES.AUTH.WEB3_CHALLENGE,
    token: ROUTES.AUTH.WEB3_VERIFY,
    userinfo: ROUTES.AUTH.WEB3_SESSION,
    logout: ROUTES.AUTH.WEB3_LOGOUT
  }
};

/**
 * Admin Web3 configuration
 */
export const ADMIN_WEB3_CONFIG = getWeb3Config();

/**
 * Admin session configuration (Web3)
 */
export const ADMIN_SESSION_CONFIG = getSessionConfig('admin');

/**
 * Admin-specific auth utilities
 */

/**
 * Check if current auth level meets admin requirements
 * @param route
 * @param authState
 */
export function meetsAdminAuthRequirements(
  route: string,
  authState: ProgressiveAuthState
): boolean {
  return meetsAuthRequirements(route, authState, 'admin');
}

/**
 * Get required auth level for admin route
 * @param route
 */
export function getAdminRequiredAuthLevel(route: string) {
  return getRequiredAuthLevel(route, 'admin');
}

/**
 * Build admin Web3 challenge URL
 * @param walletAddress
 */
export function buildAdminWeb3ChallengeUrl(walletAddress: string): string {
  // We use the shared builder but might need to adapt if specific config is needed
  // For now, reusing the one from shared which uses endpoints from config
  // But buildWeb3ChallengeUrl takes OIDCConfig... shared implementation is:
  // buildWeb3ChallengeUrl(config: OIDCConfig, walletAddress: string)
  // But getAdminAuthConfig returns oidc: null.
  // We need to construct a dummy config or update the usage.
  // The shared buildWeb3ChallengeUrl expects an object with clientId and endpoints.

  // Construct a compatible config object for the shared function
  const compatibilityConfig = {
    clientId: 'epsx-admin', // Fixed ID
    endpoints: {
      challenge: ROUTES.AUTH.WEB3_CHALLENGE
    }
  };

  return buildWeb3ChallengeUrl(compatibilityConfig as unknown as Parameters<typeof buildWeb3ChallengeUrl>[0], walletAddress);
}

/**
 * Build admin Web3 logout URL
 */
export function buildAdminWeb3LogoutUrl(): string {
  return ROUTES.AUTH.WEB3_LOGOUT;
}

/**
 * Validate admin Web3 signature
 * @param signature
 * @param message
 * @param address
 */
export function validateAdminWeb3Signature(signature: string, message: string, address: string) {
  return validateWeb3Signature(message, signature, address);
}

/**
 * Admin auth error handling
 */
export const ADMIN_AUTH_ERRORS = {
  INVALID_ADMIN_CREDENTIALS: 'Invalid administrator credentials',
  INSUFFICIENT_ADMIN_PRIVILEGES: 'Insufficient administrative privileges',
  ADMIN_SESSION_EXPIRED: 'Administrator session has expired',
  ADMIN_MFA_REQUIRED: 'Multi-factor authentication required for admin access',
  ADMIN_DEVICE_NOT_REGISTERED: 'Device not registered for administrative access',
  INVALID_ADMIN_TOKEN: 'Invalid or malformed administrator token',
  ADMIN_ACCOUNT_LOCKED: 'Administrator account is locked',
  ADMIN_PERMISSION_DENIED: 'Access denied for administrative function',
} as const;

/**
 * Admin auth state helpers
 * @param isAuthenticated
 * @param permissions
 * @param walletAddress
 * @param expiresAt
 */
export function createAdminAuthState(
  isAuthenticated: boolean,
  permissions: string[],
  walletAddress?: string,
  expiresAt?: number
): ProgressiveAuthState {
  return {
    level: isAuthenticated ? 'authenticated' : (walletAddress ? 'connected' : 'public'),
    walletAddress,
    isAuthenticated,
    permissions,
    expiresAt,
  };
}

/**
 * Check if user has admin authentication
 * @param authState
 */
export function hasAdminAuthentication(authState: ProgressiveAuthState): boolean {
  if (!authState.isAuthenticated) { return false; }

  // Check for admin permissions
  return authState.permissions.some(permission =>
    permission.startsWith('admin:') || permission === 'admin:*:*'
  );
}

/**
 * Get admin auth context for progressive auth
 */
export function getAdminAuthContext(): 'admin' {
  return 'admin';
}

// Legacy compatibility exports
export const adminAuthConfig = ADMIN_AUTH_CONFIG;
export const adminWeb3Config = ADMIN_WEB3_CONFIG;
export const adminSessionConfig = ADMIN_SESSION_CONFIG;
export const web3Config = sharedWeb3Config;

export const authConfig = adminAuthConfig;
export const sessionConfig = adminSessionConfig;

// Default export for backward compatibility
export default ADMIN_AUTH_CONFIG;