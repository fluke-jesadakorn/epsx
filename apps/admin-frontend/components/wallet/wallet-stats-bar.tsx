/**
 * Wallet Stats Bar Component
 * Dashboard statistics for wallet overview
 */
'use client';

import { AlertTriangle, Package, TrendingDown, TrendingUp, Users, Wallet } from 'lucide-react';
import React from 'react';

import type { Platform, WalletStats } from './types';

import { cn } from '@/lib/utils';

interface WalletStatsBarProps {
    stats: WalletStats;
    isLoading?: boolean;
    className?: string;
}

interface StatCardProps {
    label: string;
    value: number;
    change?: number;
    changeLabel?: string;
    icon: React.ReactNode;
    gradient: string;
}

function StatCard({ label, value, change, changeLabel = '7d', icon, gradient: _gradient }: StatCardProps) {
    const hasPositiveChange = change !== undefined && change > 0;
    const hasNegativeChange = change !== undefined && change < 0;

    return (
        <div className="rounded-xl bg-card border border-border/20 p-5 shadow-sm">
            <div className="flex items-center justify-between">
                <div className="text-muted-foreground">
                    {icon}
                </div>
                {change !== undefined && (
                    <div className={cn(
                        'flex items-center gap-1 text-xs font-medium px-2 py-1 rounded-full',
                        hasPositiveChange && 'bg-[#31d0aa]/10 text-[#31d0aa]',
                        hasNegativeChange && 'bg-destructive/10 text-destructive',
                        !hasPositiveChange && !hasNegativeChange && 'bg-muted text-muted-foreground'
                    )}>
                        {hasPositiveChange && <TrendingUp className="h-3 w-3" />}
                        {hasNegativeChange && <TrendingDown className="h-3 w-3" />}
                        {hasPositiveChange && '+'}
                        {change} ({changeLabel})
                    </div>
                )}
            </div>
            <div className="mt-3">
                <p className="text-3xl font-bold text-foreground">
                    {value.toLocaleString()}
                </p>
                <p className="text-sm text-muted-foreground mt-1">
                    {label}
                </p>
            </div>
        </div>
    );
}

function PlatformDistribution({
    distribution,
    total
}: {
    distribution: Record<Platform, number>;
    total: number;
}) {
    const platforms: { key: Platform; label: string; emoji: string; color: string }[] = [
        { key: 'analytics', label: 'Analytics', emoji: '📊', color: 'bg-[#1fc7d4]' },
        { key: 'pay', label: 'Pay', emoji: '💳', color: 'bg-[#7645d9]' },
        { key: 'token', label: 'Token', emoji: '🪙', color: 'bg-[#ffb237]' },
        { key: 'markets', label: 'Markets', emoji: '📈', color: 'bg-[#31d0aa]' },
    ];

    return (
        <div className="rounded-xl bg-card border border-border/20 p-5 shadow-sm">
            <h4 className="text-sm font-semibold text-foreground/80 mb-4">
                Platform Distribution
            </h4>
            <div className="space-y-3">
                    {platforms.map((platform) => {
                        const count = (distribution[platform.key] as number | undefined) ?? 0;
                        const percentage = total > 0 ? Math.round((count / total) * 100) : 0;

                        return (
                            <div key={platform.key}>
                                <div className="flex items-center justify-between text-sm mb-1">
                                    <span className="flex items-center gap-2 text-muted-foreground">
                                        <span>{platform.emoji}</span>
                                        {platform.label}
                                    </span>
                                    <span className="font-medium text-foreground">
                                        {count.toLocaleString()} ({percentage}%)
                                    </span>
                                </div>
                                <div className="h-2 bg-muted rounded-full overflow-hidden">
                                    <div
                                        className={cn('h-full rounded-full transition-all duration-500', platform.color)}
                                        style={{ width: `${percentage}%` }}
                                    />
                                </div>
                            </div>
                        );
                    })}
            </div>
        </div>
    );
}

function StatsBarSkeleton() {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-5 gap-4">
            {Array.from({ length: 5 }, (_, i) => `skeleton-${i}`).map((key) => (
                <div key={key} className="rounded-2xl bg-muted p-5 animate-pulse">
                    <div className="flex items-center justify-between mb-3">
                        <div className="h-6 w-6 rounded bg-muted/80" />
                        <div className="h-5 w-16 rounded-full bg-muted/80" />
                    </div>
                    <div className="h-8 w-20 rounded bg-muted/80 mb-2" />
                    <div className="h-4 w-24 rounded bg-muted/80" />
                </div>
            ))}
        </div>
    );
}

/**
 *
 * @param root0
 * @param root0.stats
 * @param root0.isLoading
 * @param root0.className
 */
export function WalletStatsBar({
    stats,
    isLoading = false,
    className,
}: WalletStatsBarProps) {
    if (isLoading) {
        return <StatsBarSkeleton />;
    }

    return (
        <div className={cn('space-y-4', className)}>
            {/* Main Stats */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <StatCard
                    label="Total Wallets"
                    value={stats.total}
                    change={stats.changes.total}
                    icon={<Wallet className="h-6 w-6 text-[#1fc7d4]" />}
                    gradient=""
                />
                <StatCard
                    label="Active"
                    value={stats.active}
                    change={stats.changes.active}
                    icon={<Users className="h-6 w-6 text-[#31d0aa]" />}
                    gradient=""
                />
                <StatCard
                    label="Disabled"
                    value={stats.disabled}
                    change={stats.changes.disabled}
                    icon={<AlertTriangle className="h-6 w-6 text-[#ffb237]" />}
                    gradient=""
                />
                <StatCard
                    label="Subscribed"
                    value={stats.subscribed}
                    change={stats.changes.subscribed}
                    icon={<Package className="h-6 w-6 text-[#7645d9]" />}
                    gradient=""
                />
            </div>

            {/* Platform Distribution */}
            <PlatformDistribution distribution={stats.platformDistribution} total={stats.total} />
        </div>
    );
}
