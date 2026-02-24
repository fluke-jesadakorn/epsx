'use client';

import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { useWalletListing } from '@/hooks/use-wallet-listing';
import { cn } from '@/lib/utils';
import { BarChart3, Clock, CreditCard, Search, User } from 'lucide-react';
import { EditWalletMetadataModal } from './edit-wallet-metadata-modal';
import { ReenableWalletModal } from './reenable-wallet-modal';
import type { WalletData } from './types';
import { WalletFilterBar } from './wallet-filter-bar';
import { WalletListRow } from './wallet-list-row';

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

interface WalletEmptyStateProps {
    onClearFilters: () => void;
}

function WalletEmptyState({ onClearFilters }: WalletEmptyStateProps) {
    return (
        <div className="flex flex-col items-center justify-center py-16 text-center border border-dashed border-border rounded-xl bg-muted/5">
            <div className="p-4 rounded-full bg-muted/30 mb-4">
                <Search className="h-8 w-8 text-muted-foreground/50" />
            </div>
            <h3 className="text-lg font-semibold">No wallets found</h3>
            <p className="text-sm text-muted-foreground max-w-sm mt-2">
                Try adjusting your filters or search terms.
            </p>
            <Button variant="link" onClick={onClearFilters} className="mt-2">
                Clear all filters
            </Button>
        </div>
    );
}

function ListHeader() {
    return (
        <div className="flex items-center gap-3 px-4 py-2 border-b border-border/60 bg-muted/30 text-xs font-medium text-muted-foreground rounded-t-lg">
            <div className="flex-1 min-w-0">Wallet</div>
            <div className="hidden sm:flex items-center gap-1.5 w-32 shrink-0">
                <CreditCard size={11} /> Plan
            </div>
            <div className="hidden md:flex items-center gap-1.5 w-24 shrink-0">
                <BarChart3 size={11} /> Platforms
            </div>
            <div className="hidden sm:block w-20 shrink-0">Status</div>
            <div className="hidden lg:flex items-center gap-1.5 w-24 shrink-0">
                <User size={11} /> Joined
            </div>
            <div className="hidden xl:flex items-center gap-1.5 w-24 shrink-0">
                <Clock size={11} /> Last Login
            </div>
            <div className="w-20 shrink-0" />
        </div>
    );
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
        disableModalWallet: _disableModalWallet,
        setDisableModalWallet: _setDisableModalWallet,
        reenableModalWallet,
        setReenableModalWallet,
        editMetadataWallet,
        setEditMetadataWallet,
        isActionLoading,
        loadData,
        handleDisableWallet: _handleDisableWallet,
        handleReenableWallet,
        handleMetadataUpdateSuccess,
        router
    } = useWalletListing({ initialData });

    return (
        <div className={cn("space-y-4", className)}>
            <WalletFilterBar
                filters={filters}
                onFilterChange={setFilters}
            />

            <div className="border border-border/60 rounded-lg overflow-hidden">
                <ListHeader />
                {isLoading ? (
                    Array.from({ length: 9 }).map((_, i) => (
                        // eslint-disable-next-line react/no-array-index-key
                        <Skeleton key={`skeleton-${i}`} className="h-14 w-full rounded-none border-b border-border/40 last:border-0" />
                    ))
                ) : wallets.length === 0 ? (
                    <WalletEmptyState onClearFilters={() => setFilters({ search: '', platform: 'all', status: 'all', sortBy: 'created_at', sortOrder: 'desc' })} />
                ) : wallets.map((wallet) => (
                    <WalletListRow
                        key={wallet.walletAddress}
                        wallet={wallet}
                        onView={() => router.push(`/wallet-management/${encodeURIComponent(wallet.walletAddress)}`)}
                        onEnable={() => setReenableModalWallet(wallet)}
                    />
                ))}
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

            {reenableModalWallet !== null && (
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
