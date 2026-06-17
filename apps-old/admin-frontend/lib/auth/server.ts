/**
 * Server-side Auth Types and Utilities
 * Provides types and utilities for server-side authentication
 */

import { logger } from '@/lib/logger';
import { COOKIES } from '@/shared/auth/cookies';
import type { EPSXJWTPayload } from '@/shared/auth/jwt';
import {
  getDesignBypassAdminPayload,
  isDesignBypassServerEnabled,
} from '@/shared/utils/design-bypass';

/**
 * Enhanced auth user type based on our JWT structure
 */
export interface EnhancedAuthUser extends EPSXJWTPayload {
  id: string; // Maps to `sub` field
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
export function createEnhancedAuthUser(
  payload: EPSXJWTPayload
): EnhancedAuthUser {
  return {
    ...payload,
    id: payload.sub,
  };
}

/**
 * Server-side auth utilities
 */
export async function getServerSession(): Promise<ServerSession | null> {
  if (await isDesignBypassServerEnabled()) {
    const payload = getDesignBypassAdminPayload();
    return {
      user: createEnhancedAuthUser(payload),
      expires: new Date(payload.exp * 1000).toISOString(),
      accessToken: 'design-bypass',
    };
  }

  try {
    const { cookies } = await import('next/headers');
    const { decodeEPSXJWT, isJWTExpired } = await import('@/shared/auth/jwt');

    const cookieStore = await cookies();
    // OIDC Migration: Use only OIDC access token
    const jwt = cookieStore.get(COOKIES.access_token)?.value;

    if (jwt === undefined || jwt === '') {
      return null;
    }

    // Check expiration first
    if (isJWTExpired(jwt)) {
      return null;
    }

    // Decode payload (skipping signature verification due to RS256/HS256 mismatch)
    const payload = decodeEPSXJWT(jwt);
    if (!payload) {
      return null;
    }

    // Convert to proper ServerSession type
    const user = createEnhancedAuthUser(payload);
    return {
      user,
      expires: new Date(payload.exp * 1000).toISOString(),
      accessToken: jwt,
    };
  } catch (_error) {
    logger.auth.error('Failed to get server session', {
      error: String(_error),
    });
    return null;
  }
}

/**
 * Get current user from server session
 */
export async function getCurrentUser(): Promise<EnhancedAuthUser | null> {
  try {
    const session = await getServerSession();
    if (!session?.user) {
      return null;
    }

    return createEnhancedAuthUser(session.user);
  } catch (_error) {
    logger.auth.error('Failed to get current user', { error: String(_error) });
    return null;
  }
}

/**
 * Require admin authentication - throws if not authenticated
 */
export async function requireAdminAuth(): Promise<EnhancedAuthUser> {
  const user = await getCurrentUser();
  if (user === null) {
    throw new Error('Admin authentication required');
  }
  return user;
}

/**
 * Get user context for the current session
 */
export async function getUserContext() {
  try {
    const user = await getCurrentUser();
    if (user === null) {
      return null;
    }

    const platform = user.platform_context ?? user.primary_platform ?? 'epsx';

    return {
      user,
      permissions: user.permissions,
      platform,
    };
  } catch (_error) {
    logger.auth.error('Failed to get user context', { error: String(_error) });
    return null;
  }
}
