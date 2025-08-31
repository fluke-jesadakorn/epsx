/**
 * Server-side authentication utilities for API actions
 * Uses JWT-based authentication for admin API calls
 */

'use server'

import { getJWTFromCookies } from '@/lib/server/jwt';
import { getAuthUser } from '@/lib/server/auth';

/**
 * Get JWT bearer token for authenticated API requests
 */
export async function getBearerToken(): Promise<string | null> {
  try {
    return await getJWTFromCookies();
  } catch (error) {
    console.error('❌ Failed to get bearer token:', error);
    return null;
  }
}

/**
 * Get current authenticated user from JWT
 */
export async function getCurrentUser() {
  try {
    return await getAuthUser();
  } catch (error) {
    console.error('❌ Failed to get current user:', error);
    return null;
  }
}

/**
 * Validate that current user has admin privileges
 */
export async function validateAdminAccess(): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    
    if (!user) {
      return false;
    }

    // Check new permissions system first
    if (user.permissions?.length > 0) {
      const hasAdminPermission = user.permissions.some(p => 
        p.includes(':manage') || 
        p.includes(':admin') || 
        p === '*'
      );
      if (hasAdminPermission) return true;
    }

    // Fallback to role check for admin access
    return user.role === 'admin';
  } catch (error) {
    console.error('❌ Failed to validate admin access:', error);
    return false;
  }
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    
    if (!user) {
      return false;
    }

    return user.permissions?.includes(permission) || 
           user.permissions?.includes('*') || 
           false;
  } catch (error) {
    console.error('❌ Failed to check permission:', error);
    return false;
  }
}

/**
 * Check if user has platform-specific permission
 */
export async function hasPlatformPermission(
  resource: string, 
  action: string, 
  platform?: string
): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    
    if (!user) {
      return false;
    }

    const targetPlatform = platform || user.platform_context || user.primary_platform || 'epsx';
    const permission = `${targetPlatform}:${resource}:${action}`;

    return hasPermission(permission);
  } catch (error) {
    console.error('❌ Failed to check platform permission:', error);
    return false;
  }
}