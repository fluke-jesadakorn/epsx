'use client';

import { disableWalletAction, enableWalletAction, fetchWalletsAction } from '@/app/wallet-management/actions';
import type { DisableWalletData } from '@/components/wallet/disable-wallet-modal';
import type { ReenableWalletData } from '@/components/wallet/reenable-wallet-modal';
import type { WalletData, WalletFilters } from '@/components/wallet/types';
import { logger } from '@/lib/logger';
import { useSharedAuth } from '@/shared/components/auth/provider';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

interface UseWalletListingProps {
    initialData?: {
        wallets: WalletData[];
        pagination: {
            page: number;
            limit: number;
            total: number;
            total_pages: number;
        }
    };
}

export function useWalletListing({ initialData }: UseWalletListingProps) {
    const router = useRouter();
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // State
    const [wallets, setWallets] = useState<WalletData[]>(initialData?.wallets ?? []);
    const [isLoading, setIsLoading] = useState(!initialData);
    const [filters, setFilters] = useState<WalletFilters>({
        search: '',
        platform: 'all',
        status: 'all',
        sortBy: 'created_at',
        sortOrder: 'desc',
    });

    const [page, setPage] = useState(1);
    const [totalPages, setTotalPages] = useState(1);

    // Modals
    const [disableModalWallet, setDisableModalWallet] = useState<string | null>(null);
    const [reenableModalWallet, setReenableModalWallet] = useState<WalletData | null>(null);
    const [editMetadataWallet, setEditMetadataWallet] = useState<WalletData | null>(null);
    const [isActionLoading, setIsActionLoading] = useState(false);

    // Load Data
    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            const res = await fetchWalletsAction(filters, page, 9);
            if (!res.success) {
                logger.error('Failed to load wallets:', { error: res.error });
                return;
            }
            setWallets(res.wallets);
            setTotalPages(res.pagination.total_pages);
        } catch (err) {
            logger.error('Failed to load wallets:', { err });
        } finally {
            setIsLoading(false);
        }
    }, [filters, page]);

    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            void loadData();
        }
    }, [isAuthenticated, authLoading, loadData]);

    // Handlers
    const handleDisableWallet = async (data: DisableWalletData) => {
        setIsActionLoading(true);
        try {
            await disableWalletAction(data.walletAddress, {
                duration_days: data.duration === 'until_manual' ? null : data.duration,
                reason_category: data.reasonCategory,
                reason_details: data.reasonDetails,
                affected_platforms: data.affectedPlatforms,
                block_login: data.blockLogin,
                pause_subscriptions: data.pauseSubscriptions,
                notify_user: data.notifyUser,
            });
            setDisableModalWallet(null);
            await loadData();
        } catch (err) {
            logger.error('Failed to disable wallet:', { err });
        } finally {
            setIsActionLoading(false);
        }
    };

    const handleReenableWallet = async (data: ReenableWalletData) => {
        setIsActionLoading(true);
        try {
            await enableWalletAction(data.walletAddress, {
                platforms_to_enable: data.platformsToEnable,
                restore_permissions: data.restorePermissions,
                resume_subscriptions: data.resumeSubscriptions,
                resolution_note: data.resolutionNote,
            });
            setReenableModalWallet(null);
            await loadData();
        } catch (err) {
            logger.error('Failed to re-enable wallet:', { err });
        } finally {
            setIsActionLoading(false);
        }
    };

    const handleMetadataUpdateSuccess = async () => {
        setEditMetadataWallet(null);
        await loadData();
    };

    return {
        wallets,
        isLoading,
        filters,
        setFilters,
        page,
        setPage,
        totalPages,
        disableModalWallet,
        setDisableModalWallet,
        reenableModalWallet,
        setReenableModalWallet,
        editMetadataWallet,
        setEditMetadataWallet,
        isActionLoading,
        loadData,
        handleDisableWallet,
        handleReenableWallet,
        handleMetadataUpdateSuccess,
        router
    };
}
