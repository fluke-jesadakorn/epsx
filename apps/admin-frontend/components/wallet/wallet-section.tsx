'use client';

import { disableWalletAction, enableWalletAction, fetchWalletsAction, updateWalletMetadataAction } from '@/app/wallet-management/actions';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { logger } from '@/lib/logger';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Plus, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';
import type { DisableWalletData } from './disable-wallet-modal';
import { DisableWalletModal } from './disable-wallet-modal';
import { EditWalletMetadataModal } from './edit-wallet-metadata-modal';
import type { ReenableWalletData } from './reenable-wallet-modal';
import { ReenableWalletModal } from './reenable-wallet-modal';
import { WalletCard } from './wallet-card';
import type { WalletData, WalletFilters } from './types';

interface WalletSectionProps {
    className?: string;
    initialData?: {
        wallets: WalletData[];
        pagination: {
            page: number;
            limit: number;
            total: number;
            total_pages: number;
        }
    }
}

export function WalletSection({ className, initialData }: WalletSectionProps) {
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
            const data = await fetchWalletsAction(filters, page, 9); // Limit to 9 for grid view
            setWallets(data.wallets);
            setTotalPages(data.pagination?.total_pages ?? 1);
        } catch (err) {
            logger.error('Failed to load wallets:', { err });
        } finally {
            setIsLoading(false);
        }
    }, [filters, page]);

    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            loadData();
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

    return (
        <div className={cn("space-y-4", className)}>
            {/* Filter Bar */}
            <div className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-slate-900/40 backdrop-blur-2xl p-4 rounded-[32px] border border-white/5 shadow-xl">
                <div className="flex items-center gap-3 w-full sm:w-auto flex-1">
                    <div className="relative w-full sm:max-w-md">
                        <Search className="absolute left-4 top-3 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="Search by address, name, or label..."
                            className="pl-11 h-12 bg-white/5 border-white/5 focus:bg-white/10 transition-all rounded-2xl placeholder:text-muted-foreground/50 font-medium"
                            value={filters.search}
                            onChange={(e) => setFilters(prev => ({ ...prev, search: e.target.value }))}
                        />
                    </div>

                    <Select value={filters.status} onValueChange={(v) => setFilters(prev => ({ ...prev, status: v as WalletFilters['status'] }))}>
                        <SelectTrigger className="w-[140px] h-12 bg-white/5 border-white/5 rounded-2xl font-bold text-sm">
                            <SelectValue placeholder="Status" />
                        </SelectTrigger>
                        <SelectContent className="bg-slate-900 border-white/10 rounded-2xl">
                            <SelectItem value="all">All Status</SelectItem>
                            <SelectItem value="active">Active</SelectItem>
                            <SelectItem value="disabled">Disabled</SelectItem>
                            <SelectItem value="pending">Pending</SelectItem>
                        </SelectContent>
                    </Select>

                    <Select value={filters.platform} onValueChange={(v) => setFilters(prev => ({ ...prev, platform: v as WalletFilters['platform'] }))}>
                        <SelectTrigger className="w-[140px] h-12 bg-white/5 border-white/5 rounded-2xl font-bold text-sm">
                            <SelectValue placeholder="Platform" />
                        </SelectTrigger>
                        <SelectContent className="bg-slate-900 border-white/10 rounded-2xl">
                            <SelectItem value="all">Platforms</SelectItem>
                            <SelectItem value="analytics">Analytics</SelectItem>
                            <SelectItem value="pay">Pay</SelectItem>
                            <SelectItem value="token">Token</SelectItem>
                            <SelectItem value="markets">Markets</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <Button
                        onClick={() => {/* Open Create Modal - To Be Implemented */ }}
                        className="w-full sm:w-auto h-12 px-6 bg-[#1fc7d4] hover:bg-[#1fc7d4]/90 text-white font-bold rounded-2xl shadow-lg shadow-cyan-500/20 active:scale-95 transition-all"
                    >
                        <Plus className="h-5 w-5 mr-2" />
                        Add New Wallet
                    </Button>
                </div>
            </div>

            {/* Wallet List */}
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                {isLoading ? (
                    Array.from({ length: 9 }).map((_, i) => (
                        <Skeleton key={i} className="h-[300px] w-full rounded-[24px]" />
                    ))
                ) : wallets.length === 0 ? (
                    <div className="col-span-full flex flex-col items-center justify-center py-12 text-center border border-dashed border-border rounded-xl bg-muted/5">
                        <div className="p-4 rounded-full bg-muted/30 mb-4">
                            <Search className="h-8 w-8 text-muted-foreground/50" />
                        </div>
                        <h3 className="text-lg font-semibold">No wallets found</h3>
                        <p className="text-sm text-muted-foreground max-w-sm mt-2">
                            Try adjusting your filters or search terms.
                        </p>
                        <Button
                            variant="link"
                            onClick={() => setFilters({ search: '', platform: 'all', status: 'all', sortBy: 'created_at', sortOrder: 'desc' })}
                            className="mt-2"
                        >
                            Clear all filters
                        </Button>
                    </div>
                ) : (
                    <>
                        {wallets.map((wallet) => (
                            <WalletCard
                                key={wallet.walletAddress}
                                wallet={wallet}
                                onView={() => router.push(`/wallet-management/${encodeURIComponent(wallet.walletAddress)}`)}
                                onDisable={() => setDisableModalWallet(wallet.walletAddress)}
                                onEnable={() => setReenableModalWallet(wallet)}
                                onEdit={() => setEditMetadataWallet(wallet)}
                                onUpdateMetadata={async (label, note) => {
                                    await updateWalletMetadataAction(wallet.walletAddress, { label, note });
                                    await loadData();
                                }}
                            />
                        ))}
                    </>
                )}
            </div>

            {/* Pagination Footer */}
            {!isLoading && totalPages > 1 && (
                <div className="flex items-center justify-between pt-4 border-t border-border/40">
                    <p className="text-sm text-muted-foreground">
                        Page {page} of {totalPages}
                    </p>
                    <div className="flex items-center gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage(p => Math.max(1, p - 1))}
                            disabled={page === 1}
                        >
                            Previous
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                            disabled={page === totalPages}
                        >
                            Next
                        </Button>
                    </div>
                </div>
            )}

            {/* Modals */}
            {disableModalWallet && (
                <DisableWalletModal
                    walletAddress={disableModalWallet}
                    isOpen={true}
                    onClose={() => setDisableModalWallet(null)}
                    onConfirm={handleDisableWallet}
                    isLoading={isActionLoading}
                />
            )}
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
