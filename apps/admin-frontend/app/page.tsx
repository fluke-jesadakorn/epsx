import { Suspense } from 'react'
import HubDashboard from '@/components/hubs/HubDashboard'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

function DashboardSkeleton() {
  return (
    <div className="wp-pancake-page-bg p-6">
      <div className="mb-8">
        <div className="h-10 bg-gray-700/50 rounded w-64 mb-2 animate-pulse"></div>
        <div className="h-4 bg-gray-700/50 rounded w-48 animate-pulse"></div>
      </div>
      
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 gap-4 max-w-7xl">
        {Array.from({ length: 9 }).map((_, i) => (
          <div key={i} className="h-32 bg-gray-700/30 backdrop-blur-sm rounded-lg animate-pulse border border-yellow-500/10"></div>
        ))}
      </div>
    </div>
  )
}

export default function DashboardPage() {
  return (
    <Suspense fallback={<DashboardSkeleton />}>
      <HubDashboard />
    </Suspense>
  )
}