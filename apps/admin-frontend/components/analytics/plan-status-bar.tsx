'use client';

import { usePlanAccess } from '@/hooks/use-plan-access';
import { useUpgradeOptions } from '@/hooks/use-upgrade-options';
import type { PlanAccessData } from '@/shared/types/payment';
import { cn } from '@/lib/utils';
import { FREE_PLAN_NAME, FREE_PLAN_RANKING_OFFSET } from '@/shared/config/constants';
import { Crown, Lock, Rocket, Shield, Sparkles, Star, Zap } from 'lucide-react';
import Link from 'next/link';

interface PlanStatusBarProps {
    className?: string;
    planAccess?: PlanAccessData | null;
}

interface TierConfig {
    gradient: string;
    bgGradient: string;
    borderColor: string;
    icon: React.ElementType;
}

const TIER_STYLES: Record<number, TierConfig> = {
    0: {
        gradient: 'from-slate-400 via-gray-500 to-slate-600',
        bgGradient: 'from-slate-500/10 via-gray-500/10 to-slate-600/10',
        borderColor: 'border-slate-500/30',
        icon: Shield,
    },
    1: {
        gradient: 'from-blue-500 via-cyan-500 to-teal-500',
        bgGradient: 'from-blue-500/10 via-cyan-500/10 to-teal-500/10',
        borderColor: 'border-blue-500/30',
        icon: Star,
    },
    2: {
        gradient: 'from-purple-500 via-pink-500 to-rose-500',
        bgGradient: 'from-purple-500/10 via-pink-500/10 to-rose-500/10',
        borderColor: 'border-purple-500/30',
        icon: Zap,
    },
    3: {
        gradient: 'from-amber-400 via-yellow-500 to-orange-500',
        bgGradient: 'from-amber-500/10 via-yellow-500/10 to-orange-500/10',
        borderColor: 'border-amber-500/30',
        icon: Crown,
    },
};

const MAX_TIER = Math.max(...Object.keys(TIER_STYLES).map(Number));

function getTierConfig(tierLevel: number): TierConfig {
    const effectiveTier = Math.min(tierLevel, MAX_TIER);
    return TIER_STYLES[effectiveTier] ?? (TIER_STYLES[0] as TierConfig);
}

function getLockedRanksText(rankingOffset: number): string | null {
    if (rankingOffset <= 0) { return null; }
    if (rankingOffset === 1) { return 'Unlock Rank 1'; }
    return `Unlock ranks 1-${rankingOffset}`;
}

function getRankRangeText(rankingOffset: number): string {
    if (rankingOffset === 0) { return 'All ranks (1+)'; }
    return `Ranks ${rankingOffset + 1}+`;
}

function getLockedRankLabel(rankingOffset: number): string {
    if (rankingOffset === 1) { return 'Rank 1 locked'; }
    return `Ranks 1-${rankingOffset} locked`;
}

function LoadingBar({ className }: { className?: string }) {
    return (
        <div className={cn('animate-pulse rounded-2xl bg-gray-100 dark:bg-card/50 p-4', className)}>
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-xl bg-slate-700" />
                    <div className="space-y-2">
                        <div className="h-4 w-32 rounded bg-slate-700" />
                        <div className="h-3 w-48 rounded bg-slate-700" />
                    </div>
                </div>
                <div className="h-10 w-28 rounded-xl bg-slate-700" />
            </div>
        </div>
    );
}

function LockedRankInfo({ rankingOffset }: { rankingOffset: number }) {
    if (rankingOffset <= 0) { return null; }
    return (
        <>
            <span className="text-muted-foreground">•</span>
            <div className="flex items-center gap-1 text-sm text-slate-400">
                <Lock className="h-3 w-3" />
                <span>{getLockedRankLabel(rankingOffset)}</span>
            </div>
        </>
    );
}

function getPlanData(planAccess: PlanAccessData | null | undefined) {
    return {
        rankingOffset: planAccess?.ranking_offset ?? FREE_PLAN_RANKING_OFFSET,
        planName: planAccess?.plan_name,
        canUpgrade: planAccess?.can_upgrade ?? true,
        tierLevel: planAccess?.tier_level ?? 0,
    };
}

function UpgradeCta({ canUpgrade, lockedRanksText, upgradeButtonText }: {
    canUpgrade: boolean;
    lockedRanksText: string | null;
    upgradeButtonText: string | null;
}) {
    if (!canUpgrade || lockedRanksText === null) { return null; }
    return (
        <Link
            href="/plans"
            className={cn(
                'group inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl px-5 py-2.5 text-sm font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:-translate-y-0.5',
                'bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500'
            )}
        >
            <Rocket className="h-4 w-4 transition-transform group-hover:-rotate-12" />
            {upgradeButtonText}
        </Link>
    );
}

function PlanInfo({ planName, tierLevel, tierConfig, rankRangeText, rankingOffset }: {
    planName: string | undefined;
    tierLevel: number;
    tierConfig: TierConfig;
    rankRangeText: string;
    rankingOffset: number;
}) {
    return (
        <div className="flex items-center gap-4">
            <div className={cn(
                'flex h-12 w-12 items-center justify-center rounded-xl shadow-lg',
                `bg-gradient-to-br ${tierConfig.gradient}`
            )}>
                <tierConfig.icon className="h-6 w-6 text-white" />
            </div>
            <div>
                <div className="flex items-center gap-2">
                    <h3 className="text-base font-bold text-white">
                        {planName ?? FREE_PLAN_NAME}
                    </h3>
                    {tierLevel > 0 && (
                        <span className={cn(
                            'rounded-full px-2 py-0.5 text-xs font-semibold',
                            `bg-gradient-to-r ${tierConfig.gradient} text-white`
                        )}>
                            Tier {tierLevel}
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-2 mt-1">
                    <Sparkles className="h-3.5 w-3.5 text-slate-400" />
                    <span className="text-sm text-slate-300">
                        Viewing: <span className="font-semibold text-white">{rankRangeText}</span>
                    </span>
                    <LockedRankInfo rankingOffset={rankingOffset} />
                </div>
            </div>
        </div>
    );
}

export function PlanStatusBar({ className, planAccess: propPlanAccess }: PlanStatusBarProps): React.ReactElement {
    const { planAccess: hookPlanAccess, loading } = usePlanAccess();
    const { nextPlan } = useUpgradeOptions();

    const planAccess = propPlanAccess !== undefined ? propPlanAccess : hookPlanAccess;

    if (loading) { return <LoadingBar className={className} />; }

    const { rankingOffset, planName, canUpgrade, tierLevel } = getPlanData(planAccess);
    const tierConfig = getTierConfig(tierLevel);
    const rankRangeText = getRankRangeText(rankingOffset);
    const isTopRanks = rankingOffset === 0;
    const lockedRanksText = getLockedRanksText(rankingOffset);
    const upgradeButtonText = nextPlan !== null
        ? `${lockedRanksText ?? ''} with ${nextPlan.name}`
        : lockedRanksText;

    return (
        <div className={cn(
            'relative overflow-hidden rounded-2xl border',
            `bg-gradient-to-r ${tierConfig.bgGradient}`,
            tierConfig.borderColor,
            className
        )}>
            <div className={cn('absolute inset-0 opacity-20 bg-gradient-to-r', tierConfig.gradient)} />

            <div className="relative flex flex-col gap-4 p-4 sm:flex-row sm:items-center sm:justify-between">
                <PlanInfo
                    planName={planName}
                    tierLevel={tierLevel}
                    tierConfig={tierConfig}
                    rankRangeText={rankRangeText}
                    rankingOffset={rankingOffset}
                />
                <UpgradeCta
                    canUpgrade={canUpgrade}
                    lockedRanksText={lockedRanksText}
                    upgradeButtonText={upgradeButtonText}
                />
                {!canUpgrade && isTopRanks && (
                    <div className="flex items-center gap-2 rounded-xl bg-emerald-500/20 px-4 py-2 text-sm font-semibold text-emerald-400">
                        <Crown className="h-4 w-4" />
                        Full Access
                    </div>
                )}
            </div>
        </div>
    );
}

export default PlanStatusBar;
