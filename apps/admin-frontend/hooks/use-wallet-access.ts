/**
 * Wallet Access Management Hook
 * Unified hook for monitoring wallet permissions and plans
 */

'use client';

import { useWalletAccessActions } from './use-wallet-access-actions';
import { useWalletAccessData } from './use-wallet-access-data';

// Re-export types for backward compatibility
export type { AccessItem, AccessItemType, UseWalletAccessReturn, WalletAccessData } from '@/types/wallet';

/**
 * Hook for managing wallet permissions and plans
 * @param walletAddress Wallet address to manage access for
 */
export function useWalletAccess(walletAddress: string | null) {
    const { data, isLoading, error, refresh } = useWalletAccessData(walletAddress);
    const actions = useWalletAccessActions(walletAddress, refresh);

    return {
        data,
        isLoading,
        error,
        refresh,
        ...actions,
    };
}

export default useWalletAccess;
