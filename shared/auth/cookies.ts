/**
 * OpenID Connect Compliant Cookie Configuration
 *
 * Cookie Naming Convention:
 * - Production: __Host-epsx.{context}.{type}
 * - Development: epsx.{context}.{type}
 *
 * Security Prefixes:
 * - __Host-: Requires Secure flag, no Domain attribute, path=/
 * - Only used in production for maximum security
 *
 * Context:
 * - user: Frontend application tokens
 * - admin: Admin-frontend application tokens
 *
 * Token Types:
 * - access: Short-lived access token (1 hour)
 * - id: OpenID Connect ID token (1 hour)
 * - refresh: Long-lived refresh token (7 days)
 * - state: OAuth state parameter for CSRF protection
 * - nonce: OAuth nonce for replay protection
 *
 * Client-Side Data (JavaScript Accessible):
 * - theme: UI theme preference (light/dark)
 * - browser_notifications: Browser notification settings
 * - affiliate_attribution: Affiliate tracking data
 * - affiliate_code: Affiliate referral code
 * - wallet_state: Wallet connection state
 * - expires_at: Token expiration timestamp
 * - auth_time: Authentication timestamp
 */

const env = typeof process !== 'undefined' ? process.env.NODE_ENV : 'development';
const isProduction = env === 'production';
const prefix = isProduction ? '__Host-' : '';

// Cookie duration constants (in seconds)
const SECONDS_MINUTE = 60;
const SECONDS_HOUR = 60 * SECONDS_MINUTE;
const SECONDS_DAY = 24 * SECONDS_HOUR;
const SECONDS_WEEK = 7 * SECONDS_DAY;
const SECONDS_MONTH = 30 * SECONDS_DAY;
const SECONDS_YEAR = 365 * SECONDS_DAY;

/**
 * Unified cookie names - shared across all EPSX apps
 * No context separation - same cookies for frontend and admin-frontend
 */
export const COOKIES = {
  // Server-side HttpOnly auth cookies
  access_token: `${prefix}epsx.access_token`,
  refresh_token: `${prefix}epsx.refresh_token`,
  id_token: `${prefix}epsx.id_token`,


  // Client-side JavaScript accessible cookies
  user: `${prefix}epsx.user`,
  sid: `${prefix}epsx.sid`, // Replaces client_session
  expires_at: `${prefix}epsx.expires_at`,
  auth_time: `${prefix}epsx.auth_time`,

  // UX/State
  theme: `${prefix}epsx.theme`,
  browser_notifications: `${prefix}epsx.browser_notifications`,
  affiliate_attribution: `${prefix}epsx.affiliate_attribution`,
  affiliate_code: `${prefix}epsx.affiliate_code`,
  wallet_state: `${prefix}epsx.wallet_state`,
  return_url: `${prefix}epsx.return_url`,
} as const;

/**
 * Cookie configuration options
 */
export const COOKIE_OPTIONS = {
  // Server-side HttpOnly cookies (auth tokens)
  httpOnly: {
    httpOnly: true,
    secure: isProduction,
    sameSite: 'lax' as const,
    path: '/',
    domain: undefined, // Required for __Host- prefix
  },

  // Client-side JavaScript accessible cookies
  clientSide: {
    httpOnly: false,
    secure: isProduction,
    sameSite: 'lax' as const,
    path: '/',
    domain: undefined,
  },

  maxAge: {
    // Auth tokens
    access_token: SECONDS_MONTH,
    refresh_token: SECONDS_MONTH,
    id_token: SECONDS_MONTH,


    // Client-side data
    user: SECONDS_MONTH,
    sid: SECONDS_MONTH,
    expires_at: SECONDS_MONTH,
    auth_time: SECONDS_MONTH,
    theme: SECONDS_YEAR,
    browser_notifications: SECONDS_YEAR,
    affiliate_attribution: SECONDS_MONTH,
    affiliate_code: SECONDS_MONTH,
    wallet_state: null, // Session cookie
    return_url: 300, // 5 minutes
  },
} as const;

/**
 * Get cookie name for type
 */
export function getCookieName(
  type: 'access_token' | 'refresh_token' | 'id_token'
): string {
  return COOKIES[type];
}

/**
 * Get cookie options for token type
 */
export function getCookieOptions(type: keyof typeof COOKIE_OPTIONS.maxAge) {
  return {
    ...COOKIE_OPTIONS,
    maxAge: COOKIE_OPTIONS.maxAge[type],
  };
}

/**
 * Build cookie string for setting cookies
 */
export function buildCookieString(
  name: string,
  value: string,
  options: Partial<typeof COOKIE_OPTIONS> = {}
): string {
  const opts = { ...COOKIE_OPTIONS, ...options };
  const parts = [`${name}=${value}`];

  if ((opts as any).path) parts.push(`path=${(opts as any).path}`);
  if ((opts as any).domain) parts.push(`domain=${(opts as any).domain}`);
  if (opts.maxAge) parts.push(`max-age=${opts.maxAge}`);
  if ((opts as any).sameSite) parts.push(`SameSite=${(opts as any).sameSite}`);
  if ((opts as any).secure) parts.push('Secure');
  if ((opts as any).httpOnly) parts.push('HttpOnly');

  return parts.join('; ');
}

/**
 * Parse cookies from cookie header string
 */
export function parseCookies(cookieHeader: string): Record<string, string> {
  return cookieHeader.split(';').reduce((acc, cookie) => {
    const [key, value] = cookie.trim().split('=');
    if (key && value) {
      acc[key] = value;
    }
    return acc;
  }, {} as Record<string, string>);
}

/**
 * Get cookie value from cookie header
 */
export function getCookieValue(
  cookieHeader: string,
  cookieName: string
): string | null {
  const cookies = parseCookies(cookieHeader);
  return cookies[cookieName] || null;
}

/**
 * Check if browser has any cookies set
 */
export function hasCookies(): boolean {
  if (typeof document === 'undefined') return false;
  return document.cookie.length > 0;
}

/**
 * Get all EPSX cookies from browser
 */
export function getEpsxCookies(): Record<string, string> {
  if (typeof document === 'undefined') return {};

  const cookies = parseCookies(document.cookie);
  const epsxCookies: Record<string, string> = {};

  // Filter only EPSX cookies (with or without __Host- prefix)
  for (const [name, value] of Object.entries(cookies)) {
    if (name.includes('epsx.')) {
      epsxCookies[name] = value;
    }
  }

  return epsxCookies;
}

/**
 * Clear all EPSX cookies
 */
export function clearAllCookies(): void {
  if (typeof document === 'undefined') return;

  const cookiesToClear = Object.values(COOKIES);

  cookiesToClear.forEach(cookieName => {
    document.cookie = `${cookieName}=; Max-Age=0; path=${(COOKIE_OPTIONS as any).httpOnly?.path || '/'}`;
    document.cookie = `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=${(COOKIE_OPTIONS as any).httpOnly?.path || '/'}`;
  });
}

/**
 * Client-side cookie manipulation utilities
 */

/**
 * Set a client-side cookie (JavaScript accessible)
 */
export function setClientCookie(
  name: string,
  value: string,
  maxAge?: number | null
): void {
  if (typeof document === 'undefined') return;

  const options = COOKIE_OPTIONS.clientSide;
  const parts = [`${name}=${encodeURIComponent(value)}`];

  parts.push(`path=${options.path}`);
  if (options.secure) parts.push('Secure');
  if (options.sameSite) parts.push(`SameSite=${options.sameSite}`);

  // Handle maxAge (null means session cookie)
  if (maxAge !== null && maxAge !== undefined) {
    parts.push(`max-age=${maxAge}`);
  }

  document.cookie = parts.join('; ');
}

/**
 * Get a client-side cookie value
 */
export function getClientCookie(name: string): string | null {
  if (typeof document === 'undefined') return null;

  const cookies = parseCookies(document.cookie);
  const value = cookies[name];

  // Decode URL-encoded value
  return value ? decodeURIComponent(value) : null;
}

/**
 * Remove a client-side cookie
 */
export function removeClientCookie(name: string): void {
  if (typeof document === 'undefined') return;

  const options = COOKIE_OPTIONS.clientSide;
  document.cookie = `${name}=; max-age=0; path=${options.path}; SameSite=${options.sameSite}${options.secure ? '; Secure' : ''
    }`;
}

/**
 * Set JSON data in a client-side cookie
 */
export function setClientCookieJSON<T extends Record<string, any>>(
  name: string,
  data: T,
  maxAge?: number | null
): void {
  setClientCookie(name, JSON.stringify(data), maxAge);
}

/**
 * Get JSON data from a client-side cookie
 */
export function getClientCookieJSON<T = any>(name: string): T | null {
  const value = getClientCookie(name);
  if (!value) return null;

  try {
    return JSON.parse(value) as T;
  } catch (error) {
    console.warn(`Failed to parse JSON from cookie ${name}:`, error);
    return null;
  }
}



/**
 * Clear all client-side cookies
 */
export function clearClientSideCookies(): void {
  Object.entries(COOKIES).forEach(([key, name]) => {
    if (!HTTP_ONLY_COOKIES.includes(key as any)) {
      removeClientCookie(name);
    }
  });
}

/**
 * Type definitions for cookies
 */
export type CookieType = keyof typeof COOKIES;

/**
 * Client-side cookie names (those that are NOT HttpOnly)
 */
export const CLIENT_SIDE_COOKIES = [
  'user',
  'expires_at',
  'auth_time',
  'theme',
  'browser_notifications',
  'affiliate_attribution',
  'affiliate_code',
  'wallet_state',
] as const;

/**
 * Server-side HttpOnly cookie names (auth tokens)
 */
export const HTTP_ONLY_COOKIES = [
  'access_token',
  'refresh_token',
  'id_token',
  'return_url',
] as const;

/**
 * Server-side utility to get auth token from cookies with fallback support.
 * This consolidates the token fetching logic used across server actions.
 * 
 * @param cookieStore - The Next.js cookies() store
 * @returns The auth token if found, null otherwise
 */
export function getServerAuthToken(
  cookieStore: { get: (name: string) => { value: string } | undefined }
): string | null {
  // Primary: Check session ID cookie
  let token = cookieStore.get(COOKIES.sid)?.value;
  if (token) return token;

  // Secondary: Check access token
  token = cookieStore.get(COOKIES.access_token)?.value;
  if (token) return token;

  return null;
}

/**
 * Check if a cookie key corresponds to an HttpOnly cookie
 */
export function isHttpOnlyCookie(cookieKey: keyof typeof COOKIES): boolean {
  return HTTP_ONLY_COOKIES.includes(cookieKey as any);
}

/**
 * Check if a cookie key corresponds to a client-side cookie
 */
export function isClientSideCookie(cookieKey: keyof typeof COOKIES): boolean {
  return CLIENT_SIDE_COOKIES.includes(cookieKey as any);
}
