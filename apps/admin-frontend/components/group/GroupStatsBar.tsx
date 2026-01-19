/**
 * Group Stats Bar Component
 * Dashboard statistics for group overview - matches WalletStatsBar design
 */
'use client';

import { Clock, Shield, Trophy, Users } from 'lucide-react';
import React from 'react';

import type { GroupStats } from './types';

import { cn } from '@/lib/utils';

interface GroupStatsBarProps {
    stats: GroupStats;
    isLoading?: boolean;
    className?: string;
}

interface StatCardProps {
    label: string;
    value: number | string;
    subLabel?: string;
    icon: React.ReactNode;
    gradient: string;
}

function StatCard({ label, value, subLabel, icon, gradient }: StatCardProps) {
    return (
        <div className={cn(
            'relative overflow-hidden rounded-2xl p-0.5 h-full',
            gradient
        )}>
            <div className="relative bg-white dark:bg-gray-900 rounded-2xl p-5 h-full flex flex-col">
                <div className="flex items-center justify-between mb-3">
                    <div className="text-gray-500 dark:text-gray-400">
                        {icon}
                    </div>
                </div>
                <div className="flex-1 flex flex-col">
                    <p className="text-3xl font-bold text-gray-900 dark:text-white">
                        {typeof value === 'number' ? value.toLocaleString() : value}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                        {label}
                    </p>
                    {subLabel && (
                        <p className="text-xs text-gray-400 dark:text-gray-500 mt-0.5">
                            {subLabel}
                        </p>
                    )}
                </div>
            </div>
        </div>
    );
}

function StatsBarSkeleton() {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="rounded-2xl bg-gray-100 dark:bg-gray-800 p-5 animate-pulse">
                    <div className="flex items-center justify-between mb-3">
                        <div className="h-6 w-6 rounded bg-gray-200 dark:bg-gray-700" />
                    </div>
                    <div className="h-8 w-20 rounded bg-gray-200 dark:bg-gray-700 mb-2" />
                    <div className="h-4 w-24 rounded bg-gray-200 dark:bg-gray-700" />
                </div>
            ))}
        </div>
    );
}

/**
 * GroupStatsBar component displays analytics statistics in a dashboard format
 */
export function GroupStatsBar({
    stats,
    isLoading = false,
    className,
}: GroupStatsBarProps) {
    if (isLoading) {
        return <StatsBarSkeleton />;
    }

    return (
        <div className={cn('space-y-4', className)}>
            {/* Main Stats */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <StatCard
                    label="Total Groups"
                    value={stats.totalGroups}
                    icon={<Shield className="h-6 w-6 text-blue-500" />}
                    gradient="bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20"
                />
                <StatCard
                    label="Active Memberships"
                    value={stats.activeMemberships}
                    icon={<Users className="h-6 w-6 text-green-500" />}
                    gradient="bg-gradient-to-r from-green-400/20 to-emerald-400/20"
                />
                <StatCard
                    label="Expiring Soon"
                    value={stats.expiringSoon}
                    subLabel="Within 7 days"
                    icon={<Clock className="h-6 w-6 text-amber-500" />}
                    gradient="bg-gradient-to-r from-amber-400/20 to-orange-400/20"
                />
                <StatCard
                    label="Largest Group"
                    value={stats.largestGroup.memberCount}
                    subLabel={stats.largestGroup.name || 'N/A'}
                    icon={<Trophy className="h-6 w-6 text-purple-500" />}
                    gradient="bg-gradient-to-r from-purple-400/20 to-pink-400/20"
                />
            </div>
        </div>
    );
}
