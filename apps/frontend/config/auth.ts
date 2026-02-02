/**
 * FRONTEND - AUTH CONFIGURATION COMPATIBILITY LAYER
 * Migrated to use consolidated shared/config/auth.ts
 * Provides user-specific auth configuration and backward compatibility
 */

import {
  // Web3 Auth configuration types (OIDC removed)
  AuthConfig,
  AuthFeatures,
  ProgressiveAuthState,
  SessionConfig,
  Web3Config,
  // Legacy compatibility
  authConfig,
  // Web3 URL builders
  buildWeb3ChallengeUrl,
  // Configuration getters (Web3 only)
  getFrontendAuthConfig,
  getJWTExpiry,
  // Progressive auth helpers
  getRequiredAuthLevel,
  getSessionConfig,
  getWeb3Config,
  isJWTExpired,
  meetsAuthRequirements,

  // Utility functions (JWT for Web3 sessions)
  parseJWTPayload,
  sessionConfig,
  // Web3 Validation functions
  validateWeb3Signature,
  web3Config
} from '@/shared/config/auth';
import { env } from './env';

// Re-export types (Web3-focused)
export type {
  AuthConfig, AuthFeatures,
  ProgressiveAuthState, SessionConfig, Web3Config
};

// Re-export Web3 auth utilities for frontend use
export {

  // Web3 URL builders
  buildWeb3ChallengeUrl,
  // Configuration getters (Web3 only)
  getFrontendAuthConfig, getJWTExpiry,
  // Progressive auth helpers
  getRequiredAuthLevel, getSessionConfig,
  getWeb3Config, isJWTExpired, meetsAuthRequirements,

  // Utility functions (JWT for Web3 sessions)
  parseJWTPayload,
  // Web3 Validation functions
  validateWeb3Signature
};

/**
 * Frontend Web3 auth configuration
 * Uses shared configuration with user context
 */
export const FRONTEND_AUTH_CONFIG = getFrontendAuthConfig();

/**
 * Frontend Web3 configuration
 */
export const FRONTEND_WEB3_CONFIG = getWeb3Config();

/**
 * Frontend session configuration (Web3)
 */
export const FRONTEND_SESSION_CONFIG = getSessionConfig('user');

/**
 * Frontend-specific auth utilities
 */

/**
 * Check if current auth level meets user requirements
 */
export function meetsUserAuthRequirements(
  route: string,
  authState: ProgressiveAuthState
): boolean {
  return meetsAuthRequirements(route, authState, 'user');
}

/**
 * Get required auth level for user route
 */
export function getUserRequiredAuthLevel(route: string) {
  return getRequiredAuthLevel(route, 'user');
}

/**
 * Build Web3 challenge URL for frontend
 */
export function buildFrontendWeb3ChallengeUrl(walletAddress: string): string {
  // Create an OIDC-compatible config for Web3 challenge URL
  const BACKEND_URL = process.env.BACKEND_URL || 'http://127.0.0.1:8080';
  const oidcLikeConfig = {
    clientId: 'epsx-frontend',
    issuer: BACKEND_URL,
    scope: ['web3'],
    responseType: 'code',
    grantType: 'authorization_code',
    redirectUri: `${env.APP_URL}/api/auth/callback`,
    endpoints: {
      authorize: `${BACKEND_URL}/oauth/authorize`,
      token: `${BACKEND_URL}/oauth/token`,
      userinfo: `${BACKEND_URL}/oauth/userinfo`,
      logout: `${BACKEND_URL}/oauth/logout`,
      challenge: `${BACKEND_URL}/api/auth/web3/challenge`,
      verify: `${BACKEND_URL}/api/auth/web3/verify`,
      refresh: `${BACKEND_URL}/oauth/token`,
      permissions: `${BACKEND_URL}/api/auth/permissions`
    }
  };
  return buildWeb3ChallengeUrl(oidcLikeConfig, walletAddress);
}

/**
 * Validate Web3 signature for frontend
 */
export function validateFrontendWeb3Signature(signature: string, message: string, address: string) {
  return validateWeb3Signature(signature, message, address);
}

/**
 * Frontend auth error handling
 */
export const FRONTEND_AUTH_ERRORS = {
  INVALID_USER_CREDENTIALS: 'Invalid user credentials',
  USER_SESSION_EXPIRED: 'User session has expired',
  WEB3_WALLET_NOT_CONNECTED: 'Web3 wallet is not connected',
  WEB3_SIGNATURE_INVALID: 'Web3 signature is invalid or expired',
  UNSUPPORTED_WALLET: 'Wallet type is not supported',
  NETWORK_MISMATCH: 'Wallet connected to wrong network',
  INSUFFICIENT_PERMISSIONS: 'Insufficient permissions for this action',
  SUBSCRIPTION_REQUIRED: 'Active subscription required',
  RATE_LIMIT_EXCEEDED: 'Too many authentication attempts',
  DEVICE_NOT_TRUSTED: 'Device not recognized or trusted',
} as const;

/**
 * Progressive auth state helpers for frontend
 */
export function createUserAuthState(
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
 * Check if user has basic authentication
 */
export function hasUserAuthentication(authState: ProgressiveAuthState): boolean {
  return authState.isAuthenticated;
}

/**
 * Check if user has Web3 wallet connected
 */
export function hasWeb3Connection(authState: ProgressiveAuthState): boolean {
  return authState.level !== 'public' && !!authState.walletAddress;
}

/**
 * Check if user can access premium features
 * PERMISSION REFACTOR: Client-side is permissive for authenticated users.
 * Backend (Rust) enforces actual plan/premium access control.
 */
export function canAccessPremiumFeatures(authState: ProgressiveAuthState): boolean {
  return authState.isAuthenticated;
}


/**
 * Get user auth context for progressive auth
 */
export function getUserAuthContext(): 'user' {
  return 'user';
}

/**
 * Check auth state expiry
 */
export function isAuthStateExpired(authState: ProgressiveAuthState): boolean {
  if (!authState.expiresAt) return false;
  return Date.now() >= authState.expiresAt;
}

/**
 * Get time until auth state expires (in minutes)
 */
export function getTimeUntilAuthExpiry(authState: ProgressiveAuthState): number | null {
  if (!authState.expiresAt) return null;
  const timeLeft = authState.expiresAt - Date.now();
  return Math.max(0, Math.floor(timeLeft / (1000 * 60))); // Convert to minutes
}

/**
 * Frontend Web3 auth helpers
 */

/**
 * Get supported wallet types
 */
export function getSupportedWallets(): string[] {
  const config = getWeb3Config();
  return config.supportedWallets;
}

/**
 * Get current network configuration
 */
export function getCurrentNetworkConfig(): any {
  const config = getWeb3Config();
  return {
    chainId: config.chainId,
    networkId: config.networkId,
    siweConfig: config.siweConfig,
  };
}

/**
 * Create SIWE message for Web3 authentication
 */
export function createSIWEMessage(walletAddress: string, nonce: string): string {
  const config = getWeb3Config();
  const { domain, uri, version, statement } = config.siweConfig;

  const message = [
    `${domain} wants you to sign in with your Ethereum account:`,
    walletAddress,
    '',
    statement || 'Sign in to EPSX with your Ethereum wallet',
    '',
    `URI: ${uri}`,
    `Version: ${version}`,
    `Chain ID: ${config.chainId}`,
    `Nonce: ${nonce}`,
    `Issued At: ${new Date().toISOString()}`,
  ].join('\\n');

  return message;
}

// Legacy compatibility exports (Web3-focused)
export { authConfig, sessionConfig, web3Config };

// Default export for backward compatibility
export default FRONTEND_AUTH_CONFIG;