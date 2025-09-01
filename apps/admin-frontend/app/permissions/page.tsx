import { Suspense } from 'react'
import PermissionsHub from '@/components/hubs/PermissionsHub'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

function PermissionsHubSkeleton() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="mb-8">
        <div className="h-10 bg-gray-200 dark:bg-gray-700 rounded w-72 mb-2 animate-pulse"></div>
        <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-56 animate-pulse"></div>
      </div>
      
      {/* Stats cards skeleton */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="h-24 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        ))}
      </div>
      
      {/* Health banner skeleton */}
      <div className="mb-6 h-20 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
      
      {/* Pivot navigation skeleton */}
      <div className="mb-6">
        <div className="flex gap-4 border-b border-gray-200 dark:border-gray-700 pb-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="h-6 bg-gray-200 dark:bg-gray-700 rounded w-20 animate-pulse"></div>
          ))}
        </div>
      </div>
      
      {/* Controls skeleton */}
      <div className="flex gap-2 mb-6">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="w-32 h-12 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        ))}
      </div>
      
      {/* Permissions list skeleton */}
      <div className="space-y-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="h-20 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
        ))}
      </div>
    </div>
  )
}

export default function AdminPermissionsPage() {
  return (
    <Suspense fallback={<PermissionsHubSkeleton />}>
      <PermissionsHub />
    </Suspense>
  )
}