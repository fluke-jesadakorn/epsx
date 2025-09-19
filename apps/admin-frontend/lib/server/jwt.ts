/**
 * Server-side JWT Cookie Utilities for Admin Frontend
 * Uses jose library for JWT verification and cookie management
 */
// Legacy JWT support - consider migrating to OIDC
import { jwtVerify } from 'jose';
import { env } from '@/config/env';

// Minimal JWT payload type for legacy support
export interface EPSXJWTPayload {
  sub: string;
  email: string;
  name: string;
  permissions: string[];
  iat: number;
  exp: number;
}

/**
 * Minimal JWT verification function for legacy support
 */
async function verifyJWT(token: string): Promise<EPSXJWTPayload | null> {
  try {
    const jwtSecret = env.NEXTAUTH_SECRET;
    const { payload } = await jwtVerify(
      token,
      new TextEncoder().encode(jwtSecret),
      {
        algorithms: ['HS256'],
      }
    );
    return payload as EPSXJWTPayload;
  } catch (error) {
    console.error('JWT verification failed:', error);
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
    
    // Debug: Log all cookies to see what's available
    const allCookies = cookieStore.getAll();
    console.log('🔍 All available cookies:', allCookies.map(c => ({ name: c.name, hasValue: !!c.value })));
    
    // OIDC Migration: Get access token instead of legacy JWT
    const jwtCookie = cookieStore.get('access_token');
    console.log('🔍 OIDC access token lookup result:', { found: !!jwtCookie, hasValue: !!jwtCookie?.value });
    
    return jwtCookie?.value || null;
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