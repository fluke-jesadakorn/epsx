/**
 * WALLET HOOKS
 *
 * React hooks for wallet search, lookup, and management.
 * Provides automatic data fetching, caching, and mutation handling.
 */

'use client';

import { useState, useEffect, useCallback } from 'react';
import { useApiClient } from './useApiClient';
import type {
  WalletInfo,
  WalletSearchFilters,
  WalletStats,
  RecentWallet
} from '../api/wallets';
import type { PaginatedResponse } from '../utils/api-client';

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
// WALLET LOOKUP HOOKS
// ============================================================================

/**
 * Get wallet information
 *
 * @example
 * const { data: wallet, loading } = useWallet('0x123...');
 */
export function useWallet(address?: string): UseDataResult<WalletInfo> {
  const { wallets } = useApiClient();
  const [data, setData] = useState<WalletInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchWallet = useCallback(async () => {
    if (!address) {
      setData(null);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const response = await wallets.getWallet(address);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch wallet');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [wallets, address]);

  useEffect(() => {
    fetchWallet();
  }, [fetchWallet]);

  return { data, loading, error, refetch: fetchWallet };
}

/**
 * Search wallets with filters
 *
 * @example
 * const { data: searchResults, loading, refetch } = useWalletSearch({
 *   query: '0x',
 *   tier: 'premium',
 *   limit: 20
 * });
 */
export function useWalletSearch(filters: WalletSearchFilters): UseDataResult<PaginatedResponse<WalletInfo>> {
  const { wallets } = useApiClient();
  const [data, setData] = useState<PaginatedResponse<WalletInfo> | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const searchWallets = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await wallets.searchWallets(filters);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to search wallets');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [wallets, JSON.stringify(filters)]);

  useEffect(() => {
    searchWallets();
  }, [searchWallets]);

  return { data, loading, error, refetch: searchWallets };
}

/**
 * Get recent wallets
 *
 * @example
 * const { data: recentWallets } = useRecentWallets({ limit: 10, days: 7 });
 */
export function useRecentWallets(filters?: { limit?: number; days?: number }): UseDataResult<RecentWallet[]> {
  const { wallets } = useApiClient();
  const [data, setData] = useState<RecentWallet[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchRecentWallets = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await wallets.getRecentWallets(filters);
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch recent wallets');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [wallets, JSON.stringify(filters)]);

  useEffect(() => {
    fetchRecentWallets();
  }, [fetchRecentWallets]);

  return { data, loading, error, refetch: fetchRecentWallets };
}

// ============================================================================
// WALLET MANAGEMENT HOOKS (Admin only)
// ============================================================================

/**
 * Update wallet status
 *
 * @example
 * const { mutate: updateStatus } = useUpdateWalletStatus();
 * await updateStatus({
 *   address: '0x123...',
 *   status: 'suspended',
 *   reason: 'Policy violation'
 * });
 */
export function useUpdateWalletStatus(): UseMutationResult<
  { updated: boolean },
  { address: string; status: 'active' | 'inactive' | 'suspended'; reason?: string }
> {
  const { wallets } = useApiClient();
  const [data, setData] = useState<{ updated: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ address, status, reason }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await wallets.updateWalletStatus(address, status, reason);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to update wallet status');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [wallets]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

/**
 * Update wallet tier
 *
 * @example
 * const { mutate: updateTier } = useUpdateWalletTier();
 * await updateTier({ address: '0x123...', tier: 'premium' });
 */
export function useUpdateWalletTier(): UseMutationResult<
  { updated: boolean },
  { address: string; tier: string }
> {
  const { wallets } = useApiClient();
  const [data, setData] = useState<{ updated: boolean } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ address, tier }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await wallets.updateWalletTier(address, tier);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to update wallet tier');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [wallets]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// STATISTICS HOOKS
// ============================================================================

/**
 * Get wallet statistics
 *
 * @example
 * const { data: stats, loading } = useWalletStats();
 */
export function useWalletStats(): UseDataResult<WalletStats> {
  const { wallets } = useApiClient();
  const [data, setData] = useState<WalletStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchStats = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await wallets.getStats();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch wallet stats');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [wallets]);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  return { data, loading, error, refetch: fetchStats };
}

// ============================================================================
// EXPORTS
// ============================================================================

export default useWallet;
