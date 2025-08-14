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

    // Check if user has any admin modules or is super admin
    const hasAdminAccess = user.admin_modules.length > 0 || 
                          user.role === 'super_admin' ||
                          user.role === 'admin';

    return hasAdminAccess;
  } catch (error) {
    console.error('❌ Failed to validate admin access:', error);
    return false;
  }
}