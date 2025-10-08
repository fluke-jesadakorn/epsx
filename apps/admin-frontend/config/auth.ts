/**
 * ADMIN FRONTEND - AUTH CONFIGURATION COMPATIBILITY LAYER
 * Web3 wallet-first authentication system with stub compatibility
 * Provides admin-specific auth configuration and backward compatibility
 */

// Web3-focused types for admin authentication
export interface AuthConfig {
  name: string;
  version: string;
  features: AuthFeatures;
  web3: Web3Config;
  session: SessionConfig;
  endpoints: AuthEndpoints;
}

export interface Web3Config {
  supportedChains: number[];
  defaultChain: number;
  siweEnabled: boolean;
}

export interface SessionConfig {
  maxAge: number;
  secure: boolean;
  httpOnly: boolean;
  sameSite: 'lax' | 'strict' | 'none';
}

export interface AuthFeatures {
  web3: boolean;
  progressive: boolean;
  walletAuth: boolean;
}

export interface AuthEndpoints {
  authorize: string;
  token: string;
  userinfo: string;
  logout: string;
}

export interface ProgressiveAuthState {
  level: 'public' | 'connected' | 'authenticated';
  walletAddress?: string;
  isAuthenticated: boolean;
  permissions: string[];
  expiresAt?: number;
}

// Web3 Admin Authentication Configuration
export function getAdminAuthConfig(): AuthConfig {
  return {
    name: 'EPSX Admin Web3',
    version: '2.0.0',
    features: { web3: true, progressive: true, walletAuth: true },
    web3: getWeb3Config(),
    session: getSessionConfig('admin'),
    endpoints: {
      authorize: '/api/auth/web3/challenge',
      token: '/api/auth/web3/verify', 
      userinfo: '/api/auth/web3/permissions',
      logout: '/api/auth/web3/logout'
    }
  };
}

export function getWeb3Config(): Web3Config {
  return {
    supportedChains: [56, 97], // BSC mainnet and testnet
    defaultChain: 56,
    siweEnabled: true
  };
}

export function getSessionConfig(context?: string): SessionConfig {
  return {
    maxAge: 24 * 60 * 60 * 1000, // 24 hours
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    sameSite: 'lax'
  };
}

// Web3 URL builders
export function buildWeb3ChallengeUrl(address: string): string {
  return `/api/auth/web3/challenge?address=${address}`;
}

export function buildWeb3VerifyUrl(): string {
  return `/api/auth/web3/verify`;
}

export function buildWeb3LogoutUrl(): string {
  return `/api/auth/web3/logout`;
}

// Web3 validation functions
export function validateWeb3Signature(signature: string, message: string, address: string): boolean {
  return signature.length > 0 && message.length > 0 && address.length > 0;
}

export function validateWalletAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address);
}

// Progressive auth helpers (stub implementations)
export function getRequiredAuthLevel(route: string, context?: string): 'public' | 'connected' | 'authenticated' {
  if (route.includes('/admin/')) return 'authenticated';
  if (route.includes('/dashboard')) return 'authenticated';
  return 'public';
}

export function meetsAuthRequirements(route: string, authState: ProgressiveAuthState, context?: string): boolean {
  const required = getRequiredAuthLevel(route, context);
  const levels = { public: 0, connected: 1, authenticated: 2 };
  const userLevel = levels[authState.level] || 0;
  const requiredLevel = levels[required] || 0;
  return userLevel >= requiredLevel;
}

// Utility functions (stub implementations)
export function createPKCEChallenge(): { codeVerifier: string; codeChallenge: string; codeChallengeMethod: string } {
  const codeVerifier = 'stub-code-verifier';
  const codeChallenge = 'stub-code-challenge';
  return { codeVerifier, codeChallenge, codeChallengeMethod: 'S256' };
}

export function parseJWTPayload(token: string): any {
  try {
    const payload = token.split('.')[1];
    return JSON.parse(atob(payload));
  } catch {
    return {};
  }
}

export function isJWTExpired(token: string): boolean {
  try {
    const payload = parseJWTPayload(token);
    return payload.exp ? Date.now() >= payload.exp * 1000 : true;
  } catch {
    return true;
  }
}

export function getJWTExpiry(token: string): number | null {
  try {
    const payload = parseJWTPayload(token);
    return payload.exp ? payload.exp * 1000 : null;
  } catch {
    return null;
  }
}

/**
 * Admin Web3 auth configuration
 * Uses shared configuration with admin context
 */
export const ADMIN_AUTH_CONFIG = getAdminAuthConfig();

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
 */
export function meetsAdminAuthRequirements(
  route: string,
  authState: ProgressiveAuthState
): boolean {
  return meetsAuthRequirements(route, authState, 'admin');
}

/**
 * Get required auth level for admin route
 */
export function getAdminRequiredAuthLevel(route: string) {
  return getRequiredAuthLevel(route, 'admin');
}

/**
 * Build admin Web3 challenge URL
 */
export function buildAdminWeb3ChallengeUrl(walletAddress: string): string {
  return buildWeb3ChallengeUrl(walletAddress);
}

/**
 * Build admin Web3 logout URL
 */
export function buildAdminWeb3LogoutUrl(): string {
  return buildWeb3LogoutUrl();
}

/**
 * Validate admin Web3 signature
 */
export function validateAdminWeb3Signature(signature: string, message: string, address: string) {
  return validateWeb3Signature(signature, message, address);
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
 */
export function hasAdminAuthentication(authState: ProgressiveAuthState): boolean {
  if (!authState.isAuthenticated) return false;
  
  // Check for admin permissions
  const hasAdminPermission = authState.permissions.some(permission =>
    permission.startsWith('admin:') || permission === 'admin:*:*'
  );
  
  return hasAdminPermission;
}

/**
 * Get admin auth context for progressive auth
 */
export function getAdminAuthContext(): 'admin' {
  return 'admin';
}

// Legacy compatibility exports (Web3-focused)
export const adminAuthConfig = ADMIN_AUTH_CONFIG;
export const adminWeb3Config = ADMIN_WEB3_CONFIG;
export const adminSessionConfig = ADMIN_SESSION_CONFIG;
export const web3Config = getWeb3Config();

export const authConfig = adminAuthConfig;
export const sessionConfig = adminSessionConfig;

// Default export for backward compatibility
export default ADMIN_AUTH_CONFIG;