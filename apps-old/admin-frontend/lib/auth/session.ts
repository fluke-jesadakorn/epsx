/**
 * Simple Session Management for JWT Authentication
 * Clean utilities for JWT-based session handling
 */

import { cookies } from 'next/headers';

import { logger } from '@/lib/logger';
import { getJWTFromCookies, verifyJWTFromCookies } from '@/lib/server/token';
import { COOKIES } from '@/shared/auth/cookies';
import type { EPSXJWTPayload } from '@/shared/auth/jwt';

export interface SessionData {
  isLoggedIn: boolean;
  user?: EPSXJWTPayload;
  accessToken?: string;
  expiresAt?: number;
}

/**
 * Get current session from JWT cookie
 */
export async function getSession(): Promise<SessionData> {
  try {
    const payload = await verifyJWTFromCookies();

    if (!payload) {
      return { isLoggedIn: false };
    }

    // Get the raw JWT token for API calls
    const accessToken = await getJWTFromCookies();

    return {
      isLoggedIn: true,
      user: payload,
      accessToken: accessToken ?? undefined,
      expiresAt: payload.exp * 1000, // Convert to milliseconds
    };
  } catch (_error) {
    logger.auth.error('Failed to get session', { error: _error });
    return { isLoggedIn: false };
  }
}

/**
 * Clear session by removing OIDC cookies
 */
export async function clearSession(): Promise<void> {
  try {
    const cookieStore = await cookies();

    // OIDC Migration: Clear OIDC tokens instead of legacy JWT
    cookieStore.delete(COOKIES.access_token);
    cookieStore.delete(COOKIES.id_token);
    cookieStore.delete(COOKIES.refresh_token);

  } catch (_error) {
    logger.auth.error('Failed to clear session', { error: _error });
  }
}

// OAuth userinfo response interface
interface OAuthUserInfo {
  sub?: string;
  id?: string;
  email?: string;
  name?: string;
  display_name?: string;
  role?: string;
  permissions?: string[];
  platform_context?: string;
  primary_platform?: string;
  package_tier?: string;
  wallet_address?: string;
}

/**
 * Get display name from userinfo
 */
function getDisplayName(userinfo: OAuthUserInfo): string {
  return userinfo.name ?? userinfo.display_name ?? 'Unknown user';
}

/**
 * Get subject from userinfo
 */
function getSubject(userinfo: OAuthUserInfo): string {
  return userinfo.sub ?? userinfo.id ?? 'unknown';
}

/**
 * Helper to build JWT payload from OAuth userinfo
 */
function buildPayload(userinfo: OAuthUserInfo): EPSXJWTPayload {
  const now = Math.floor(Date.now() / 1000);
  const sub = getSubject(userinfo);

  return {
    sub,
    iss: 'epsx-backend',
    aud: 'epsx-admin',
    exp: now + (30 * 24 * 60 * 60), // 30 days
    iat: now,
    email: userinfo.email ?? '',
    name: getDisplayName(userinfo),
    role: userinfo.role ?? 'user',
    permissions: userinfo.permissions ?? ['epsx:user:read'],
    platform_context: userinfo.platform_context ?? 'epsx',
    primary_platform: userinfo.primary_platform ?? 'epsx',
    package_tier: userinfo.package_tier ?? 'FREE',
    wallet_address: userinfo.wallet_address ?? userinfo.sub ?? '',
  };
}

/**
 * Create user session data from userinfo (used in OAuth callback)
 * @param userinfo
 * @param _accessToken - Reserved for future token persistence
 * @param _refreshToken - Reserved for future token refresh logic
 */
export function createUserSession(
  userinfo: OAuthUserInfo,
  _accessToken?: string,
  _refreshToken?: string
): SessionData {
  const user = buildPayload(userinfo);

  return {
    isLoggedIn: true,
    user,
    expiresAt: user.exp * 1000,
  };
}