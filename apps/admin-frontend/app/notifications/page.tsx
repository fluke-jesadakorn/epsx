import { Suspense } from 'react'
import NotificationsHub from '@/components/hubs/NotificationsHub'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

function NotificationsHubSkeleton() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="mb-8">
        <div className="h-10 bg-gray-200 dark:bg-gray-700 rounded w-80 mb-2 animate-pulse"></div>
        <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-64 animate-pulse"></div>
      </div>
      
      {/* Stats cards skeleton */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="h-24 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        ))}
      </div>
      
      {/* Pivot navigation skeleton */}
      <div className="mb-6">
        <div className="flex gap-4 border-b border-gray-200 dark:border-gray-700 pb-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="h-6 bg-gray-200 dark:bg-gray-700 rounded w-20 animate-pulse"></div>
          ))}
        </div>
      </div>
      
      {/* Controls skeleton */}
      <div className="flex flex-col sm:flex-row gap-4 mb-6">
        <div className="flex gap-2">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="w-32 h-12 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
          ))}
        </div>
        <div className="flex-1 max-w-sm h-12 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
      </div>
      
      {/* Real-time status skeleton */}
      <div className="mb-6 h-20 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
      
      {/* Notifications list skeleton */}
      <div className="space-y-4">
        {Array.from({ length: 8 }).map((_, i) => (
          <div key={i} className="h-24 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        ))}
      </div>
      
      {/* Settings sections skeleton */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="h-48 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        <div className="h-48 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
      </div>
    </div>
  )
}

export default function NotificationsPage() {
  return (
    <Suspense fallback={<NotificationsHubSkeleton />}>
      <NotificationsHub />
    </Suspense>
  )
}