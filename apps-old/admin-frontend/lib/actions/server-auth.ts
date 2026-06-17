/**
 * Server-side authentication utilities for API actions
 * Uses JWT-based authentication for admin API calls
 */

'use server'

import { logger } from '@/lib/logger';
import { getAuthUser } from '@/lib/server/auth';

/**
 * Get JWT bearer token for authenticated API requests
 */
export async function getBearerToken(): Promise<string | null> {
  await Promise.resolve();
  logger.warn('Refusing to return HttpOnly bearer token to a client-callable server action');
  return null;
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
