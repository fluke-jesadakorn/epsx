/**
 * Server-side JWT Cookie Utilities for Admin Frontend - WEB3-FIRST
 * Uses jose library for JWT verification and cookie management with Web3 secrets
 * Phase 4.2: Updated to use Web3 app secrets, legacy JWT marked for Web3 migration
 */
import { jwtVerify } from 'jose';

import { env } from '@/config/env';
import { COOKIES } from '@/shared/auth/cookies';
import type { EPSXJWTPayload } from '@/shared/auth/jwt';

export type { EPSXJWTPayload };

/**
 * JWT verification function with Web3 app secret
 * Phase 4.2: Updated to use WEB3_APP_SECRET instead of NEXTAUTH_SECRET
 * @param token
 */
async function verifyJWT(token: string): Promise<EPSXJWTPayload | null> {
  try {
    // Use Web3 app secret with legacy fallback
    const jwtSecret = env.WEB3_APP_SECRET || env.WEB3_APP_SECRET;

    if (!jwtSecret) {

      console.error('No WEB3_APP_SECRET or JWT_SECRET configured for JWT verification');
      return null;
    }

    const { payload } = await jwtVerify(
      token,
      new TextEncoder().encode(jwtSecret),
      {
        algorithms: ['HS256'],
      }
    );
    return payload as EPSXJWTPayload;
  } catch (_error) {

    console.error('JWT verification failed:', _error);
    return null;
  }
}

/**
 * Get JWT token from httpOnly cookies
 */
export async function getJWTFromCookies(): Promise<string | null> {
  try {
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();

    // Get access token from unified cookies (no context separation)
    const jwtCookie = cookieStore.get(COOKIES.access_token);

    return jwtCookie?.value || null;
  } catch (_error) {

    console.error('❌ Failed to get JWT from cookies:', _error);
    return null;
  }
}

/**
 * Verify and decode JWT token from cookies
 */
export async function verifyJWTFromCookies(): Promise<EPSXJWTPayload | null> {
  try {
    const token = await getJWTFromCookies();
    if (!token) { return null; }

    return await verifyJWT(token);
  } catch (_error) {

    console.error('❌ Failed to verify JWT from cookies:', _error);
    return null;
  }
}

/**
 * Get user session data from JWT cookies
 */
export async function getSessionFromJWT(): Promise<{
  isAuthenticated: boolean;
  user: EPSXJWTPayload | null;
}> {
  try {
    const payload = await verifyJWTFromCookies();

    if (!payload) {
      return { isAuthenticated: false, user: null };
    }

    return { isAuthenticated: true, user: payload };
  } catch (_error) {

    console.error('❌ Failed to get session from JWT:', _error);
    return { isAuthenticated: false, user: null };
  }
}