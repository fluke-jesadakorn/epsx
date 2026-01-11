'use client';

import { Crown, Lock, Rocket, Sparkles } from 'lucide-react';
import Link from 'next/link';

interface RankingUpgradeOverlayProps {
    /** Current rank being viewed */
    rank: number;
    /** User's ranking limit (-1 = unlimited, 0 = none, positive = limit) */
    rankingsLimit: number;
    /** Total available rankings */
    totalRankings?: number;
    /** Current plan name for display */
    planName?: string | null;
}

/**
 * Overlay component shown on locked ranking cards
 * Displays upgrade prompt when user exceeds their plan's ranking limit
 */
export function RankingUpgradeOverlay({
    rank,
    rankingsLimit,
    totalRankings = 100,
    planName,
}: RankingUpgradeOverlayProps): React.ReactElement | null {
    // Don't show overlay for unlimited access or if within limit
    if (rankingsLimit === -1 || rank <= rankingsLimit) {
        return null;
    }

    return (
        <div className="absolute inset-0 z-10 flex items-center justify-center rounded-2xl backdrop-blur-[6px] bg-gradient-to-br from-slate-900/80 via-slate-800/70 to-slate-900/80">
            {/* Decorative gradient border effect */}
            <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-purple-500/20 via-pink-500/20 to-orange-500/20 opacity-50" />

            <div className="relative z-10 p-6 text-center">
                {/* Lock icon with glow */}
                <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg shadow-purple-500/30">
                    <Lock className="h-7 w-7 text-white" />
                </div>

                {/* Premium badge */}
                <div className="mb-3 inline-flex items-center gap-1.5 rounded-full bg-gradient-to-r from-amber-500/20 to-orange-500/20 px-3 py-1 text-xs font-semibold text-amber-400">
                    <Crown className="h-3 w-3" />
                    Premium Content
                </div>

                {/* Message */}
                <p className="mb-4 text-sm text-slate-300">
                    Rank #{rank} requires a higher plan
                </p>

                {/* Upgrade CTA */}
                <Link
                    href="/plans"
                    className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 px-5 py-2.5 text-sm font-semibold text-white shadow-lg shadow-purple-500/25 transition-all duration-300 hover:shadow-xl hover:shadow-purple-500/40 hover:-translate-y-0.5"
                >
                    <Rocket className="h-4 w-4" />
                    Upgrade Now
                </Link>
            </div>
        </div>
    );
}

interface UpgradeBannerInlineProps {
    /** User's ranking limit */
    rankingsLimit: number;
    /** Total rankings available */
    totalRankings: number;
    /** Current plan name */
    planName?: string | null;
    /** Custom class name */
    className?: string;
}

/**
 * Inline banner shown at top of rankings page indicating upgrade availability
 */
export function UpgradeBannerInline({
    rankingsLimit,
    totalRankings,
    planName,
    className = '',
}: UpgradeBannerInlineProps): React.ReactElement | null {
    // Don't show for unlimited access
    if (rankingsLimit === -1) {
        return null;
    }

    const moreRankings = totalRankings - rankingsLimit;

    return (
        <div className={`relative overflow-hidden rounded-2xl bg-gradient-to-r from-purple-500/10 via-pink-500/10 to-orange-500/10 border border-purple-500/20 ${className}`}>
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
                            Viewing {rankingsLimit} of {totalRankings} rankings
                        </h3>
                        <p className="text-sm text-slate-600 dark:text-slate-400">
                            {planName ? `Your ${planName} includes ${rankingsLimit} rankings.` : `Free tier includes ${rankingsLimit} rankings.`}{' '}
                            Upgrade to unlock {moreRankings > 0 ? `${moreRankings} more` : 'unlimited'} rankings!
                        </p>
                    </div>
                </div>

                {/* CTA Button */}
                <Link
                    href="/plans"
                    className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-purple-500/25 transition-all duration-300 hover:shadow-xl hover:shadow-purple-500/40 hover:-translate-y-0.5"
                >
                    <Crown className="h-4 w-4" />
                    Upgrade Plan
                </Link>
            </div>
        </div>
    );
}

export default RankingUpgradeOverlay;
