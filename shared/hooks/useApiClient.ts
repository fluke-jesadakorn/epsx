/**
 * BASE API CLIENT HOOK
 *
 * Platform-aware API client creation with automatic context detection.
 * Provides unified access to all API modules.
 */

'use client';

import { useMemo } from 'react';
import {
  UnifiedApiClient,
  createAdminApiClient,
  createFrontendApiClient
} from '../utils/api-client';

import { createUsersClient, UsersApi } from '../api/users';
import { createPermissionsClient, PermissionsApi } from '../api/permissions';
import { createGroupsClient, GroupsApi } from '../api/groups';
import { createWalletsClient, WalletsApi } from '../api/wallets';
import { createComplianceClient, ComplianceApi } from '../api/compliance';
import { createAnalyticsClient, AnalyticsApi } from '../api/analytics';
import { createAuthClient, AuthApi } from '../api/auth';
import { createNotificationsClient, NotificationsApi } from '../api/notifications';
import { createPlansClient, PlansApi } from '../api/plans';

// ============================================================================
// TYPES
// ============================================================================

export interface ApiClients {
  base: UnifiedApiClient;
  users: UsersApi;
  permissions: PermissionsApi;
  groups: GroupsApi;
  wallets: WalletsApi;
  compliance: ComplianceApi;
  analytics: AnalyticsApi;
  auth: AuthApi;
  notifications: NotificationsApi;
  plans: PlansApi;
}

export type Platform = 'admin' | 'frontend';

// ============================================================================
// PLATFORM DETECTION
// ============================================================================

/**
 * Detect platform from URL or environment
 */
function detectPlatform(): Platform {
  if (typeof window === 'undefined') {
    return 'frontend'; // Default for SSR
  }

  // Check URL path
  const path = window.location.pathname;
  if (path.startsWith('/admin') || window.location.port === '3001') {
    return 'admin';
  }

  // Check environment variable
  if (process.env.NEXT_PUBLIC_PLATFORM === 'admin') {
    return 'admin';
  }

  return 'frontend';
}

// ============================================================================
// HOOK
// ============================================================================

/**
 * Get platform-aware API clients
 *
 * @example
 * // Frontend usage
 * const { users, permissions } = useApiClient();
 *
 * @example
 * // Admin usage (auto-detected)
 * const { wallets, compliance } = useApiClient();
 *
 * @example
 * // Force platform
 * const { users } = useApiClient({ platform: 'admin' });
 */
export function useApiClient(options?: {
  platform?: Platform;
  baseURL?: string;
  token?: string;
}): ApiClients {
  const platform = options?.platform || detectPlatform();

  return useMemo(() => {
    // Create base client
    const baseClient = platform === 'admin'
      ? createAdminApiClient({
          baseURL: options?.baseURL,
          token: options?.token,
          serverSide: false
        })
      : createFrontendApiClient({
          baseURL: options?.baseURL,
          token: options?.token,
          serverSide: false
        });

    // Create domain clients
    return {
      base: baseClient,
      users: createUsersClient(baseClient),
      permissions: createPermissionsClient(baseClient),
      groups: createGroupsClient(baseClient),
      wallets: createWalletsClient(baseClient),
      compliance: createComplianceClient(baseClient),
      analytics: createAnalyticsClient(baseClient),
      auth: createAuthClient(baseClient),
      notifications: createNotificationsClient(baseClient),
      plans: createPlansClient(baseClient)
    };
  }, [platform, options?.baseURL, options?.token]);
}

/**
 * Get admin API clients
 */
export function useAdminApiClient(options?: {
  baseURL?: string;
  token?: string;
}): ApiClients {
  return useApiClient({ ...options, platform: 'admin' });
}

/**
 * Get frontend API clients
 */
export function useFrontendApiClient(options?: {
  baseURL?: string;
  token?: string;
}): ApiClients {
  return useApiClient({ ...options, platform: 'frontend' });
}

// ============================================================================
// EXPORTS
// ============================================================================

export default useApiClient;
