/**
 * PERMISSION HOOKS
 *
 * React hooks for permission management and queries.
 * Provides automatic data fetching, caching, and mutation handling.
 */

'use client';

import { useState, useEffect, useCallback } from 'react';
import { useApiClient } from './useApiClient';
import type {
  Permission,
  UserPermissionsResponse,
  GrantPermissionRequest,
  RevokePermissionRequest,
  PermissionStats
} from '../api/permissions';

// ============================================================================
// TYPES
// ============================================================================

interface UseDataResult<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

interface UseMutationResult<TData, TVariables> {
  mutate: (variables: TVariables) => Promise<TData>;
  data: TData | null;
  loading: boolean;
  error: Error | null;
  reset: () => void;
}

// ============================================================================
// PERMISSION QUERY HOOKS
// ============================================================================

/**
 * Get current user's permissions
 *
 * @example
 * const { data: permissions, loading } = useCurrentUserPermissions();
 */
export function useCurrentUserPermissions(): UseDataResult<UserPermissionsResponse> {
  const { permissions } = useApiClient();
  const [data, setData] = useState<UserPermissionsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchPermissions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await permissions.getCurrentUserPermissions();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch permissions');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [permissions]);

  useEffect(() => {
    fetchPermissions();
  }, [fetchPermissions]);

  return { data, loading, error, refetch: fetchPermissions };
}

/**
 * Get permissions for specific wallet (Admin only)
 *
 * @example
 * const { data, loading } = useWalletPermissions('0x123...');
 */
export function useWalletPermissions(walletAddress?: string): UseDataResult<UserPermissionsResponse> {
  const { permissions } = useApiClient();
  const [data, setData] = useState<UserPermissionsResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchWalletPermissions = useCallback(async () => {
    if (!walletAddress) {
      setData(null);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const response = await permissions.getWalletPermissions(walletAddress);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch wallet permissions');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [permissions, walletAddress]);

  useEffect(() => {
    fetchWalletPermissions();
  }, [fetchWalletPermissions]);

  return { data, loading, error, refetch: fetchWalletPermissions };
}

/**
 * Get permission statistics (Admin only)
 *
 * @example
 * const { data: stats } = usePermissionStats();
 */
export function usePermissionStats(): UseDataResult<PermissionStats> {
  const { permissions } = useApiClient();
  const [data, setData] = useState<PermissionStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchStats = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await permissions.getStats();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch stats');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [permissions]);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  return { data, loading, error, refetch: fetchStats };
}

// ============================================================================
// PERMISSION MUTATION HOOKS (Admin only)
// ============================================================================

/**
 * Grant permission to wallet
 *
 * @example
 * const { mutate: grantPermission, loading } = useGrantPermission();
 * await grantPermission({
 *   wallet_address: '0x123...',
 *   permission: 'epsx:analytics:view'
 * });
 */
export function useGrantPermission(): UseMutationResult<
  { granted: boolean; permission: Permission },
  GrantPermissionRequest
> {
  const { permissions } = useApiClient();
  const [data, setData] = useState<{ granted: boolean; permission: Permission } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (request: GrantPermissionRequest) => {
    try {
      setLoading(true);
      setError(null);
      const response = await permissions.grantPermission(request);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to grant permission');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [permissions]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

/**
 * Revoke permission from wallet
 *
 * @example
 * const { mutate: revokePermission } = useRevokePermission();
 * await revokePermission({
 *   wallet_address: '0x123...',
 *   permission: 'epsx:analytics:view'
 * });
 */
export function useRevokePermission(): UseMutationResult<{ revoked: boolean }, RevokePermissionRequest> {
  const { permissions } = useApiClient();
  const [data, setData] = useState<{ revoked: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (request: RevokePermissionRequest) => {
    try {
      setLoading(true);
      setError(null);
      const response = await permissions.revokePermission(request);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to revoke permission');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [permissions]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

/**
 * Check if wallet has permission
 *
 * @example
 * const { mutate: checkPermission } = useCheckPermission();
 * const result = await checkPermission({
 *   wallet_address: '0x123...',
 *   permission: 'epsx:analytics:view'
 * });
 */
export function useCheckPermission(): UseMutationResult<
  { has_permission: boolean; reason?: string },
  { wallet_address: string; permission: string }
> {
  const { permissions } = useApiClient();
  const [data, setData] = useState<{ has_permission: boolean; reason?: string } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ wallet_address, permission }: { wallet_address: string; permission: string }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await permissions.checkPermission(wallet_address, permission);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to check permission');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [permissions]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// HELPER HOOK - Permission Display
// ============================================================================

/**
 * Simple permission display helper
 * For UI display only - NOT for authorization decisions
 *
 * @example
 * const { hasPermission, loading } = usePermissionDisplay();
 * const canView = hasPermission('epsx:analytics:view');
 */
export function usePermissionDisplay() {
  const { data: permissionsData, loading } = useCurrentUserPermissions();

  const hasPermission = useCallback(
    (permission: string): boolean => {
      if (!permissionsData) return false;
      return permissionsData.effective_permissions.includes(permission);
    },
    [permissionsData]
  );

  const hasAnyPermission = useCallback(
    (permissions: string[]): boolean => {
      if (!permissionsData) return false;
      return permissions.some(p => permissionsData.effective_permissions.includes(p));
    },
    [permissionsData]
  );

  const hasAllPermissions = useCallback(
    (permissions: string[]): boolean => {
      if (!permissionsData) return false;
      return permissions.every(p => permissionsData.effective_permissions.includes(p));
    },
    [permissionsData]
  );

  return {
    loading,
    permissions: permissionsData?.effective_permissions || [],
    tier: permissionsData?.tier,
    hasPermission,
    hasAnyPermission,
    hasAllPermissions
  };
}

// ============================================================================
// EXPORTS
// ============================================================================

export default useCurrentUserPermissions;
