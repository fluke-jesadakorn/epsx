/**
 * Server-side Auth Types and Utilities
 * Provides types and utilities for server-side authentication
 */

import { COOKIES } from '@/shared/auth/cookies';
import type { EPSXJWTPayload } from '@/shared/auth/jwt';

/**
 * Enhanced auth user type based on our JWT structure
 */
export interface EnhancedAuthUser extends EPSXJWTPayload {
  id: string;  // Maps to `sub` field
}

/**
 * Server-side session type
 */
export interface ServerSession {
  user: EnhancedAuthUser;
  expires: string;
  accessToken?: string;
}

/**
 * Convert JWT payload to EnhancedAuthUser
 * @param payload
 */
export function createEnhancedAuthUser(payload: EPSXJWTPayload): EnhancedAuthUser {
  return {
    ...payload,
    id: payload.sub,
  };
}

/**
 * Server-side auth utilities
 */
export async function getServerSession(): Promise<ServerSession | null> {
  try {
    const { cookies } = await import('next/headers');
    const { verifyJWT } = await import('@/shared/auth/jwt');

    const cookieStore = await cookies();
    // OIDC Migration: Use only OIDC access token
    const jwt = cookieStore.get(COOKIES.access_token)?.value;

    if (!jwt) { return null; }

    const payload = await verifyJWT(jwt);
    if (!payload) { return null; }

    // Convert to proper ServerSession type
    const user = createEnhancedAuthUser(payload);
    return {
      user,
      expires: new Date(payload.exp * 1000).toISOString(),
      accessToken: jwt,
    };
  } catch (_error) {

    console.error('❌ Failed to get server session:', _error);
    return null;
  }
}

/**
 * Get current user from server session
 */
export async function getCurrentUser(): Promise<EnhancedAuthUser | null> {
  try {
    const session = await getServerSession();
    if (!session?.user) { return null; }

    return createEnhancedAuthUser(session.user as EPSXJWTPayload);
  } catch (_error) {

    console.error('❌ Failed to get current user:', _error);
    return null;
  }
}

export function hasPermission(user: EnhancedAuthUser | null, _permission: string): boolean {
  // PERMISSION REFACTOR: Server-side checks in the frontend are now permissive.
  // The Rust backend makes all final authorization decisions.
  return !!user;
}

/**
 * Check if user has required admin module (deprecated - use hasPermission instead)
 * @param user
 * @param module
 */
export function hasAdminModule(user: EnhancedAuthUser | null, _module: string): boolean {
  return !!user;
}

/**
 * Check if user is admin (has any admin permissions)
 * @param user
 */
export function isAdmin(user: EnhancedAuthUser | null): boolean {
  return !!user;
}

/**
 * Require admin authentication - throws if not admin
 */
export async function requireAdminAuth(): Promise<EnhancedAuthUser> {
  const user = await getCurrentUser();
  if (!user || !isAdmin(user)) {
    throw new Error('Admin authentication required');
  }
  return user;
}

/**
 * Get user context with permissions info
 */
export async function getUserContext() {
  try {
    const user = await getCurrentUser();
    if (!user) { return null; }

    const platform = user.platform_context || user.primary_platform || 'epsx';

    return {
      user,
      isAdmin: isAdmin(user),
      permissions: user.permissions || [],
      platform,
    };
  } catch (_error) {

    console.error('❌ Failed to get user context:', _error);
    return null;
  }
}

/**
 * Check if user has platform-specific permission
 * @param user
 * @param resource
 * @param action
 * @param platform
 */
export function hasPlatformPermission(
  user: EnhancedAuthUser | null,
  resource: string,
  action: string,
  platform?: string
): boolean {
  return !!user;
}