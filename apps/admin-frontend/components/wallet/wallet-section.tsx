'use client';

import { updateWalletMetadataAction } from '@/app/wallet-management/actions';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { useWalletListing } from '@/hooks/use-wallet-listing';
import { cn } from '@/lib/utils';
import { Search } from 'lucide-react';
import { DisableWalletModal } from './disable-wallet-modal';
import { EditWalletMetadataModal } from './edit-wallet-metadata-modal';
import { ReenableWalletModal } from './reenable-wallet-modal';
import type { WalletData } from './types';
import { WalletCard } from './wallet-card';
import { WalletFilterBar } from './wallet-filter-bar';

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
    const {
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
    } = useWalletListing({ initialData });

    return (
        <div className={cn("space-y-4", className)}>
            <WalletFilterBar
                filters={filters}
                onFilterChange={setFilters}
                onAddWallet={() => {/* Open Create Modal - To Be Implemented */ }}
            />

            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                {isLoading ? (
                    Array.from({ length: 9 }).map((_, i) => (
                        <Skeleton key={`skeleton-${i}`} className="h-[300px] w-full rounded-[24px]" />
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

            {!isLoading && totalPages > 1 && (
                <div className="flex items-center justify-between pt-4 border-t border-border/40">
                    <p className="text-sm text-muted-foreground">
                        Page {page} of {totalPages}
                    </p>
                    <div className="flex items-center gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage((p) => Math.max(1, p - 1))}
                            disabled={page === 1}
                        >
                            Previous
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                            disabled={page === totalPages}
                        >
                            Next
                        </Button>
                    </div>
                </div>
            )}

            {disableModalWallet !== null && (
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
                    onSuccess={() => { void handleMetadataUpdateSuccess(); }}
                />
            )}
        </div>
    );
}
