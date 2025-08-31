/**
 * Simple Session Management for JWT Authentication
 * Clean utilities for JWT-based session handling
 */

import { cookies } from 'next/headers';
import { verifyJWTFromCookies } from '@/lib/server/jwt';
import type { EPSXJWTPayload } from '@/lib/auth-utils';

export interface SessionData {
  isLoggedIn: boolean;
  user?: EPSXJWTPayload;
  expiresAt?: number;
}

/**
 * Get current session from JWT cookie
 */
export async function getSession(): Promise<SessionData | null> {
  try {
    const payload = await verifyJWTFromCookies();
    
    if (!payload) {
      return { isLoggedIn: false };
    }

    return {
      isLoggedIn: true,
      user: payload,
      expiresAt: payload.exp * 1000, // Convert to milliseconds
    };
  } catch (error) {
    console.error('❌ Failed to get session:', error);
    return { isLoggedIn: false };
  }
}

/**
 * Clear session by removing JWT cookie
 */
export async function clearSession(): Promise<void> {
  try {
    const cookieStore = await cookies();
    
    // Remove the JWT cookie
    cookieStore.delete('epsx_jwt');
    
    console.log('✅ Session cleared successfully');
  } catch (error) {
    console.error('❌ Failed to clear session:', error);
  }
}

/**
 * Create user session data from userinfo (used in OAuth callback)
 */
export function createUserSession(
  userinfo: any, 
  accessToken?: string, 
  refreshToken?: string
): SessionData {
  const user: EPSXJWTPayload = {
    sub: userinfo.sub || userinfo.id,
    iss: 'epsx-backend',
    aud: 'epsx-admin',
    exp: Math.floor(Date.now() / 1000) + (2 * 60 * 60), // 2 hours
    iat: Math.floor(Date.now() / 1000),
    email: userinfo.email,
    name: userinfo.name || userinfo.display_name,
    role: userinfo.role || 'user',
    permissions: userinfo.permissions || ['epsx:user:read'],
    platform_context: userinfo.platform_context || 'epsx',
    primary_platform: userinfo.primary_platform || 'epsx',
    package_tier: userinfo.package_tier || 'FREE',
    firebase_uid: userinfo.firebase_uid || userinfo.sub,
  };

  return {
    isLoggedIn: true,
    user,
    expiresAt: user.exp * 1000,
  };
}