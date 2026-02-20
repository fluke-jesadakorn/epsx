/**
 * OpenID Connect Compliant Cookie Configuration
 *
 * All cookie read/write/delete goes through Next.js server actions only.
 * Client keeps state in React context (in-memory) only.
 *
 * Cookie Naming Convention:
 * - Production: __Host-epsx.{context}.{type}
 * - Development: epsx.{context}.{type}
 */

const env = typeof process !== 'undefined' ? process.env.NODE_ENV : 'development';
const isProduction = env === 'production';
const prefix = isProduction ? '__Host-' : '';

// Cookie duration constants (in seconds)
const SECONDS_MINUTE = 60;
const SECONDS_HOUR = 60 * SECONDS_MINUTE;
const SECONDS_DAY = 24 * SECONDS_HOUR;
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
 * Server-side HttpOnly cookie names (auth tokens)
 */
export const HTTP_ONLY_COOKIES = [
  'access_token',
  'refresh_token',
  'id_token',
  'return_url',
] as const;

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
 * Cookie configuration options
 */
export const COOKIE_OPTIONS = {
  // Server-side HttpOnly cookies (auth tokens)
  httpOnly: {
    httpOnly: true,
    secure: isProduction,
    sameSite: 'lax' as const,
    path: '/',
    domain: undefined as string | undefined, // Required for __Host- prefix
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
    access_token: SECONDS_HOUR,
    refresh_token: SECONDS_MONTH,
    id_token: SECONDS_HOUR,

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
  options: Partial<typeof COOKIE_OPTIONS.httpOnly & { maxAge: number }> = {}
): string {
  const opts = { ...COOKIE_OPTIONS.httpOnly, ...options };
  const parts = [`${name}=${value}`];

  parts.push(`path=${opts.path}`);
  if (opts.domain !== undefined && opts.domain !== '') {
    parts.push(`domain=${opts.domain}`);
  }
  if (opts.maxAge !== undefined) {
    parts.push(`max-age=${opts.maxAge}`);
  }
  parts.push(`SameSite=${opts.sameSite}`);
  if (opts.secure) {
    parts.push('Secure');
  }
  parts.push('HttpOnly');

  return parts.join('; ');
}

/**
 * Type definitions for cookies
 */
export type CookieType = keyof typeof COOKIES;

/**
 * Server-side utility to get auth token from cookies with fallback support.
 * This consolidates the token fetching logic used across server actions.
 * 
 * @param cookieStore - The Next.js cookies() store
 * @param cookieStore.get - The get method of the cookie store
 * @returns The auth token if found, null otherwise
 */
export function getServerAuthToken(
  cookieStore: { get: (name: string) => { value: string } | undefined }
): string | null {
  // Primary: Check session ID cookie
  let token = cookieStore.get(COOKIES.sid)?.value;
  if (token !== undefined && token !== '') { return token; }

  // Secondary: Check access token (HttpOnly)
  token = cookieStore.get(COOKIES.access_token)?.value;
  if (token !== undefined && token !== '') { return token; }

  // Tertiary: Extract from user cookie (client-set JSON with access field)
  try {
    const userCookie = cookieStore.get(COOKIES.user)?.value;
    if (userCookie !== undefined && userCookie !== '') {
      const user = JSON.parse(decodeURIComponent(userCookie)) as { access?: string };
      if (user.access !== undefined && user.access !== '') { return user.access; }
    }
  } catch {
    // Invalid JSON in user cookie
  }

  return null;
}

/**
 * Check if a cookie key corresponds to an HttpOnly cookie
 */
export function isHttpOnlyCookie(cookieKey: keyof typeof COOKIES): boolean {
  return HTTP_ONLY_COOKIES.includes(cookieKey as typeof HTTP_ONLY_COOKIES[number]);
}

/**
 * Check if a cookie key corresponds to a client-side cookie
 */
export function isClientSideCookie(cookieKey: keyof typeof COOKIES): boolean {
  return CLIENT_SIDE_COOKIES.includes(cookieKey as typeof CLIENT_SIDE_COOKIES[number]);
}
