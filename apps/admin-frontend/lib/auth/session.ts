/**
 * Simple Session Management for JWT Authentication
 * Clean utilities for JWT-based session handling
 */

import { cookies } from 'next/headers';

import type { EPSXJWTPayload } from '@/shared/auth/jwt';

import { verifyJWTFromCookies } from '@/lib/server/jwt';

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
    const { getJWTFromCookies } = await import('@/lib/server/jwt');
    const accessToken = await getJWTFromCookies();

    return {
      isLoggedIn: true,
      user: payload,
      accessToken: accessToken ?? undefined,
      expiresAt: payload.exp * 1000, // Convert to milliseconds
    };
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to get session:', _error);
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
    cookieStore.delete(COOKIES.access);
    cookieStore.delete(COOKIES.id);
    cookieStore.delete(COOKIES.refresh);

  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to clear session:', _error);
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
 * Create user session data from userinfo (used in OAuth callback)
 * @param userinfo
 * @param accessToken
 * @param refreshToken
 */
export function createUserSession(
  userinfo: OAuthUserInfo,
  accessToken?: string,
  refreshToken?: string
): SessionData {
  const user: EPSXJWTPayload = {
    sub: userinfo.sub || userinfo.id || 'unknown',
    iss: 'epsx-backend',
    aud: 'epsx-admin',
    exp: Math.floor(Date.now() / 1000) + (2 * 60 * 60), // 2 hours
    iat: Math.floor(Date.now() / 1000),
    email: userinfo.email || '',
    name: userinfo.name || userinfo.display_name || 'Unknown User',
    role: userinfo.role || 'user',
    permissions: userinfo.permissions || ['epsx:user:read'],
    platform_context: userinfo.platform_context || 'epsx',
    primary_platform: userinfo.primary_platform || 'epsx',
    package_tier: userinfo.package_tier || 'FREE',
    wallet_address: userinfo.wallet_address || userinfo.sub || '',
  };

  return {
    isLoggedIn: true,
    user,
    expiresAt: user.exp * 1000,
  };
}