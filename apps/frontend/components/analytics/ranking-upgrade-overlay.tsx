'use client';

import { cn } from '@/lib/utils';
import { Crown, Lock, Rocket, Sparkles, TrendingUp } from 'lucide-react';
import Link from 'next/link';

interface RankingUpgradeOverlayProps {
    /** Current rank being viewed */
    rank: number;
    /** user's ranking offset (0 = top ranks, 100 = ranks 101+) */
    rankingOffset: number;
    /** Total available rankings */
    totalRankings?: number;
    /** Current plan name for display */
    planName?: string | null;
    /** Size variant */
    variant?: 'compact' | 'full';
}

/**
 * Redesigned overlay component for locked ranking cards
 * Shows rank number prominently with glass-morphism effect
 */
export function RankingUpgradeOverlay({
    rank,
    rankingOffset,
    totalRankings: _totalRankings = 100,
    planName: _planName,
    variant = 'full',
}: RankingUpgradeOverlayProps): React.ReactElement | null {
    // Don't show overlay for unlocked ranks (offset 0 = full access)
    if (rankingOffset === 0 || rank > rankingOffset) {
        return null;
    }

    const isTopTier = rank <= 3;
    const isCompact = variant === 'compact';

    return (
        <div className={cn(
            "absolute inset-0 z-20 flex items-center justify-center rounded-2xl overflow-hidden",
            "group transition-all duration-300"
        )}>
            {/* Glass-morphism background with animated gradient */}
            <div className="absolute inset-0 backdrop-blur-md bg-white dark:bg-slate-900/60" />
            
            {/* Animated gradient border */}
            <div className="absolute inset-0 rounded-2xl">
                <div className="absolute inset-0 bg-gradient-to-br from-purple-500/30 via-transparent to-pink-500/30 animate-pulse" />
                <div className="absolute inset-[1px] rounded-2xl bg-white dark:bg-slate-900/80" />
            </div>

            {/* Decorative particles */}
            <div className="absolute inset-0 overflow-hidden">
                <div className="absolute top-1/4 left-1/4 h-2 w-2 rounded-full bg-purple-400/40 animate-ping" style={{ animationDuration: '2s' }} />
                <div className="absolute top-3/4 right-1/4 h-1.5 w-1.5 rounded-full bg-pink-400/40 animate-ping" style={{ animationDuration: '2.5s', animationDelay: '0.5s' }} />
                <div className="absolute top-1/2 right-1/3 h-1 w-1 rounded-full bg-orange-400/40 animate-ping" style={{ animationDuration: '3s', animationDelay: '1s' }} />
            </div>

            {/* Content */}
            <div className={cn(
                "relative z-10 flex flex-col items-center text-center",
                isCompact ? "p-3" : "p-5"
            )}>
                {/* Rank number - prominently displayed */}
                <div className={cn(
                    "relative mb-3",
                    isCompact ? "mb-2" : "mb-3"
                )}>
                    {/* Glow effect behind rank */}
                    <div className={cn(
                        "absolute inset-0 rounded-2xl blur-xl",
                        isTopTier 
                            ? "bg-gradient-to-br from-amber-400/50 to-orange-500/50" 
                            : "bg-gradient-to-br from-purple-500/40 to-pink-500/40"
                    )} />
                    
                    <div className={cn(
                        "relative flex items-center justify-center rounded-2xl font-black",
                        isTopTier 
                            ? "bg-gradient-to-br from-amber-400 via-yellow-500 to-orange-500" 
                            : "bg-gradient-to-br from-purple-500 via-pink-500 to-rose-500",
                        "shadow-lg text-white",
                        isCompact ? "h-12 w-16 text-xl" : "h-16 w-20 text-2xl"
                    )}>
                        #{rank}
                    </div>
                </div>

                {/* Lock indicator with premium badge */}
                <div className={cn(
                    "flex items-center gap-1.5 rounded-full px-3 py-1 mb-2",
                    "bg-gradient-to-r from-gray-100/80 dark:from-slate-800/80 to-slate-700/80 border border-gray-200 dark:border-white/10"
                )}>
                    <Lock className={cn("text-slate-400", isCompact ? "h-3 w-3" : "h-3.5 w-3.5")} />
                    <span className={cn(
                        "font-semibold text-slate-300",
                        isCompact ? "text-[10px]" : "text-xs"
                    )}>
                        Premium Rank
                    </span>
                    {isTopTier && (
                        <Crown className={cn("text-amber-400", isCompact ? "h-3 w-3" : "h-3.5 w-3.5")} />
                    )}
                </div>

                {/* Performance teaser */}
                {!isCompact && (
                    <div className="flex items-center gap-1 mb-3 text-emerald-400">
                        <TrendingUp className="h-3.5 w-3.5" />
                        <span className="text-xs font-semibold">High Growth Potential</span>
                    </div>
                )}

                {/* Upgrade CTA */}
                <Link
                    href="/plans"
                    className={cn(
                        "inline-flex items-center gap-1.5 font-semibold text-white",
                        "bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500",
                        "shadow-lg shadow-purple-500/25",
                        "transition-all duration-300",
                        "hover:shadow-xl hover:shadow-purple-500/40 hover:-translate-y-0.5",
                        "group-hover:scale-105",
                        isCompact 
                            ? "rounded-lg px-3 py-1.5 text-[10px]" 
                            : "rounded-xl px-4 py-2 text-xs"
                    )}
                >
                    <Rocket className={cn(isCompact ? "h-3 w-3" : "h-3.5 w-3.5")} />
                    Unlock
                </Link>
            </div>
        </div>
    );
}

interface UpgradeBannerInlineProps {
    /** user's ranking offset (0 = top ranks, higher = locked top ranks) */
    rankingOffset: number;
    /** Total rankings available */
    totalRankings?: number;
    /** Current plan name */
    planName?: string | null;
    /** Custom class name */
    className?: string;
}

/**
 * Inline banner shown at top of rankings page indicating upgrade availability
 * Updated to use offset-based logic
 */
export function UpgradeBannerInline({
    rankingOffset,
    totalRankings = 100,
    planName,
    className = '',
}: UpgradeBannerInlineProps): React.ReactElement | null {
    // Don't show for full access (offset 0)
    if (rankingOffset === 0) {
        return null;
    }

    const visibleRankStart = rankingOffset + 1;
    const lockedRanks = rankingOffset;

    return (
        <div className={cn(
            "relative overflow-hidden rounded-2xl border border-purple-500/20",
            "bg-gradient-to-r from-purple-500/10 via-pink-500/10 to-orange-500/10",
            className
        )}>
            {/* Animated gradient background */}
            <div className="absolute inset-0 bg-gradient-to-r from-purple-500/5 via-pink-500/5 to-orange-500/5 animate-pulse" />

            <div className="relative flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-4">
                    {/* Icon */}
                    <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg shadow-purple-500/25">
                        <Sparkles className="h-6 w-6 text-white" />
                    </div>

                    {/* Text content */}
                    <div>
                        <h3 className="text-base font-bold text-slate-800 dark:text-white">
                            Viewing ranks {visibleRankStart}+ of {totalRankings}
                        </h3>
                        <p className="text-sm text-slate-600 dark:text-slate-400">
                            {planName 
                                ? `Your ${planName} shows ranks ${visibleRankStart}+.` 
                                : `Free tier shows ranks ${visibleRankStart}+.`}{' '}
                            Upgrade to unlock the top {lockedRanks} premium rankings!
                        </p>
                    </div>
                </div>

                {/* CTA Button */}
                <Link
                    href="/plans"
                    className={cn(
                        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl",
                        "bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500",
                        "px-6 py-3 text-sm font-semibold text-white",
                        "shadow-lg shadow-purple-500/25",
                        "transition-all duration-300",
                        "hover:shadow-xl hover:shadow-purple-500/40 hover:-translate-y-0.5"
                    )}
                >
                    <Crown className="h-4 w-4" />
                    Unlock Ranks 1-{lockedRanks}
                </Link>
            </div>
        </div>
    );
}

export default RankingUpgradeOverlay;
