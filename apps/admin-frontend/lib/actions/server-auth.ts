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
    logger.auth.error('Failed to get bearer token', { error: String(_error) });
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
    logger.auth.error('Failed to get current user', { error: String(_error) });
    return null;
  }
}

