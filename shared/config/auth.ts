/**
 * CONSOLIDATED AUTH CONFIGURATION
 * Unified authentication configuration shared across admin-frontend and frontend
 * Supports OIDC, Web3, and progressive authentication patterns
 */

import { env } from '../env/schema';

// ============================================================================
// AUTH CONFIGURATION TYPES
// ============================================================================

export interface AuthEndpoints {
  authorize: string;
  token: string;
  userinfo: string;
  logout: string;
  challenge: string;
  verify: string;
  refresh: string;
  permissions: string;
}

export interface OIDCConfig {
  clientId: string;
  clientSecret?: string; // Only available server-side
  issuer: string;
  scope: string[];
  responseType: string;
  grantType: string;
  redirectUri: string;
  endpoints: AuthEndpoints;
}

export interface Web3Config {
  networkId: string;
  chainId: number;
  walletConnectProjectId: string;
  supportedWallets: string[];
  siweConfig: {
    domain: string;
    uri: string;
    version: string;
    chainId: number;
    statement?: string;
  };
}

export interface SessionConfig {
  cookieName: string;
  maxAge: number;
  secure: boolean;
  sameSite: 'strict' | 'lax' | 'none';
  httpOnly: boolean;
  domain?: string;
}

export interface AuthConfig {
  oidc?: OIDCConfig | null; // Optional for Web3-first architecture
  web3: Web3Config;
  session: SessionConfig;
  features: AuthFeatures;
}

export interface AuthFeatures {
  enableOIDC: boolean;
  enableWeb3: boolean;
  enableProgressiveAuth: boolean;
  enableMFA: boolean;
  enableDeviceBinding: boolean;
  enableSessionManagement: boolean;
  enableAuditLogging: boolean;
}

// ============================================================================
// BASE AUTH CONFIGURATIONS
// ============================================================================

/**
 * Get OIDC configuration for frontend application
 * NOTE: Disabled in Web3-first architecture - OIDC variables removed from schema
 */
/*
export function getFrontendOIDCConfig(): OIDCConfig {
  return {
    clientId: env.OIDC_CLIENT_ID,
    issuer: env.BACKEND_URL,
    scope: ['openid', 'profile', 'email', 'permissions'],
    responseType: 'code',
    grantType: 'authorization_code',
    redirectUri: `${env.APP_URL}/api/auth/callback/epsx-backend`,
    endpoints: {
      authorize: urls.oauth.authorize,
      token: urls.oauth.token,
      userinfo: urls.oauth.userinfo,
      logout: `${env.BACKEND_URL}/api/auth/logout`,
      challenge: `${env.BACKEND_URL}/api/auth/web3/challenge`,
      verify: `${env.BACKEND_URL}/api/auth/web3/verify`,
      refresh: `${env.BACKEND_URL}/api/auth/refresh`,
      permissions: `${env.BACKEND_URL}/api/auth/permissions`,
    },
  };
}
*/

/**
 * Get OIDC configuration for admin frontend
 * NOTE: Disabled in Web3-first architecture - OIDC variables removed from schema
 */
/*
export function getAdminOIDCConfig(): OIDCConfig {
  return {
    clientId: env.ADMIN_CLIENT_ID,
    issuer: env.BACKEND_URL,
    scope: ['openid', 'profile', 'email', 'admin_permissions'],
    responseType: 'code',
    grantType: 'authorization_code',
    redirectUri: `${env.ADMIN_URL}/api/auth/callback/epsx-backend`,
    endpoints: {
      authorize: urls.oauth.authorize,
      token: urls.oauth.token,
      userinfo: urls.oauth.userinfo,
      logout: `${env.BACKEND_URL}/api/auth/logout`,
      challenge: `${env.BACKEND_URL}/api/auth/web3/challenge`,
      verify: `${env.BACKEND_URL}/api/auth/web3/verify`,
      refresh: `${env.BACKEND_URL}/api/auth/refresh`,
      permissions: `${env.BACKEND_URL}/api/auth/permissions`,
    },
  };
}
*/

/**
 * Get Web3 configuration
 */
export function getWeb3Config(): Web3Config {
  const isMainnet = env.BLOCKCHAIN_NETWORK === 'mainnet';
  const chainId = isMainnet ? 56 : 97; // BSC Mainnet : BSC Testnet

  // Determine the correct domain for SIWE
  const getSIWEDomain = () => {
    // If we're in browser, check the current hostname
    if (typeof window !== 'undefined') {
      const currentDomain = window.location.hostname;
      // If accessing via production domain, use it
      if (currentDomain === 'epsx.io') {
        return 'epsx.io';
      }
      // If accessing via localhost or other, use the configured APP_URL domain
      return getDomainFromUrl(env.APP_URL);
    }

    // Server-side: use production domain if configured URLs suggest production
    const appUrl = env.APP_URL;
    if (appUrl.includes('epsx.io')) {
      return 'epsx.io';
    }

    // Default to extracting domain from APP_URL
    return getDomainFromUrl(appUrl);
  };

  const siweUri = () => {
    // If we're in browser and on epsx.io, use https://epsx.io
    if (typeof window !== 'undefined' && window.location.hostname === 'epsx.io') {
      return 'https://epsx.io';
    }
    // Otherwise use the configured APP_URL
    return env.APP_URL;
  };

  return {
    networkId: env.BLOCKCHAIN_NETWORK,
    chainId,
    walletConnectProjectId: env.WALLETCONNECT_PROJECT_ID,
    supportedWallets: [
      'metamask',
      'walletConnect',
      'coinbaseWallet',
      'trustWallet',
      'binanceWallet'
    ],
    siweConfig: {
      domain: getSIWEDomain(),
      uri: siweUri(),
      version: '1',
      chainId,
      statement: 'Sign in to EPSX with your Ethereum wallet',
    },
  };
}

/**
 * Get session configuration
 */
export function getSessionConfig(context: 'admin' | 'user' = 'user'): SessionConfig {
  const isDev = process.env.NODE_ENV === 'development';

  return {
    cookieName: context === 'admin' ? 'admin_session' : 'user_session',
    maxAge: context === 'admin' ? 60 * 60 * 4 : 60 * 60 * 24 * 7, // 4 hours for admin, 7 days for users
    secure: !isDev,
    sameSite: 'lax',
    httpOnly: true,
    domain: isDev ? undefined : getDomainFromUrl(context === 'admin' ? env.ADMIN_URL : env.APP_URL),
  };
}

// ============================================================================
// COMPLETE AUTH CONFIGURATIONS
// ============================================================================

/**
 * Get complete auth configuration for frontend
 */
export function getFrontendAuthConfig(): AuthConfig {
  return {
    oidc: null, // Disabled in Web3-first architecture
    web3: getWeb3Config(),
    session: getSessionConfig('user'),
    features: {
      enableOIDC: false, // Disabled in Web3-first architecture
      enableWeb3: true,
      enableProgressiveAuth: true,
      enableMFA: false, // Disabled for users by default
      enableDeviceBinding: false,
      enableSessionManagement: true,
      enableAuditLogging: true,
    },
  };
}

/**
 * Get complete auth configuration for admin frontend
 */
export function getAdminAuthConfig(): AuthConfig {
  return {
    oidc: null, // Disabled in Web3-first architecture
    web3: getWeb3Config(),
    session: getSessionConfig('admin'),
    features: {
      enableOIDC: false, // Disabled in Web3-first architecture
      enableWeb3: true,
      enableProgressiveAuth: true,
      enableMFA: true, // Enabled for admins
      enableDeviceBinding: true,
      enableSessionManagement: true,
      enableAuditLogging: true,
    },
  };
}

// ============================================================================
// AUTH URL BUILDERS
// ============================================================================

/**
 * Build authorization URL for OIDC flow
 */
export function buildAuthorizationUrl(
  config: OIDCConfig,
  options: {
    state?: string;
    nonce?: string;
    codeChallenge?: string;
    codeChallengeMethod?: string;
  } = {}
): string {
  const params = new URLSearchParams({
    response_type: config.responseType,
    client_id: config.clientId,
    redirect_uri: config.redirectUri,
    scope: config.scope.join(' '),
    state: options.state ?? generateRandomString(32),
    nonce: options.nonce ?? generateRandomString(32),
  });

  // Add PKCE parameters if provided
  if (options.codeChallenge !== undefined) {
    params.append('code_challenge', options.codeChallenge);
    params.append('code_challenge_method', options.codeChallengeMethod ?? 'S256');
  }

  return `${config.endpoints.authorize}?${params.toString()}`;
}

/**
 * Build logout URL
 */
export function buildLogoutUrl(
  config: OIDCConfig,
  options: {
    postLogoutRedirectUri?: string;
    idTokenHint?: string;
  } = {}
): string {
  const params = new URLSearchParams();

  if (options.postLogoutRedirectUri !== undefined) {
    params.append('post_logout_redirect_uri', options.postLogoutRedirectUri);
  }

  if (options.idTokenHint !== undefined) {
    params.append('id_token_hint', options.idTokenHint);
  }

  const queryString = params.toString();
  return queryString ? `${config.endpoints.logout}?${queryString}` : config.endpoints.logout;
}

/**
 * Build Web3 challenge request URL
 */
export function buildWeb3ChallengeUrl(config: OIDCConfig, walletAddress: string): string {
  const params = new URLSearchParams({
    wallet_address: walletAddress,
    client_id: config.clientId,
  });

  return `${config.endpoints.challenge}?${params.toString()}`;
}

// ============================================================================
// AUTH STATE VALIDATION
// ============================================================================

/**
 * Validate OIDC callback parameters
 */
export function validateOIDCCallback(params: Record<string, string>): {
  valid: boolean;
  error?: string;
  data?: {
    code: string;
    state: string;
  };
} {
  if (params.error) {
    return {
      valid: false,
      error: params.error_description || params.error,
    };
  }

  if (!params.code) {
    return {
      valid: false,
      error: 'Missing authorization code',
    };
  }

  if (!params.state) {
    return {
      valid: false,
      error: 'Missing state parameter',
    };
  }

  return {
    valid: true,
    data: {
      code: params.code,
      state: params.state,
    },
  };
}

/**
 * Validate Web3 signature
 */
export function validateWeb3Signature(
  message: string,
  signature: string,
  walletAddress: string
): {
  valid: boolean;
  error?: string;
} {
  // Basic validation - in real implementation, you'd verify the signature
  if (!message || !signature || !walletAddress) {
    return {
      valid: false,
      error: 'Missing required parameters',
    };
  }

  if (!signature.startsWith('0x')) {
    return {
      valid: false,
      error: 'Invalid signature format',
    };
  }

  if (!walletAddress.match(/^0x[a-fA-F0-9]{40}$/)) {
    return {
      valid: false,
      error: 'Invalid wallet address format',
    };
  }

  return { valid: true };
}

// ============================================================================
// PROGRESSIVE AUTH HELPERS
// ============================================================================

export interface ProgressiveAuthState {
  level: 'public' | 'connected' | 'authenticated';
  walletAddress?: string;
  isAuthenticated: boolean;
  permissions: string[];
  expiresAt?: number;
}

/**
 * Determine required auth level for route
 */
export function getRequiredAuthLevel(
  route: string,
  userContext: 'admin' | 'user' = 'user'
): 'public' | 'connected' | 'authenticated' {
  // Admin routes always require full authentication
  if (userContext === 'admin' || route.startsWith('/admin')) {
    return 'authenticated';
  }

  // Payment and sensitive routes require full authentication
  const sensitiveRoutes = [
    '/payment',
    '/billing',
    '/api/payment',
    '/settings/security',
    '/profile/edit',
  ];

  if (sensitiveRoutes.some(sensitiveRoute => route.startsWith(sensitiveRoute))) {
    return 'authenticated';
  }

  // Personalized routes require wallet connection
  const personalizedRoutes = [
    '/dashboard',
    '/portfolio',
    '/settings',
    '/profile',
    '/analytics/export',
  ];

  if (personalizedRoutes.some(personalizedRoute => route.startsWith(personalizedRoute))) {
    return 'connected';
  }

  // Default to public
  return 'public';
}

/**
 * Check if user meets auth requirements for route
 */
export function meetsAuthRequirements(
  route: string,
  authState: ProgressiveAuthState,
  userContext: 'admin' | 'user' = 'user'
): boolean {
  const requiredLevel = getRequiredAuthLevel(route, userContext);

  switch (requiredLevel) {
    case 'public':
      return true;

    case 'connected':
      return authState.level !== 'public';

    case 'authenticated':
      return authState.isAuthenticated;

    default:
      return false;
  }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Extract domain from URL
 */
function getDomainFromUrl(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.hostname;
  } catch {
    return 'localhost';
  }
}

/**
 * Generate random string for state/nonce
 */
function generateRandomString(length: number): string {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * characters.length));
  }
  return result;
}

/**
 * Get auth config based on context
 */
export function getAuthConfig(context: 'admin' | 'user'): AuthConfig {
  return context === 'admin' ? getAdminAuthConfig() : getFrontendAuthConfig();
}

/**
 * Create PKCE challenge
 */
export function createPKCEChallenge(): {
  codeVerifier: string;
  codeChallenge: string;
  codeChallengeMethod: string;
} {
  const codeVerifier = generateRandomString(128);

  // In a real implementation, you'd use crypto.subtle.digest
  // For now, we'll use a simple base64 encoding
  const codeChallenge = btoa(codeVerifier)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');

  return {
    codeVerifier,
    codeChallenge,
    codeChallengeMethod: 'S256',
  };
}

/**
 * Parse JWT payload (client-side only, for display purposes)
 */
export function parseJWTPayload(token: string): Record<string, unknown> | null {
  try {
    const parts = token.split('.');
    const base64Url = parts[1] || '';
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map((c) => `%${(`00${c.charCodeAt(0).toString(16)}`).slice(-2)}`)
        .join('')
    );
    return JSON.parse(jsonPayload) as Record<string, unknown>;
  } catch {
    return null;
  }
}

/**
 * Check if JWT is expired
 */
export function isJWTExpired(token: string): boolean {
  const payload = parseJWTPayload(token);
  if (!payload || typeof payload.exp !== 'number') { return true; }

  return Date.now() >= payload.exp * 1000;
}

/**
 * Get JWT expiry time
 */
export function getJWTExpiry(token: string): number | null {
  const payload = parseJWTPayload(token);
  return (payload && typeof payload.exp === 'number') ? payload.exp * 1000 : null;
}

// ============================================================================
// LEGACY COMPATIBILITY
// ============================================================================

// Export legacy auth configuration objects for backward compatibility
export const authConfig = getFrontendAuthConfig();
export const adminAuthConfig = getAdminAuthConfig();

// Export specific configurations
// NOTE: OIDC configs disabled in Web3-first architecture
// export const oidcConfig = getFrontendOIDCConfig();
// export const adminOidcConfig = getAdminOIDCConfig();
export const web3Config = getWeb3Config();
export const sessionConfig = getSessionConfig();
export const adminSessionConfig = getSessionConfig('admin');