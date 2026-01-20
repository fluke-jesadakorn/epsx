'use client';

import { usePlanAccess } from '@/hooks/usePlanAccess';
import { cn } from '@/lib/utils';
import { StockDataCard } from '@/shared/components';
import type { AnalyticsPagination, SymbolCardData } from '@/shared/types/analytics';
import { Crown, Lock, Rocket, Sparkles } from 'lucide-react';
import Link from 'next/link';
import { useMemo } from 'react';

interface AnalyticsCardGridProps {
    rankings: SymbolCardData[];
    pagination?: AnalyticsPagination;
    className?: string;
}

interface LockedCardOverlayProps {
    rank: number;
}

function LockedCardOverlay({ rank }: LockedCardOverlayProps): React.ReactElement {
    return (
        <div className="absolute inset-0 z-20 flex items-center justify-center rounded-2xl overflow-hidden animate-in fade-in duration-300">
            {/* Blurred background */}
            <div className="absolute inset-0 backdrop-blur-md bg-slate-900/70" />

            {/* Gradient border effect */}
            <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-purple-500/20 via-pink-500/10 to-orange-500/20" />

            {/* Content */}
            <div className="relative z-10 flex flex-col items-center p-4 text-center">
                {/* Lock icon with glow */}
                <div className="mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-gradient-to-br from-purple-500 via-pink-500 to-orange-500 shadow-lg shadow-purple-500/30 animate-in zoom-in-75 duration-300">
                    <Lock className="h-7 w-7 text-white" />
                </div>

                {/* Rank badge */}
                <div className="mb-2 flex items-center gap-1.5 rounded-full bg-gradient-to-r from-amber-500/20 to-orange-500/20 px-3 py-1 text-xs font-bold text-amber-400 animate-in slide-in-from-bottom-2 duration-300 delay-100">
                    <Crown className="h-3 w-3" />
                    Rank #{rank}
                </div>

                {/* Message */}
                <p className="mb-3 text-sm text-slate-300 animate-in slide-in-from-bottom-2 duration-300 delay-150">
                    Premium Content
                </p>

                {/* Upgrade CTA */}
                <div className="animate-in slide-in-from-bottom-2 duration-300 delay-200">
                    <Link
                        href="/plans"
                        className="inline-flex items-center gap-1.5 rounded-xl bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 px-4 py-2 text-xs font-semibold text-white shadow-lg shadow-purple-500/25 transition-all duration-300 hover:shadow-xl hover:shadow-purple-500/40 hover:-translate-y-0.5"
                    >
                        <Rocket className="h-3.5 w-3.5" />
                        Unlock
                    </Link>
                </div>
            </div>
        </div>
    );
}

function LockedStockCard({ cardData }: { cardData: SymbolCardData }): React.ReactElement {
    const latestQuarter = cardData.quarterly_performance?.[0];
    const isPremium = cardData.rank <= 5;

    return (
        <div className="relative">
            {/* Underlying card (visible but blurred) */}
            <div className="opacity-30 blur-[2px] pointer-events-none">
                <StockDataCard
                    symbol={cardData.symbol}
                    rank={cardData.rank}
                    epsGrowth={latestQuarter?.eps_growth || 0}
                    price={latestQuarter?.price || 0}
                    currency={cardData.currency}
                    daysUntilNextAction={cardData.next_quarter_estimate?.days_until_announcement ?? 0}
                    companyName={cardData.company_name || cardData.name}
                    variant={isPremium ? 'premium' : 'standard'}
                />
            </div>
            <LockedCardOverlay rank={cardData.rank} />
        </div>
    );
}

function UnlockedStockCard({ cardData, delay = 0 }: { cardData: SymbolCardData; delay?: number }): React.ReactElement {
    const latestQuarter = cardData.quarterly_performance?.[0];
    const isPremium = cardData.rank <= 5;

    return (
        <div
            className="animate-in fade-in slide-in-from-bottom-4 duration-300"
            style={{ animationDelay: `${delay}ms` }}
        >
            <StockDataCard
                symbol={cardData.symbol}
                rank={cardData.rank}
                epsGrowth={latestQuarter?.eps_growth || 0}
                price={latestQuarter?.price || 0}
                currency={cardData.currency}
                daysUntilNextAction={cardData.next_quarter_estimate?.days_until_announcement ?? 0}
                companyName={cardData.company_name || cardData.name}
                variant={isPremium ? 'premium' : 'standard'}
            />
        </div>
    );
}

export function AnalyticsCardGrid({ rankings, className }: AnalyticsCardGridProps): React.ReactElement {
    const { planAccess, loading: planLoading } = usePlanAccess();

    // Get ranking offset from plan access (default to 100 for free tier)
    const rankingOffset = planAccess?.ranking_offset ?? 100;

    // Sort all cards by rank and determine locked/unlocked status
    const sortedCards = useMemo(() => {
        return [...rankings].sort((a, b) => a.rank - b.rank).map(card => ({
            ...card,
            isLocked: card.rank <= rankingOffset
        }));
    }, [rankings, rankingOffset]);

    // Count locked cards for the upgrade banner
    const lockedCount = useMemo(() => {
        return sortedCards.filter(card => card.isLocked).length;
    }, [sortedCards]);

    // Show loading state while plan access is loading
    if (planLoading) {
        return (
            <div className={cn('space-y-6', className)}>
                <div className="grid grid-cols-1 justify-items-center gap-4 px-2 sm:grid-cols-2 sm:gap-6 sm:px-0 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-5">
                    {Array.from({ length: 10 }).map((_, i) => (
                        <div
                            key={i}
                            className="w-full max-w-[400px] animate-pulse rounded-2xl bg-slate-800/50 p-6"
                        >
                            <div className="mb-4 h-6 w-24 rounded bg-slate-700" />
                            <div className="mb-2 h-12 w-32 rounded bg-slate-700 mx-auto" />
                            <div className="space-y-2">
                                <div className="h-4 w-full rounded bg-slate-700" />
                                <div className="h-4 w-3/4 rounded bg-slate-700" />
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        );
    }

    // If no rankings data
    if (!rankings || rankings.length === 0) {
        return (
            <div className="py-12 text-center">
                <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-800/50">
                    <Sparkles className="h-8 w-8 text-slate-400" />
                </div>
                <p className="text-slate-400">No rankings data available</p>
            </div>
        );
    }

    return (
        <div className={cn('space-y-6', className)}>
            {/* Upgrade banner if there are locked cards */}
            {lockedCount > 0 && (
                <div className="flex flex-wrap items-center justify-center gap-4 rounded-xl bg-gradient-to-r from-purple-500/10 via-pink-500/10 to-orange-500/10 border border-purple-500/20 p-4">
                    <div className="flex items-center gap-2">
                        <Crown className="h-5 w-5 text-purple-400" />
                        <span className="text-sm text-slate-300">
                            {lockedCount} premium {lockedCount === 1 ? 'ranking' : 'rankings'} locked (Ranks 1-{rankingOffset})
                        </span>
                    </div>
                    <Link
                        href="/plans"
                        className="inline-flex items-center gap-1.5 rounded-lg bg-gradient-to-r from-purple-500 to-pink-500 px-4 py-1.5 text-xs font-semibold text-white transition-all hover:shadow-lg hover:shadow-purple-500/25"
                    >
                        <Rocket className="h-3.5 w-3.5" />
                        Unlock Premium
                    </Link>
                </div>
            )}

            {/* Unified Grid: All cards displayed together */}
            <div className="grid grid-cols-1 justify-items-center gap-4 px-2 sm:grid-cols-2 sm:gap-6 sm:px-0 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-5">
                {sortedCards.map((cardData, index) => (
                    <div
                        key={cardData.symbol}
                        className="w-full max-w-[400px]"
                    >
                        {cardData.isLocked ? (
                            <div className="animate-in fade-in zoom-in-95 duration-300" style={{ animationDelay: `${Math.min(index * 50, 300)}ms` }}>
                                <LockedStockCard cardData={cardData} />
                            </div>
                        ) : (
                            <UnlockedStockCard cardData={cardData} delay={Math.min(index * 50, 500)} />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}

export default AnalyticsCardGrid;
