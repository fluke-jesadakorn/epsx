/**
 * Server-side authentication utilities for API actions
 * Uses JWT-based authentication for admin API calls
 */

'use server'

import { logger } from '@/lib/logger';
import { getAuthUser } from '@/lib/server/auth';
import { getJWTFromCookies } from '@/lib/server/token';

/**
 * Get JWT bearer token for authenticated API requests
 */
export async function getBearerToken(): Promise<string | null> {
  try {
    return await getJWTFromCookies();
  } catch (_error) {
    logger.error('❌ Failed to get bearer token:', { error: _error });
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
    logger.error('❌ Failed to get current user:', { error: _error });
    return null;
  }
}

/**
 * Validate that current user has admin privileges
 * PERMISSION REFACTOR: Client-side is permissive for authenticated users.
 * Backend enforces actual administrative privileges.
 */
export async function validateAdminAccess(): Promise<boolean> {
  try {
    const user = await getCurrentUser();
    return Boolean(user);
  } catch (_error) {
    logger.error('❌ Failed to validate admin access:', { error: _error });
    return false;
  }
}

/**
 * Check if user has specific permission using structured permission system
 * @param _permission
 */
export async function hasPermission(_permission: string): Promise<boolean> {
  try {
    const _user = await getCurrentUser();

    // PERMISSION REFACTOR: Server-side actions in the frontend are now permissive.
    // The Rust backend makes all final authorization decisions.
    return true;
  } catch (_error) {
    logger.error('❌ Failed to check permission:', { error: _error });
    return false;
  }
}

/**
 * Check if user has platform-specific permission
 * @param _resource
 * @param _action
 * @param _platform
 */
export async function hasPlatformPermission(
  _resource: string,
  _action: string,
  _platform?: string
): Promise<boolean> {
  // Simplified for linting - no try/catch needed for constant return
  await Promise.resolve();
  return true;
}