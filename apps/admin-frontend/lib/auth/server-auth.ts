/**
 * Server-side Auth Types and Utilities
 * Provides types and utilities for server-side authentication
 */

import type { EPSXJWTPayload } from '@/lib/auth/jwt-utils';

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
    const { verifyJWT } = await import('@/lib/auth-utils');
    
    const cookieStore = await cookies();
    const jwt = cookieStore.get('epsx_admin_jwt')?.value || cookieStore.get('epsx_jwt')?.value;
    
    if (!jwt) return null;
    
    const payload = await verifyJWT(jwt);
    if (!payload) return null;
    
    // Convert to proper ServerSession type
    const user = createEnhancedAuthUser(payload);
    return {
      user,
      expires: new Date(payload.exp * 1000).toISOString(),
      accessToken: jwt,
    };
  } catch (error) {
    console.error('❌ Failed to get server session:', error);
    return null;
  }
}

/**
 * Get current user from server session
 */
export async function getCurrentUser(): Promise<EnhancedAuthUser | null> {
  try {
    const session = await getServerSession();
    if (!session?.user) return null;
    
    return createEnhancedAuthUser(session.user as any);
  } catch (error) {
    console.error('❌ Failed to get current user:', error);
    return null;
  }
}

/**
 * Check if user has required permissions
 */
export function hasPermission(user: EnhancedAuthUser | null, permission: string): boolean {
  if (!user?.permissions) return false;
  return user.permissions.includes(permission) || user.permissions.includes('*');
}

/**
 * Check if user has required admin module (deprecated - use hasPermission instead)
 */
export function hasAdminModule(user: EnhancedAuthUser | null, module: string): boolean {
  if (!user) return false;
  
  // Convert legacy module to structured permission
  const modulePermissionMap: Record<string, string> = {
    'user_management': 'epsx:users:manage',
    'analytics': 'epsx:analytics:view',
    'security': 'epsx:security:manage',
    'notifications': 'epsx:notifications:manage',
    'billing': 'epsx:billing:manage',
  };
  
  const permission = modulePermissionMap[module];
  return permission ? hasPermission(user, permission) : false;
}

/**
 * Check if user is admin (has any admin permissions)
 */
export function isAdmin(user: EnhancedAuthUser | null): boolean {
  if (!user) return false;
  
  // Check permissions system
  if (user.permissions?.length > 0) {
    return user.permissions.some(p => 
      p.includes(':manage') || 
      p.includes(':admin') || 
      p === '*'
    );
  }
  
  return false;
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
    if (!user) return null;
    
    const platform = user.platform_context || user.primary_platform || 'epsx';
    
    return {
      user,
      isAdmin: isAdmin(user),
      permissions: user.permissions || [],
      platform,
    };
  } catch (error) {
    console.error('❌ Failed to get user context:', error);
    return null;
  }
}

/**
 * Check if user has platform-specific permission
 */
export function hasPlatformPermission(
  user: EnhancedAuthUser | null, 
  resource: string, 
  action: string,
  platform?: string
): boolean {
  if (!user) return false;
  
  const targetPlatform = platform || user.platform_context || user.primary_platform || 'epsx';
  const permission = `${targetPlatform}:${resource}:${action}`;
  
  return hasPermission(user, permission);
}