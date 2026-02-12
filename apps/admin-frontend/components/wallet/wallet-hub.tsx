'use client';

import { LayoutGrid, List, Plus, RefreshCw, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

import { BulkActionsBar } from './bulk-actions-bar';
import { EditWalletMetadataModal } from './edit-wallet-metadata-modal';
import { ReenableWalletModal, type ReenableWalletData } from './reenable-wallet-modal';
import type {
    WalletData,
    WalletFilters,
    WalletStats
} from './types';
import { WalletCard } from './wallet-card';
import { WalletTable } from './wallet-table';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Skeleton } from '@/components/ui/skeleton';
import { walletMgmt } from '@/lib/api/wallet-management-client';
import { logger } from '@/lib/logger';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';

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

/**
 * WalletHub component
 */
export function WalletHub({ className }: WalletHubProps) {
    const router = useRouter();
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // State
    const [wallets, setWallets] = useState<WalletData[]>([]);
    const [_stats, setStats] = useState<WalletStats>(DEFAULT_STATS);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [viewMode, setViewMode] = useState<'cards' | 'table'>('table');

    // Pagination
    const [page, _setPage] = useState(1);
    const [_totalPages, setTotalPages] = useState(1);

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

    // Modals
    const [reenableModalWallet, setReenableModalWallet] = useState<WalletData | null>(null);
    const [editMetadataWallet, setEditMetadataWallet] = useState<WalletData | null>(null);
    const [isActionLoading, setIsActionLoading] = useState(false);

    // Load data from API
    const loadData = useCallback(async () => {
        setIsLoading(true);
        setError(null);

        try {
            const [statsData, walletsData] = await Promise.all([
                walletMgmt.fetchWalletStats(),
                walletMgmt.fetchWallets(filters, page, 50),
            ]);

            setStats(statsData);
            setWallets(walletsData.wallets);
            setTotalPages(walletsData.pagination?.total_pages ?? 1);
        } catch (err) {
            logger.error('Failed to load wallet data:', { err });
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

    // Filter wallets by platform
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

    // Navigate to disable page
    const handleDisableWallet = (walletAddress: string) => {
        router.push(`/wallet-management/wallets/${encodeURIComponent(walletAddress)}/disable`);
    };

    const handleReenableWallet = async (data: ReenableWalletData) => {
        setIsActionLoading(true);
        try {
            await walletMgmt.enableWallet(data.walletAddress, {
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

    // Auth check
    if (authLoading) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="text-center">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4" />
                    <p className="text-muted-foreground">Checking authentication...</p>
                </div>
            </div>
        );
    }

    if (!isAuthenticated) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="text-center max-w-md">
                    <div className="text-4xl mb-4">🔐</div>
                    <h3 className="text-lg font-semibold text-foreground mb-2">
                        Authentication Required
                    </h3>
                    <p className="text-muted-foreground">
                        Please connect your wallet to access the wallet management hub.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className={cn('space-y-4', className)}>
            {/* Unified Toolbar: Search + Filters + Actions */}
            <div className="flex flex-col gap-4 p-4 bg-card border border-border/60 rounded-xl shadow-sm">
                <div className="flex flex-col lg:flex-row gap-4">
                    {/* Search & Basic Filters */}
                    <div className="flex flex-1 flex-col sm:flex-row gap-3">
                        <div className="relative flex-1">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                            <Input
                                placeholder="Search address or label..."
                                value={filters.search}
                                onChange={(e) => setFilters((prev) => ({ ...prev, search: e.target.value }))}
                                className="pl-10 h-10 bg-muted/30 border-border/50 focus:bg-background transition-colors"
                            />
                        </div>

                        <div className="flex gap-2">
                            <Select
                                value={filters.status}
                                onValueChange={(v) => setFilters((prev) => ({ ...prev, status: v as WalletFilters['status'] | 'all' }))}
                            >
                                <SelectTrigger className="w-32 h-10 bg-muted/30 border-border/50">
                                    <SelectValue placeholder="Status" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="all">All Status</SelectItem>
                                    <SelectItem value="active">🟢 Active</SelectItem>
                                    <SelectItem value="disabled">⚠️ Disabled</SelectItem>
                                    <SelectItem value="pending">⏳ Pending</SelectItem>
                                </SelectContent>
                            </Select>

                            <Select
                                value={filters.sortBy}
                                onValueChange={(v) => setFilters((prev) => ({ ...prev, sortBy: v as WalletFilters['sortBy'] }))}
                            >
                                <SelectTrigger className="w-36 h-10 bg-muted/30 border-border/50">
                                    <SelectValue placeholder="Sort" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="created_at">Date Created</SelectItem>
                                    <SelectItem value="last_auth_at">Last Active</SelectItem>
                                    <SelectItem value="wallet_address">Address</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>

                    <Separator orientation="vertical" className="hidden lg:block h-10" />

                    {/* Quick Actions & View Toggle */}
                    <div className="flex items-center gap-3">
                        <div className="flex items-center gap-1.5 p-1 bg-muted/50 rounded-lg border border-border/50">
                            <Button
                                size="sm"
                                variant="ghost"
                                className="h-8 px-3 gap-2 text-primary hover:bg-primary/10 hover:text-primary"
                                onClick={() => router.push('/subscriptions/plans/new')}
                            >
                                <Plus className="h-3.5 w-3.5" />
                                <span className="text-xs font-semibold">Plan</span>
                            </Button>
                            <Separator orientation="vertical" className="h-4" />
                            <Button
                                size="sm"
                                variant="ghost"
                                className="h-8 px-3 gap-2 text-secondary hover:bg-secondary/10 hover:text-secondary"
                                onClick={() => router.push('/wallet-management/groups/new')}
                            >
                                <Plus className="h-3.5 w-3.5" />
                                <span className="text-xs font-semibold">Group</span>
                            </Button>
                        </div>

                        <Separator orientation="vertical" className="h-8 hidden sm:block" />

                        <div className="flex items-center gap-2 bg-muted/50 border border-border/50 p-1 rounded-lg">
                            <Button
                                size="sm"
                                variant={viewMode === 'table' ? 'secondary' : 'ghost'}
                                className="h-7 w-9 p-0"
                                onClick={() => setViewMode('table')}
                                aria-label="Table View"
                            >
                                <List className="h-3.5 w-3.5" />
                            </Button>
                            <Button
                                size="sm"
                                variant={viewMode === 'cards' ? 'secondary' : 'ghost'}
                                className="h-7 w-9 p-0"
                                onClick={() => setViewMode('cards')}
                                aria-label="Card View"
                            >
                                <LayoutGrid className="h-3.5 w-3.5" />
                            </Button>
                        </div>

                        <Button
                            variant="outline"
                            size="icon"
                            onClick={loadData}
                            disabled={isLoading}
                            className="h-9 w-9 border-border/50 bg-muted/30"
                        >
                            <RefreshCw className={cn('h-3.5 w-3.5', isLoading && 'animate-spin')} />
                        </Button>
                    </div>
                </div>

                {/* Platform Quick Filter */}
                <div className="flex flex-wrap gap-2 pt-1">
                    <Button
                        variant={filters.platform === 'all' ? 'default' : 'outline'}
                        size="sm"
                        onClick={() => setFilters(prev => ({ ...prev, platform: 'all' }))}
                        className="h-7 text-[10px] uppercase font-bold tracking-wider px-3"
                    >
                        All Platforms
                    </Button>
                    {(['analytics', 'pay', 'token', 'markets'] as const).map(p => (
                        <Button
                            key={p}
                            variant={filters.platform === p ? 'default' : 'outline'}
                            size="sm"
                            onClick={() => setFilters(prev => ({ ...prev, platform: p }))}
                            className="h-7 text-[10px] uppercase font-bold tracking-wider px-3"
                        >
                            {p}
                        </Button>
                    ))}
                </div>
            </div>

            {/* Error */}
            {error && (
                <div className="p-4 rounded-xl bg-destructive/10 border border-destructive/30 text-destructive text-sm flex items-center gap-2">
                    <span>⚠️</span>
                    {error}
                </div>
            )}

            {/* Wallet List Area */}
            <div className="min-h-[400px]">
                {isLoading ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {Array.from({ length: 6 }).map((_, i) => (
                            <Skeleton key={i} className="h-32 rounded-xl" />
                        ))}
                    </div>
                ) : filteredWallets.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-20 bg-muted/20 border border-dashed border-border/60 rounded-2xl">
                        <Search className="h-12 w-12 text-muted-foreground/30 mb-4" />
                        <p className="font-semibold text-muted-foreground">No wallets found</p>
                        <p className="text-sm text-muted-foreground/60 mt-1">Try refining your search or filters</p>
                    </div>
                ) : viewMode === 'table' ? (
                    <WalletTable
                        wallets={filteredWallets}
                        selectedAddresses={selectedWallets}
                        onSelectWallet={handleSelectWallet}
                        onView={handleViewWallet}
                        onManage={handleViewWallet}
                        onDisable={(w) => handleDisableWallet(w.walletAddress)}
                        onEnable={(w) => setReenableModalWallet(w)}
                        onEdit={(w) => setEditMetadataWallet(w)}
                    />
                ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                        {filteredWallets.map((wallet) => (
                            <WalletCard
                                key={wallet.walletAddress}
                                wallet={wallet}
                                isSelected={selectedWallets.has(wallet.walletAddress)}
                                onSelect={(selected) => handleSelectWallet(wallet.walletAddress, selected)}
                                onView={() => handleViewWallet(wallet)}
                                onManage={() => handleViewWallet(wallet)}
                                onDisable={() => handleDisableWallet(wallet.walletAddress)}
                                onEnable={() => setReenableModalWallet(wallet)}
                                onEdit={() => setEditMetadataWallet(wallet)}
                                onUpdateMetadata={async (label, note) => {
                                    await walletMgmt.updateWalletMetadata(wallet.walletAddress, { label, note });
                                    await loadData();
                                }}
                            />
                        ))}
                    </div>
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

            {/* Re-enable Modal */}
            {reenableModalWallet?.disableInfo && (
                <ReenableWalletModal
                    walletAddress={reenableModalWallet.walletAddress}
                    disableInfo={reenableModalWallet.disableInfo}
                    isOpen={true}
                    onClose={() => setReenableModalWallet(null)}
                    onConfirm={handleReenableWallet}
                    isLoading={isActionLoading}
                />
            )}

            {/* Edit Metadata Modal */}
            {editMetadataWallet && (
                <EditWalletMetadataModal
                    walletAddress={editMetadataWallet.walletAddress}
                    currentLabel={editMetadataWallet.label}
                    currentNote={editMetadataWallet.note}
                    isOpen={true}
                    onClose={() => setEditMetadataWallet(null)}
                    onSuccess={handleMetadataUpdateSuccess}
                />
            )}
        </div>
    );
}
