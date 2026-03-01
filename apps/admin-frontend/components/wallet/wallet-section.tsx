'use client';

import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { useWalletListing } from '@/hooks/use-wallet-listing';
import { cn } from '@/lib/utils';
import { BarChart3, ChevronLeft, ChevronRight, Clock, CreditCard, Search, User, Wallet } from 'lucide-react';
import { useState } from 'react';
import { EditWalletMetadataModal } from './edit-wallet-metadata-modal';
import { ReenableWalletModal } from './reenable-wallet-modal';
import type { WalletData } from './types';
import { WalletFilterBar } from './wallet-filter-bar';
import { WalletListRow, WalletMobileCard } from './wallet-list-row';

const COL_HDR = 'text-[10px] font-bold uppercase tracking-[0.15em] text-muted-foreground/70';

function PageHeader({ total, isLoading }: { total: number; isLoading: boolean }) {
    return (
        <div className="rounded-2xl bg-card border border-border/20 shadow-xl overflow-hidden">
            <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
            <div className="px-5 py-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="h-[3px] w-8 bg-[#1fc7d4] rounded-full" />
                    <Wallet className="w-5 h-5 text-[#1fc7d4]" />
                    <h1 className="text-xl font-bold text-foreground">Wallets</h1>
                </div>
                {!isLoading && total > 0 && (
                    <span className="px-2.5 py-1 rounded-full bg-[#1fc7d4]/10 text-[#1fc7d4] text-xs font-bold border border-[#1fc7d4]/20">
                        {total} {total === 1 ? 'wallet' : 'wallets'}
                    </span>
                )}
            </div>
        </div>
    );
}

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
        <div className="flex items-center gap-3 px-4 py-2.5 border-b border-border/20 bg-muted/20 text-xs">
            <div className={cn('flex-1 min-w-0', COL_HDR)}>Wallet</div>
            <div className={cn('hidden sm:flex items-center gap-1.5 w-32 shrink-0', COL_HDR)}>
                <CreditCard size={10} /> Plan
            </div>
            <div className={cn('hidden md:flex items-center gap-1.5 w-24 shrink-0', COL_HDR)}>
                <BarChart3 size={10} /> Platforms
            </div>
            <div className={cn('hidden sm:flex w-20 shrink-0', COL_HDR)}>Status</div>
            <div className={cn('hidden lg:flex items-center gap-1.5 w-24 shrink-0', COL_HDR)}>
                <User size={10} /> Joined
            </div>
            <div className={cn('hidden xl:flex items-center gap-1.5 w-24 shrink-0', COL_HDR)}>
                <Clock size={10} /> Last Login
            </div>
            <div className="w-20 shrink-0" />
        </div>
    );
}

const PAGE_SIZES = [10, 25, 50];

interface WalletPaginationProps {
    page: number;
    totalPages: number;
    limit: number;
    total: number;
    onPageChange: (p: number) => void;
    onLimitChange: (v: number) => void;
}

function WalletPagination({ page, totalPages, limit, total, onPageChange, onLimitChange }: WalletPaginationProps) {
    const [goToInput, setGoToInput] = useState('');

    const handleGoTo = () => {
        const n = parseInt(goToInput, 10);
        if (!isNaN(n) && n >= 1 && n <= totalPages) {
            onPageChange(n);
            setGoToInput('');
        }
    };

    const start = (page - 1) * limit + 1;
    const end = Math.min(page * limit, total);

    return (
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pt-3 border-t border-border/30">
            <div className="flex items-center gap-3">
                <p className="text-xs text-muted-foreground whitespace-nowrap">
                    {start}-{end} of {total}
                </p>
                <div className="flex items-center gap-1.5">
                    <label className="text-xs text-muted-foreground" htmlFor="page-size">Show</label>
                    <select
                        id="page-size"
                        value={limit}
                        onChange={(e) => onLimitChange(Number(e.target.value))}
                        className="h-7 rounded-md border border-border/50 bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                        {PAGE_SIZES.map((s) => (
                            <option key={s} value={s}>{s}</option>
                        ))}
                    </select>
                </div>
            </div>
            <div className="flex items-center gap-2">
                {totalPages > 2 && (
                    <div className="flex items-center gap-1.5">
                        <label className="text-xs text-muted-foreground whitespace-nowrap" htmlFor="go-to-page">Go to</label>
                        <input
                            id="go-to-page"
                            type="number"
                            min={1}
                            max={totalPages}
                            value={goToInput}
                            onChange={(e) => setGoToInput(e.target.value)}
                            onKeyDown={(e) => { if (e.key === 'Enter') { handleGoTo(); } }}
                            placeholder={String(page)}
                            className="h-7 w-14 rounded-md border border-border/50 bg-background px-2 text-xs text-center focus:outline-none focus:ring-1 focus:ring-ring [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
                        />
                    </div>
                )}
                <div className="flex items-center gap-1">
                    <Button
                        variant="outline"
                        size="icon"
                        className="h-7 w-7"
                        onClick={() => onPageChange(Math.max(1, page - 1))}
                        disabled={page === 1}
                    >
                        <ChevronLeft size={14} />
                    </Button>
                    <span className="text-xs text-muted-foreground px-1.5 whitespace-nowrap">
                        {page} / {totalPages}
                    </span>
                    <Button
                        variant="outline"
                        size="icon"
                        className="h-7 w-7"
                        onClick={() => onPageChange(Math.min(totalPages, page + 1))}
                        disabled={page === totalPages}
                    >
                        <ChevronRight size={14} />
                    </Button>
                </div>
            </div>
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
        limit,
        setLimit,
        total,
        disableModalWallet: _disableModalWallet,
        setDisableModalWallet: _setDisableModalWallet,
        reenableModalWallet,
        setReenableModalWallet,
        editMetadataWallet,
        setEditMetadataWallet,
        isActionLoading,
        loadData: _loadData,
        handleDisableWallet: _handleDisableWallet,
        handleReenableWallet,
        handleMetadataUpdateSuccess,
        router
    } = useWalletListing({ initialData });

    return (
        <div className={cn("p-4 sm:p-6 lg:p-8 space-y-4", className)}>
            <PageHeader total={total} isLoading={isLoading} />
            <WalletFilterBar
                filters={filters}
                onFilterChange={setFilters}
            />

            {/* Mobile cards */}
            <div className="md:hidden">
                {isLoading ? (
                    Array.from({ length: Math.min(limit, 6) }, (_, i) => `m-skeleton-${i}`).map((key) => (
                        <Skeleton key={key} className="h-36 w-full rounded-2xl mb-3" />
                    ))
                ) : wallets.length === 0 ? (
                    <WalletEmptyState onClearFilters={() => setFilters({ search: '', platform: 'all', status: 'all', sortBy: 'created_at', sortOrder: 'desc' })} />
                ) : (
                    <div className="grid grid-cols-1 gap-3">
                        {wallets.map((wallet) => (
                            <WalletMobileCard
                                key={wallet.walletAddress}
                                wallet={wallet}
                                onView={() => router.push(`/wallet-management/${encodeURIComponent(wallet.walletAddress)}`)}
                                onEnable={() => setReenableModalWallet(wallet)}
                            />
                        ))}
                    </div>
                )}
            </div>

            {/* Desktop / tablet table */}
            <div className="hidden md:block rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
                <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
                <div className="flex items-center justify-between px-4 py-3 border-b border-border/20">
                    <h2 className="text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em]">Wallets</h2>
                </div>
                <ListHeader />
                {isLoading ? (
                    Array.from({ length: limit }, (_, i) => `skeleton-${i}`).map((key) => (
                        <Skeleton key={key} className="h-14 w-full rounded-none border-b border-border/40 last:border-0" />
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

            {!isLoading && total > 0 && (
                <WalletPagination
                    page={page}
                    totalPages={totalPages}
                    limit={limit}
                    total={total}
                    onPageChange={setPage}
                    onLimitChange={(v) => { setLimit(v); setPage(1); }}
                />
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
