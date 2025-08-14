import React from 'react';
import { Skeleton } from '@/components/ui/skeleton';
import { Card, CardContent } from '@epsx/ui';

interface AnalyticsCardSkeletonProps {
  index: number;
}

/**
 * Loading skeleton specifically designed for analytics cards
 * Matches the FinancialCard layout and design
 */
export function AnalyticsCardSkeleton({ index }: AnalyticsCardSkeletonProps): React.JSX.Element {
  return (
    <Card className="w-full transition-all duration-200 hover:shadow-2xl border-0 bg-gradient-to-br from-blue-50/50 via-purple-50/50 to-pink-50/50 dark:from-[#232946]/50 dark:via-[#1a1a2e]/50 dark:to-[#0f1021]/50 rounded-3xl shadow-lg relative animate-pulse">
      {/* Rank Badge Skeleton */}
      <div className="absolute top-4 left-4 z-30 w-12 h-12 flex items-center justify-center">
        <div className="w-10 h-10 bg-gradient-to-br from-orange-200/70 via-yellow-100/60 to-amber-100/50 dark:from-orange-900/40 dark:via-yellow-900/30 dark:to-amber-900/30 rounded-full flex items-center justify-center">
          <span className="text-sm font-bold text-slate-600 dark:text-slate-400">
            #{index + 1}
          </span>
        </div>
      </div>

      {/* Floating decorative elements */}
      <div className="absolute top-2 right-2 text-2xl opacity-20">
        🥞
      </div>
      <div className="absolute bottom-2 left-2 text-lg opacity-15">
        💰
      </div>

      <CardContent className="p-6 pt-16">
        {/* Header Skeleton */}
        <div className="flex items-start justify-between mb-5">
          <div className="flex items-center gap-3">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-2xl opacity-30">📈</span>
                <Skeleton className="h-8 w-20" />
                <span className="text-sm opacity-60">🚀</span>
              </div>
              <div className="flex items-center gap-2">
                <Skeleton className="h-4 w-16" />
              </div>
            </div>
          </div>
          <div className="text-right space-y-2">
            <div className="flex items-center gap-1">
              <span className="text-xs opacity-30">📅</span>
              <Skeleton className="h-3 w-16" />
            </div>
            <Skeleton className="h-5 w-24" />
          </div>
        </div>

        {/* Key Metrics Skeleton */}
        <div className="grid grid-cols-3 gap-3 mb-6">
          {/* Latest Price Card */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-slate-50 to-slate-100/50 dark:from-slate-900/20 dark:to-slate-800/20 border border-slate-200/50 dark:border-slate-700/30">
            <Skeleton className="h-3 w-16 mb-2" />
            <Skeleton className="h-6 w-20" />
          </div>
          
          {/* Latest EPS Card */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-blue-50 to-blue-100/50 dark:from-blue-900/20 dark:to-blue-800/20 border border-blue-200/50 dark:border-blue-700/30">
            <Skeleton className="h-3 w-16 mb-2" />
            <Skeleton className="h-6 w-16" />
          </div>
          
          {/* Average Growth Card */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-emerald-50 to-emerald-100/50 dark:from-emerald-900/20 dark:to-emerald-800/20 border border-emerald-200/50 dark:border-emerald-700/30">
            <Skeleton className="h-3 w-20 mb-2" />
            <div className="flex items-center gap-2">
              <Skeleton className="h-6 w-16" />
            </div>
          </div>
        </div>

        {/* TradingView Link Skeleton */}
        <div className="mb-6">
          <div className="flex items-center justify-center gap-2 w-full p-3 rounded-xl bg-gradient-to-r from-orange-200/50 to-yellow-200/50 dark:from-orange-700/30 dark:to-yellow-700/30 border border-orange-200/50 dark:border-orange-700/30">
            <span className="text-lg opacity-30">📊</span>
            <Skeleton className="h-5 w-40" />
            <span className="text-lg opacity-30">🚀</span>
          </div>
        </div>

        {/* Quarterly Performance Skeleton */}
        <div className="space-y-3">
          <div className="flex items-center gap-3 pb-2 border-b border-slate-200/70 dark:border-slate-700/70">
            <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-orange-200/50 to-yellow-200/50 dark:from-orange-700/30 dark:to-yellow-700/30 flex items-center justify-center">
              <span className="text-white text-xs opacity-30">📊</span>
            </div>
            <Skeleton className="h-5 w-32" />
            <div className="ml-auto">
              <Skeleton className="h-5 w-12" />
            </div>
          </div>

          {/* Table Header Skeleton */}
          <div className="hidden sm:grid grid-cols-5 gap-3 px-3 py-2 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
            {['Quarter', 'Price', 'EPS', 'EPS %', 'Price %'].map((_, i) => (
              <Skeleton key={i} className="h-4 w-16" />
            ))}
          </div>

          {/* Quarter Rows Skeleton */}
          <div className="space-y-2">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="grid grid-cols-5 gap-3 px-3 py-2 text-sm">
                <Skeleton className="h-4 w-16" />
                <Skeleton className="h-4 w-20" />
                <Skeleton className="h-4 w-16" />
                <Skeleton className="h-4 w-16" />
                <Skeleton className="h-4 w-16" />
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

/**
 * Grid of analytics card skeletons
 */
export function AnalyticsCardsSkeletonGrid({ count = 10 }: { count?: number }): React.JSX.Element {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      {[...Array(count)].map((_, index) => (
        <div
          key={index}
          className="animate-fadeIn relative"
          style={{
            animationDelay: `${index * 150}ms`,
            animationDuration: '600ms',
            animationFillMode: 'both',
          }}
        >
          <AnalyticsCardSkeleton index={index} />
        </div>
      ))}
    </div>
  );
}

export default AnalyticsCardSkeleton;
