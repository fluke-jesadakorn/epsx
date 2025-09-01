import { Suspense } from 'react'
import AnalyticsHub from '@/components/hubs/AnalyticsHub'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

function AnalyticsHubSkeleton() {
  return (
    <div className="wp-pancake-page-bg p-6">
      <div className="mb-8">
        <div className="h-10 bg-gray-700/50 rounded w-80 mb-2 animate-pulse"></div>
        <div className="h-4 bg-gray-700/50 rounded w-64 animate-pulse"></div>
      </div>
      
      {/* Metrics tiles skeleton */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 gap-4 mb-8">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="h-24 bg-gray-700/30 backdrop-blur-sm rounded-lg animate-pulse border border-yellow-500/10"></div>
        ))}
      </div>
      
      {/* Pivot navigation skeleton */}
      <div className="mb-6">
        <div className="flex gap-4 border-b border-gray-200 dark:border-gray-700 pb-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="h-6 bg-gray-200 dark:bg-gray-700 rounded w-24 animate-pulse"></div>
          ))}
        </div>
      </div>
      
      {/* Main widgets skeleton */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <div className="h-80 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        <div className="h-80 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        <div className="xl:col-span-2 h-64 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
      </div>
      
      {/* Summary widgets skeleton */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-48 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        ))}
      </div>
    </div>
  )
}

export default function AnalyticsPage() {
  return (
    <Suspense fallback={<AnalyticsHubSkeleton />}>
      <AnalyticsHub />
    </Suspense>
  )
}