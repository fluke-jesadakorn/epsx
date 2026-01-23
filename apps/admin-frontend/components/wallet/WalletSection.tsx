'use client';

import { disableWalletAction, enableWalletAction, fetchWalletsAction, updateWalletMetadataAction } from '@/app/wallet-management/actions';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Plus, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';
import { DisableWalletData, DisableWalletModal } from './DisableWalletModal';
import { EditWalletMetadataModal } from './EditWalletMetadataModal';
import { ReenableWalletData, ReenableWalletModal } from './ReenableWalletModal';
import { WalletCard } from './WalletCard';
import { WalletData, WalletFilters } from './types';

interface WalletSectionProps {
    className?: string;
}

export function WalletSection({ className }: WalletSectionProps) {
    const router = useRouter();
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // State
    const [wallets, setWallets] = useState<WalletData[]>([]);
    const [isLoading, setIsLoading] = useState(true);
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
            const data = await fetchWalletsAction(filters, page, 4); // Limit to 4 for grid view
            setWallets(data.wallets);
            setTotalPages(data.pagination?.total_pages ?? 1);
        } catch (err) {
            console.error('Failed to load wallets:', err);
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
            console.error('Failed to disable wallet:', err);
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
            console.error('Failed to re-enable wallet:', err);
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
            <div className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-card/50 p-1 rounded-xl">
                <div className="flex items-center gap-2 w-full sm:w-auto flex-1">
                    <div className="relative w-full sm:max-w-md">
                        <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="Search by address, name, or label..."
                            className="pl-9 h-10 bg-background/50 border-border/50 focus:bg-background transition-colors"
                            value={filters.search}
                            onChange={(e) => setFilters(prev => ({ ...prev, search: e.target.value }))}
                        />
                    </div>

                    <Select value={filters.status} onValueChange={(v: any) => setFilters(prev => ({ ...prev, status: v }))}>
                        <SelectTrigger className="w-[130px] h-10 bg-background/50 border-border/50">
                            <SelectValue placeholder="Status" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Status</SelectItem>
                            <SelectItem value="active">Active</SelectItem>
                            <SelectItem value="disabled">Disabled</SelectItem>
                            <SelectItem value="pending">Pending</SelectItem>
                        </SelectContent>
                    </Select>

                    <Select value={filters.platform} onValueChange={(v: any) => setFilters(prev => ({ ...prev, platform: v }))}>
                        <SelectTrigger className="w-[130px] h-10 bg-background/50 border-border/50">
                            <SelectValue placeholder="Platform" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Platforms</SelectItem>
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
                        className="w-full sm:w-auto"
                    >
                        <Plus className="h-4 w-4 mr-2" />
                        Add New Wallet
                    </Button>
                </div>
            </div>

            {/* Wallet List */}
            <div className="space-y-2">
                {isLoading ? (
                    Array.from({ length: 5 }).map((_, i) => (
                        <Skeleton key={i} className="h-24 w-full rounded-xl" />
                    ))
                ) : wallets.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-12 text-center border border-dashed border-border rounded-xl bg-muted/5">
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
                    <div className="space-y-3 animate-in fade-in-50 duration-500">
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
                    </div>
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
