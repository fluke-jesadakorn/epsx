/**
 * PolicyStatsBar Component
 * Displays aggregated statistics for all access policies
 */
'use client';

import { Skeleton } from '@/components/ui/skeleton';
import { cn } from '@/lib/utils';
import { Clock, Layers, TrendingUp, Users } from 'lucide-react';
import type { PolicyStats } from './types';

interface PolicyStatsBarProps {
  stats: PolicyStats;
  isLoading?: boolean;
  className?: string;
}

interface StatCardProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  subLabel?: string;
  iconBg: string;
  iconColor: string;
}

function StatCard({ icon, label, value, subLabel, iconBg, iconColor }: StatCardProps) {
  return (
    <div className="relative overflow-hidden rounded-2xl bg-card border border-border p-4 sm:p-5 hover:shadow-lg hover:scale-[1.02] transition-all duration-300">
      {/* Subtle gradient background */}
      <div className="absolute inset-0 bg-gradient-to-br from-transparent via-transparent to-primary/5 pointer-events-none" />

      <div className="relative flex items-start gap-4">
        <div className={cn(
          'flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center rounded-xl',
          iconBg
        )}>
          <div className={iconColor}>{icon}</div>
        </div>

        <div className="flex-1 min-w-0">
          <p className="text-2xl sm:text-3xl font-bold text-foreground truncate">
            {value}
          </p>
          <p className="text-sm text-muted-foreground font-medium">{label}</p>
          {subLabel && (
            <p className="text-xs text-muted-foreground/70 mt-0.5">{subLabel}</p>
          )}
        </div>
      </div>
    </div>
  );
}

function StatCardSkeleton() {
  return (
    <div className="rounded-2xl bg-card border border-border p-4 sm:p-5">
      <div className="flex items-start gap-4">
        <Skeleton className="h-10 w-10 sm:h-12 sm:w-12 rounded-xl" />
        <div className="flex-1 space-y-2">
          <Skeleton className="h-8 w-16" />
          <Skeleton className="h-4 w-24" />
        </div>
      </div>
    </div>
  );
}

export function PolicyStatsBar({ stats, isLoading = false, className }: PolicyStatsBarProps) {
  if (isLoading) {
    return (
      <div className={cn('grid grid-cols-2 lg:grid-cols-4 gap-4', className)}>
        <StatCardSkeleton />
        <StatCardSkeleton />
        <StatCardSkeleton />
        <StatCardSkeleton />
      </div>
    );
  }

  return (
    <div className={cn('grid grid-cols-2 lg:grid-cols-4 gap-4', className)}>
      {/* Total Policies */}
      <StatCard
        icon={<Layers className="h-5 w-5 sm:h-6 sm:w-6" />}
        label="Total Policies"
        value={stats.totalPolicies}
        subLabel={`${stats.activeSubscriptions} plans, ${stats.activeGroups} groups`}
        iconBg="bg-blue-100 dark:bg-blue-900/30"
        iconColor="text-blue-600 dark:text-blue-400"
      />

      {/* Active Members */}
      <StatCard
        icon={<Users className="h-5 w-5 sm:h-6 sm:w-6" />}
        label="Active Members"
        value={stats.totalMembers.toLocaleString()}
        subLabel="Across all policies"
        iconBg="bg-emerald-100 dark:bg-emerald-900/30"
        iconColor="text-emerald-600 dark:text-emerald-400"
      />

      {/* Monthly Revenue */}
      <StatCard
        icon={<TrendingUp className="h-5 w-5 sm:h-6 sm:w-6" />}
        label="Monthly Revenue"
        value={`$${stats.totalMRR.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
        subLabel="From subscriptions"
        iconBg="bg-purple-100 dark:bg-purple-900/30"
        iconColor="text-purple-600 dark:text-purple-400"
      />

      {/* Expiring Soon */}
      <StatCard
        icon={<Clock className="h-5 w-5 sm:h-6 sm:w-6" />}
        label="Expiring Soon"
        value={stats.expiringSoon}
        subLabel="Within 7 days"
        iconBg={stats.expiringSoon > 0
          ? "bg-amber-100 dark:bg-amber-900/30"
          : "bg-gray-100 dark:bg-gray-800/50"
        }
        iconColor={stats.expiringSoon > 0
          ? "text-amber-600 dark:text-amber-400"
          : "text-gray-500 dark:text-gray-400"
        }
      />
    </div>
  );
}

/**
 * Compact version of stats for mobile or sidebar
 */
export function PolicyStatsCompact({ stats, isLoading = false }: PolicyStatsBarProps) {
  if (isLoading) {
    return (
      <div className="flex items-center gap-4 overflow-x-auto pb-2">
        {[1, 2, 3, 4].map(i => (
          <Skeleton key={i} className="h-8 w-24 rounded-full flex-shrink-0" />
        ))}
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 overflow-x-auto pb-2">
      <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 flex-shrink-0">
        <Layers className="h-4 w-4" />
        <span className="text-sm font-semibold">{stats.totalPolicies}</span>
        <span className="text-xs">policies</span>
      </div>

      <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-400 flex-shrink-0">
        <Users className="h-4 w-4" />
        <span className="text-sm font-semibold">{stats.totalMembers}</span>
        <span className="text-xs">members</span>
      </div>

      <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400 flex-shrink-0">
        <TrendingUp className="h-4 w-4" />
        <span className="text-sm font-semibold">${stats.totalMRR.toFixed(0)}</span>
        <span className="text-xs">MRR</span>
      </div>

      {stats.expiringSoon > 0 && (
        <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 flex-shrink-0">
          <Clock className="h-4 w-4" />
          <span className="text-sm font-semibold">{stats.expiringSoon}</span>
          <span className="text-xs">expiring</span>
        </div>
      )}
    </div>
  );
}
