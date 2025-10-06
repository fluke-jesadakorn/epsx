/**
 * Server-side authentication utilities for API actions
 * Uses JWT-based authentication for admin API calls
 */

'use server'

import { getAuthUser } from '@/lib/server/auth';
import { getJWTFromCookies } from '@/lib/server/jwt';

/**
 * Get JWT bearer token for authenticated API requests
 */
export async function getBearerToken(): Promise<string | null> {
  try {
    return await getJWTFromCookies();
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to get bearer token:', _error);
    return null;
  }
}

/**
 * Get current authenticated user from JWT
 */
export async function getCurrentUser() {
  try {
    return await getAuthUser();
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to get current user:', _error);
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
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to validate admin access:', _error);
    return false;
  }
}

/**
 * Check if user has specific permission using structured permission system
 * @param permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    
    if (!user?.permissions) {
      return false;
    }

    const permissions = Array.isArray(user.permissions) ? user.permissions : [];
    
    // Check for exact permission match
    if (permissions.includes(permission)) {return true;}
    
    // Check for admin wildcard permission
    if (permissions.includes('admin:*:*')) {return true;}
    
    // Check for legacy wildcard permission
    if (permissions.includes('*')) {return true;}
    
    // Check for broader permissions (e.g., admin:users:* covers admin:users:view)
    if (permission.includes(':')) {
      const [platform, resource] = permission.split(':');
      return permissions.some(p => 
        p === `${platform}:${resource}:*` || 
        p === `${platform}:*:*`
      );
    }

    return false;
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to check permission:', _error);
    return false;
  }
}

/**
 * Check if user has platform-specific permission
 * @param resource
 * @param action
 * @param platform
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

    const targetPlatform = platform || (user as any).platform_context || (user as any).primary_platform || 'epsx';
    const permission = `${targetPlatform}:${resource}:${action}`;

    return hasPermission(permission);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to check platform permission:', _error);
    return false;
  }
}