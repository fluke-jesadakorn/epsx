/**
 * BASE API CLIENT HOOK
 *
 * Platform-aware API client creation with automatic context detection.
 * Provides unified access to all API modules.
 */

'use client';

import { usePathname } from 'next/navigation';
import { useMemo } from 'react';
import type {
  UnifiedApiClient
} from '../utils/api-client';
import {
  createAdminApiClient,
  createFrontendApiClient
} from '../utils/api-client';

import type { AnalyticsApi } from '../api/analytics';
import { createAnalyticsClient } from '../api/analytics';
import type { AuthApi } from '../api/auth';
import { createAuthClient } from '../api/auth';
import type { ComplianceApi } from '../api/compliance';
import { createComplianceClient } from '../api/compliance';
import type { NotificationsApi } from '../api/notifications';
import { createNotificationsClient } from '../api/notifications';
import type { PermissionsApi } from '../api/permissions';
import { createPermissionsClient } from '../api/permissions';
import type { PlansApi } from '../api/plans';
import { createPlansClient } from '../api/plans';
import type { UsersApi } from '../api/users';
import { createUsersClient } from '../api/users';
import type { WalletsApi } from '../api/wallets';
import { createWalletsClient } from '../api/wallets';

// ============================================================================
// TYPES
// ============================================================================

export interface ApiClients {
  base: UnifiedApiClient;
  users: UsersApi;
  permissions: PermissionsApi;
  plans: PlansApi;
  wallets: WalletsApi;
  compliance: ComplianceApi;
  analytics: AnalyticsApi;
  auth: AuthApi;
  notifications: NotificationsApi;
}

export type Platform = 'admin' | 'frontend';

// ============================================================================
// PLATFORM DETECTION
// ============================================================================

// No longer using global detectPlatform that uses window.location.pathname

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
  const pathname = usePathname();

  const platform = useMemo(() => {
    if (options?.platform) { return options.platform; }

    // Check URL path via usePathname
    if (pathname.startsWith('/admin')) {
      return 'admin';
    }

    // Fallback to window.location.port if available (next/navigation doesn't have port)
    if (typeof window !== 'undefined' && window.location.port === '3001') {
      return 'admin';
    }

    // Check environment variable
    if (process.env.NEXT_PUBLIC_PLATFORM === 'admin') {
      return 'admin';
    }

    return 'frontend';
  }, [options?.platform, pathname]);

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
      plans: createPlansClient(baseClient),
      wallets: createWalletsClient(baseClient),
      compliance: createComplianceClient(baseClient),
      analytics: createAnalyticsClient(baseClient),
      auth: createAuthClient(baseClient),
      notifications: createNotificationsClient(baseClient),
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
