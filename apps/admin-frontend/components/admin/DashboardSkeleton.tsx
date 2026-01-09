/**
 * Dashboard Loading Skeleton
 * Shows while dashboard data is being fetched
 */

import { adminCardVariants } from '@/design-system'
import { cn } from '@/lib/utils'
import React from 'react'

/**
 *
 */
export function DashboardSkeleton(): React.JSX.Element {
  return (
    <div className="space-y-8 p-6">
      {/* Header Skeleton */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }), 'p-6')}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-14 h-14 bg-muted rounded-full" />
            <div>
              <div className="h-8 w-48 bg-muted rounded mb-2" />
              <div className="h-4 w-32 bg-muted rounded" />
            </div>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-muted rounded-full" />
            <div className="h-4 w-20 bg-muted rounded" />
          </div>
        </div>
      </div>

      {/* Stats Grid Skeleton */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={`stats-card-${i}`} className={cn(adminCardVariants({ variant: 'pancake' }), 'p-6')}>
            <div className="flex items-center justify-between mb-4">
              <div className="h-5 w-20 bg-muted rounded" />
              <div className="w-5 h-5 bg-muted rounded" />
            </div>
            <div className="h-8 w-16 bg-muted rounded mb-2" />
            <div className="h-4 w-24 bg-muted rounded" />
          </div>
        ))}
      </div>

      {/* Quick Actions Skeleton */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className={cn(adminCardVariants({ variant: 'pancake' }), 'p-6')}>
          <div className="h-6 w-32 bg-muted rounded mb-4" />
          <div className="space-y-3">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={`quick-action-${i}`} className="flex items-center gap-3 p-3">
                <div className="w-8 h-8 bg-muted rounded-lg" />
                <div className="flex-1">
                  <div className="h-4 w-32 bg-muted rounded mb-1" />
                  <div className="h-3 w-48 bg-muted rounded" />
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className={cn(adminCardVariants({ variant: 'pancake' }), 'p-6')}>
          <div className="h-6 w-32 bg-muted rounded mb-4" />
          <div className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={`metric-${i}`} className="flex items-center justify-between">
                <div className="h-4 w-24 bg-muted rounded" />
                <div className="h-4 w-12 bg-muted rounded" />
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Recent Activity Skeleton */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {Array.from({ length: 2 }).map((_, i) => (
          <div key={`activity-card-${i}`} className={cn(adminCardVariants({ variant: 'pancake' }), 'p-6')}>
            <div className="flex items-center justify-between mb-4">
              <div className="h-6 w-32 bg-muted rounded" />
              <div className="h-4 w-16 bg-muted rounded" />
            </div>
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, j) => (
                <div key={`activity-${i}-${j}`} className="flex items-center gap-3 p-3">
                  <div className="w-8 h-8 bg-muted rounded-full" />
                  <div className="flex-1">
                    <div className="h-4 w-32 bg-muted rounded mb-1" />
                    <div className="h-3 w-20 bg-muted rounded" />
                  </div>
                  <div className="h-3 w-16 bg-muted rounded" />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}