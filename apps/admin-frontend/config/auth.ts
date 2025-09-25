/**
 * ADMIN FRONTEND - AUTH CONFIGURATION COMPATIBILITY LAYER
 * Web3 wallet-first authentication system with stub compatibility
 * Provides admin-specific auth configuration and backward compatibility
 */

// Stub types for compatibility
export interface AuthConfig {
  name: string;
  version: string;
  features: AuthFeatures;
  oidc: OIDCConfig;
  web3: Web3Config;
  session: SessionConfig;
  endpoints: AuthEndpoints;
}

export interface OIDCConfig {
  clientId: string;
  clientSecret: string;
  issuer: string;
  redirectUri: string;
  scope: string[];
  pkce: boolean;
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
  oidc: boolean;
  web3: boolean;
  progressive: boolean;
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

// Stub implementations for compatibility
export function getAdminAuthConfig(): AuthConfig {
  return {
    name: 'EPSX Admin',
    version: '1.0.0',
    features: { oidc: true, web3: true, progressive: true },
    oidc: getAdminOIDCConfig(),
    web3: getWeb3Config(),
    session: getSessionConfig('admin'),
    endpoints: {
      authorize: '/api/auth/authorize',
      token: '/api/auth/token', 
      userinfo: '/api/auth/userinfo',
      logout: '/api/auth/logout'
    }
  };
}

export function getAdminOIDCConfig(): OIDCConfig {
  return {
    clientId: process.env.OIDC_ADMIN_CLIENT_ID || 'epsx-admin',
    clientSecret: process.env.OIDC_ADMIN_CLIENT_SECRET || 'dev-admin-secret',
    issuer: process.env.BACKEND_URL || 'http://localhost:8080',
    redirectUri: '/api/auth/callback',
    scope: ['openid', 'profile', 'email', 'admin'],
    pkce: true
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

// URL builders (stub implementations)
export function buildAuthorizationUrl(config: OIDCConfig, options: any = {}): string {
  const params = new URLSearchParams({
    client_id: config.clientId,
    redirect_uri: config.redirectUri,
    scope: config.scope.join(' '),
    response_type: 'code',
    state: options.state || 'default',
    ...options
  });
  return `${config.issuer}/oauth/authorize?${params.toString()}`;
}

export function buildLogoutUrl(config: OIDCConfig, options: any = {}): string {
  const params = new URLSearchParams({
    client_id: config.clientId,
    post_logout_redirect_uri: options.postLogoutRedirectUri || '/',
    ...options
  });
  return `${config.issuer}/oauth/logout?${params.toString()}`;
}

export function buildWeb3ChallengeUrl(address: string): string {
  return `/api/auth/web3/challenge?address=${address}`;
}

// Validation functions (stub implementations)
export function validateOIDCCallback(params: Record<string, string>): boolean {
  return !!(params.code && params.state);
}

export function validateWeb3Signature(signature: string, message: string, address: string): boolean {
  return signature.length > 0 && message.length > 0 && address.length > 0;
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
 * Admin-specific auth configuration
 * Uses shared configuration with admin context
 */
export const ADMIN_AUTH_CONFIG = getAdminAuthConfig();

/**
 * Admin OIDC configuration
 */
export const ADMIN_OIDC_CONFIG = getAdminOIDCConfig();

/**
 * Admin session configuration
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
 * Build admin authorization URL
 */
export function buildAdminAuthorizationUrl(options: {
  state?: string;
  nonce?: string;
  codeChallenge?: string;
  codeChallengeMethod?: string;
} = {}): string {
  return buildAuthorizationUrl(ADMIN_OIDC_CONFIG, options);
}

/**
 * Build admin logout URL
 */
export function buildAdminLogoutUrl(options: {
  postLogoutRedirectUri?: string;
  idTokenHint?: string;
} = {}): string {
  const defaultRedirectUri = `${ADMIN_OIDC_CONFIG.redirectUri.split('/api')[0]}/login`;
  return buildLogoutUrl(ADMIN_OIDC_CONFIG, {
    postLogoutRedirectUri: defaultRedirectUri,
    ...options
  });
}

/**
 * Validate admin auth callback
 */
export function validateAdminCallback(params: Record<string, string>) {
  return validateOIDCCallback(params);
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

// Legacy compatibility exports
export const adminAuthConfig = ADMIN_AUTH_CONFIG;
export const adminOidcConfig = ADMIN_OIDC_CONFIG;
export const adminSessionConfig = ADMIN_SESSION_CONFIG;
export const web3Config = getWeb3Config();

export const authConfig = adminAuthConfig;
export const oidcConfig = adminOidcConfig;
export const sessionConfig = adminSessionConfig;

// Default export for backward compatibility
export default ADMIN_AUTH_CONFIG;