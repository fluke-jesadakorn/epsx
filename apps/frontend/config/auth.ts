/**
 * FRONTEND - AUTH CONFIGURATION COMPATIBILITY LAYER
 * Migrated to use consolidated shared/config/auth.ts
 * Provides user-specific auth configuration and backward compatibility
 */

import {
  // Auth configuration types
  AuthConfig,
  OIDCConfig,
  Web3Config,
  SessionConfig,
  AuthFeatures,
  AuthEndpoints,
  ProgressiveAuthState,
  
  // Configuration getters
  getFrontendAuthConfig,
  getFrontendOIDCConfig,
  getSessionConfig,
  getWeb3Config,
  
  // URL builders
  buildAuthorizationUrl,
  buildLogoutUrl,
  buildWeb3ChallengeUrl,
  
  // Validation functions
  validateOIDCCallback,
  validateWeb3Signature,
  
  // Progressive auth helpers
  getRequiredAuthLevel,
  meetsAuthRequirements,
  
  // Utility functions
  createPKCEChallenge,
  parseJWTPayload,
  isJWTExpired,
  getJWTExpiry,
  
  // Legacy compatibility
  authConfig,
  oidcConfig,
  web3Config,
  sessionConfig
} from '../../../shared/config/auth';

// Re-export types
export type {
  AuthConfig,
  OIDCConfig,
  Web3Config,
  SessionConfig,
  AuthFeatures,
  AuthEndpoints,
  ProgressiveAuthState,
};

// Re-export all shared auth utilities for frontend use
export {
  
  // Configuration getters
  getFrontendAuthConfig,
  getFrontendOIDCConfig,
  getSessionConfig,
  getWeb3Config,
  
  // URL builders
  buildAuthorizationUrl,
  buildLogoutUrl,
  buildWeb3ChallengeUrl,
  
  // Validation functions
  validateOIDCCallback,
  validateWeb3Signature,
  
  // Progressive auth helpers
  getRequiredAuthLevel,
  meetsAuthRequirements,
  
  // Utility functions
  createPKCEChallenge,
  parseJWTPayload,
  isJWTExpired,
  getJWTExpiry
};

/**
 * Frontend-specific auth configuration
 * Uses shared configuration with user context
 */
export const FRONTEND_AUTH_CONFIG = getFrontendAuthConfig();

/**
 * Frontend OIDC configuration
 */
export const FRONTEND_OIDC_CONFIG = getFrontendOIDCConfig();

/**
 * Frontend session configuration
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
 * Build user authorization URL
 */
export function buildUserAuthorizationUrl(options: {
  state?: string;
  nonce?: string;
  codeChallenge?: string;
  codeChallengeMethod?: string;
} = {}): string {
  return buildAuthorizationUrl(FRONTEND_OIDC_CONFIG, options);
}

/**
 * Build user logout URL
 */
export function buildUserLogoutUrl(options: {
  postLogoutRedirectUri?: string;
  idTokenHint?: string;
} = {}): string {
  const defaultRedirectUri = `${FRONTEND_OIDC_CONFIG.redirectUri.split('/api')[0]}`;
  return buildLogoutUrl(FRONTEND_OIDC_CONFIG, {
    postLogoutRedirectUri: defaultRedirectUri,
    ...options
  });
}

/**
 * Build Web3 challenge URL for frontend
 */
export function buildFrontendWeb3ChallengeUrl(walletAddress: string): string {
  return buildWeb3ChallengeUrl(FRONTEND_OIDC_CONFIG, walletAddress);
}

/**
 * Validate frontend auth callback
 */
export function validateFrontendCallback(params: Record<string, string>) {
  return validateOIDCCallback(params);
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
 */
export function canAccessPremiumFeatures(authState: ProgressiveAuthState): boolean {
  if (!authState.isAuthenticated) return false;
  
  // Check for premium permissions
  const premiumPermissions = [
    'epsx:analytics:advanced',
    'epsx:realtime:access',
    'epsx:analytics:export'
  ];
  
  return premiumPermissions.some(permission =>
    authState.permissions.includes(permission)
  );
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

// Legacy compatibility exports
export { authConfig };
export { oidcConfig };
export { sessionConfig };
export { web3Config };

// Default export for backward compatibility
export default FRONTEND_AUTH_CONFIG;