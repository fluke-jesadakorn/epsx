'use client';

import { usePlanAccess } from '@/hooks/use-plan-access';
import { FREE_PLAN_RANKING_OFFSET } from '@/shared/config/constants';
import { Crown, Lock, Rocket } from 'lucide-react';
import Link from 'next/link';
import { UpgradeBannerInline } from './ranking-upgrade-overlay';

interface PlanGatedRankingsProps {
    children: React.ReactNode;
    /** Total available rankings to display limit info */
    totalRankings?: number;
}

/**
 * Client wrapper that adds plan-gated upgrade prompts to rankings
 * Uses usePlanAccess hook to fetch user's plan access
 */
export function PlanGatedRankings({
    children,
    totalRankings = 100
}: PlanGatedRankingsProps): React.ReactElement {
    const { planAccess, loading } = usePlanAccess();

    // ranking_offset: 0 = full access, >0 = number of top ranks locked
    const rankingOffset = planAccess?.ranking_offset ?? FREE_PLAN_RANKING_OFFSET;
    const hasFullAccess = rankingOffset === 0;
    const canUpgrade = planAccess?.can_upgrade ?? true;

    return (
        <div className="space-y-6">
            {/* Show upgrade banner for limited plans */}
            {!loading && !hasFullAccess && canUpgrade && (
                <UpgradeBannerInline
                    rankingOffset={rankingOffset}
                    totalRankings={totalRankings}
                    planName={planAccess?.plan_name}
                    className="mb-6"
                />
            )}

            {/* Rankings content */}
            {children}

            {/* Bottom upgrade prompt for limited users */}
            {!loading && !hasFullAccess && canUpgrade && (
                <div className="mt-8 rounded-2xl bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-6 text-center">
                    <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-purple-500 via-pink-500 to-orange-500 shadow-lg shadow-purple-500/30">
                        <Crown className="h-8 w-8 text-white" />
                    </div>

                    <h3 className="mb-2 text-xl font-bold text-white">
                        Unlock Premium Rankings
                    </h3>
                    <p className="mb-6 text-slate-400">
                        Ranks 1-{rankingOffset} are premium. Upgrade to unlock top performers and advanced analytics.
                    </p>

                    <div className="flex flex-col items-center justify-center gap-3 sm:flex-row">
                        <Link
                            href="/plans"
                            className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 px-8 py-3 text-base font-semibold text-white shadow-lg shadow-purple-500/25 transition-all duration-300 hover:shadow-xl hover:shadow-purple-500/40 hover:-translate-y-0.5"
                        >
                            <Rocket className="h-5 w-5" />
                            View Pricing Plans
                        </Link>

                        <span className="text-sm text-slate-500">
                            Starting at $14.99/month
                        </span>
                    </div>
                </div>
            )}
        </div>
    );
}

/**
 * Locked card overlay for individual cards beyond user's limit
 */
export function LockedCardOverlay(): React.ReactElement {
    return (
        <div className="absolute inset-0 z-10 flex items-center justify-center rounded-2xl backdrop-blur-sm bg-slate-900/60">
            <div className="text-center p-4">
                <div className="mx-auto mb-2 flex h-10 w-10 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-pink-500">
                    <Lock className="h-5 w-5 text-white" />
                </div>
                <p className="text-xs font-medium text-slate-300">Premium</p>
            </div>
        </div>
    );
}

export default PlanGatedRankings;
