/**
 * Server-side JWT Cookie Utilities for Frontend
 * Uses jose library for JWT verification and cookie management
 */
import { cookies } from 'next/headers';
import { verifyJWT, type EPSXJWTPayload } from '@/lib/auth-utils';

/**
 * Get JWT token from httpOnly cookies
 */
export async function getJWTFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('epsx_jwt')?.value || null;
  } catch (error) {
    console.error('❌ Failed to get JWT from cookies:', error);
    return null;
  }
}

/**
 * Verify and decode JWT token from cookies
 */
export async function verifyJWTFromCookies(): Promise<EPSXJWTPayload | null> {
  try {
    const token = await getJWTFromCookies();
    if (!token) return null;
    
    return await verifyJWT(token);
  } catch (error) {
    console.error('❌ Failed to verify JWT from cookies:', error);
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
  } catch (error) {
    console.error('❌ Failed to get session from JWT:', error);
    return { isAuthenticated: false, user: null };
  }
}