/**
 * Wallet Hub Component
 * Main unified hub for wallet management
 */
'use client';

import { RefreshCw, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { walletMgmt } from '@/lib/api/wallet-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';

import { BulkActionsBar } from './BulkActionsBar';
import { DisableWalletModal, type DisableWalletData } from './DisableWalletModal';
import { ReenableWalletModal, type ReenableWalletData } from './ReenableWalletModal';
import { WalletCard } from './WalletCard';
import { WalletPlatformFilter } from './WalletPlatformFilter';
import { WalletStatsBar } from './WalletStatsBar';
import type {
    WalletData,
    WalletFilters,
    WalletStats
} from './types';

interface WalletHubProps {
    className?: string;
}

// Default empty stats for loading state
const DEFAULT_STATS: WalletStats = {
    total: 0,
    active: 0,
    disabled: 0,
    subscribed: 0,
    changes: { total: 0, active: 0, disabled: 0, subscribed: 0 },
    platformDistribution: {
        analytics: 0,
        pay: 0,
        token: 0,
        markets: 0,
    },
};

export function WalletHub({ className }: WalletHubProps) {
    const router = useRouter();
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // State
    const [wallets, setWallets] = useState<WalletData[]>([]);
    const [stats, setStats] = useState<WalletStats>(DEFAULT_STATS);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Pagination
    const [page, setPage] = useState(1);
    const [totalPages, setTotalPages] = useState(1);

    // Filters
    const [filters, setFilters] = useState<WalletFilters>({
        search: '',
        platform: 'all',
        status: 'all',
        sortBy: 'created_at',
        sortOrder: 'desc',
    });

    // Selection
    const [selectedWallets, setSelectedWallets] = useState<Set<string>>(new Set());

    // Note: Detail view is now handled via router navigation to /wallet-management/[address]

    // Modals
    const [disableModalWallet, setDisableModalWallet] = useState<string | null>(null);
    const [reenableModalWallet, setReenableModalWallet] = useState<WalletData | null>(null);
    const [isActionLoading, setIsActionLoading] = useState(false);

    // Load data from API
    const loadData = useCallback(async () => {
        setIsLoading(true);
        setError(null);

        try {
            const [statsData, walletsData] = await Promise.all([
                walletMgmt.fetchWalletStats(),
                walletMgmt.fetchWallets(filters, page, 20),
            ]);

            setStats(statsData);
            setWallets(walletsData.wallets);
            setTotalPages(walletsData.pagination?.total_pages ?? 1);
        } catch (err) {
            console.error('Failed to load wallet data:', err);
            setError(err instanceof Error ? err.message : 'Failed to load data');
        } finally {
            setIsLoading(false);
        }
    }, [filters, page]);

    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            loadData();
        }
    }, [isAuthenticated, authLoading, loadData]);



    // Filter wallets by platform (client-side since backend might not support yet)
    const filteredWallets = wallets.filter((wallet) => {
        if (filters.platform !== 'all' && !wallet.platforms.includes(filters.platform)) {
            return false;
        }
        return true;
    });

    // Selection handlers
    const handleSelectWallet = (address: string, selected: boolean) => {
        setSelectedWallets((prev) => {
            const next = new Set(prev);
            if (selected) {
                next.add(address);
            } else {
                next.delete(address);
            }
            return next;
        });
    };

    const handleClearSelection = () => {
        setSelectedWallets(new Set());
    };

    // Navigate to wallet detail page
    const handleViewWallet = (wallet: WalletData) => {
        router.push(`/wallet-management/${encodeURIComponent(wallet.walletAddress)}`);
    };


    // Disable/Enable handlers
    const handleDisableWallet = async (data: DisableWalletData) => {
        setIsActionLoading(true);
        try {
            // TODO: Call API
            await new Promise(resolve => setTimeout(resolve, 1000));
            setDisableModalWallet(null);
            await loadData();
        } finally {
            setIsActionLoading(false);
        }
    };

    const handleReenableWallet = async (data: ReenableWalletData) => {
        setIsActionLoading(true);
        try {
            // TODO: Call API
            await new Promise(resolve => setTimeout(resolve, 1000));
            setReenableModalWallet(null);
            await loadData();
        } finally {
            setIsActionLoading(false);
        }
    };

    // Auth check
    if (authLoading) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="text-center">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4" />
                    <p className="text-gray-600 dark:text-gray-400">Checking authentication...</p>
                </div>
            </div>
        );
    }

    if (!isAuthenticated) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="text-center max-w-md">
                    <div className="text-4xl mb-4">🔐</div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                        Authentication Required
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                        Please connect your wallet to access the wallet management hub.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className={cn('space-y-6', className)}>
            {/* Platform Filter */}
            <WalletPlatformFilter
                value={filters.platform}
                onChange={(platform) => setFilters((prev) => ({ ...prev, platform }))}
            />

            {/* Stats Dashboard */}
            <WalletStatsBar stats={stats} isLoading={isLoading && stats.total === 0} />

            {/* Search & Filters */}
            <div className="rounded-2xl bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700 p-4">
                <div className="flex flex-col sm:flex-row gap-4">
                    {/* Search */}
                    <div className="flex-1">
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                            <Input
                                placeholder="Search wallet address..."
                                value={filters.search}
                                onChange={(e) => setFilters((prev) => ({ ...prev, search: e.target.value }))}
                                className="pl-10"
                            />
                        </div>
                    </div>

                    {/* Status Filter */}
                    <Select
                        value={filters.status}
                        onValueChange={(v) => setFilters((prev) => ({ ...prev, status: v as any }))}
                    >
                        <SelectTrigger className="w-40">
                            <SelectValue placeholder="Status" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Status</SelectItem>
                            <SelectItem value="active">🟢 Active</SelectItem>
                            <SelectItem value="disabled">⚠️ Disabled</SelectItem>
                            <SelectItem value="pending">⏳ Pending</SelectItem>
                        </SelectContent>
                    </Select>

                    {/* Sort */}
                    <Select
                        value={filters.sortBy}
                        onValueChange={(v) => setFilters((prev) => ({ ...prev, sortBy: v as any }))}
                    >
                        <SelectTrigger className="w-40">
                            <SelectValue placeholder="Sort by" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="created_at">Date Created</SelectItem>
                            <SelectItem value="last_auth_at">Last Active</SelectItem>
                            <SelectItem value="wallet_address">Address</SelectItem>
                        </SelectContent>
                    </Select>

                    {/* Refresh */}
                    <Button
                        variant="outline"
                        onClick={loadData}
                        disabled={isLoading}
                        className="gap-2"
                    >
                        <RefreshCw className={cn('h-4 w-4', isLoading && 'animate-spin')} />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Error */}
            {error && (
                <div className="p-4 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400">
                    ⚠️ {error}
                </div>
            )}

            {/* Wallet List */}
            <div className="space-y-3">
                {isLoading ? (
                    // Loading skeleton
                    Array.from({ length: 5 }).map((_, i) => (
                        <div key={i} className="rounded-2xl bg-gray-100 dark:bg-gray-800 p-6 animate-pulse">
                            <div className="flex items-center gap-4">
                                <Skeleton className="h-12 w-12 rounded-xl" />
                                <div className="space-y-2 flex-1">
                                    <Skeleton className="h-4 w-40" />
                                    <Skeleton className="h-3 w-32" />
                                </div>
                                <Skeleton className="h-9 w-20" />
                            </div>
                        </div>
                    ))
                ) : filteredWallets.length === 0 ? (
                    <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                        <Search className="h-12 w-12 mx-auto mb-4 opacity-30" />
                        <p className="font-medium">No wallets found</p>
                        <p className="text-sm mt-1">Try adjusting your filters</p>
                    </div>
                ) : (
                    filteredWallets.map((wallet) => (
                        <WalletCard
                            key={wallet.walletAddress}
                            wallet={wallet}
                            isSelected={selectedWallets.has(wallet.walletAddress)}
                            onSelect={(selected) => handleSelectWallet(wallet.walletAddress, selected)}
                            onView={() => handleViewWallet(wallet)}
                            onManage={() => handleViewWallet(wallet)}
                            onDisable={() => setDisableModalWallet(wallet.walletAddress)}
                            onEnable={() => setReenableModalWallet(wallet)}
                        />
                    ))
                )}
            </div>

            {/* Bulk Actions Bar */}
            <BulkActionsBar
                selectedCount={selectedWallets.size}
                onClearSelection={handleClearSelection}
                onAddPermission={() => { }}
                onRemovePermission={() => { }}
                onDisable={() => { }}
                onNotify={() => { }}
            />


            {/* Disable Modal */}
            {disableModalWallet && (
                <DisableWalletModal
                    walletAddress={disableModalWallet}
                    isOpen={true}
                    onClose={() => setDisableModalWallet(null)}
                    onConfirm={handleDisableWallet}
                    isLoading={isActionLoading}
                />
            )}

            {/* Re-enable Modal */}
            {reenableModalWallet && reenableModalWallet.disableInfo && (
                <ReenableWalletModal
                    walletAddress={reenableModalWallet.walletAddress}
                    disableInfo={reenableModalWallet.disableInfo}
                    isOpen={true}
                    onClose={() => setReenableModalWallet(null)}
                    onConfirm={handleReenableWallet}
                    isLoading={isActionLoading}
                />
            )}
        </div>
    );
}
