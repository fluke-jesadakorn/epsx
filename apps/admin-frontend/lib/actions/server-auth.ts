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
 * Validate that current user has admin privileges using structured permissions only
 */
export async function validateAdminAccess(): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    
    if (!user) {
      return false;
    }

    // Check structured permissions only
    if (user.permissions?.length > 0) {
      return user.permissions.some(p => 
        p === 'admin:*:*' ||           // Full admin access
        p.startsWith('admin:')         // Any admin-scoped permission
      );
    }

    // No valid admin permissions found
    return false;
  } catch (error) {
    console.error('❌ Failed to validate admin access:', error);
    return false;
  }
}

/**
 * Check if user has specific permission using structured permission system
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    
    if (!user || !user.permissions) {
      return false;
    }

    // Check for exact permission match
    if (user.permissions.includes(permission)) return true;
    
    // Check for admin wildcard permission
    if (user.permissions.includes('admin:*:*')) return true;
    
    // Check for legacy wildcard permission
    if (user.permissions.includes('*')) return true;
    
    // Check for broader permissions (e.g., admin:users:* covers admin:users:view)
    if (permission.includes(':')) {
      const [platform, resource] = permission.split(':');
      return user.permissions.some(p => 
        p === `${platform}:${resource}:*` || 
        p === `${platform}:*:*`
      );
    }

    return false;
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