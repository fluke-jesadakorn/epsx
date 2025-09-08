import { Suspense } from 'react'
import { UnifiedDataFetchers } from '@/lib/server/unified-data-fetchers'
import { ServerAuth } from '@/lib/server/auth-helpers'
import HubDashboardServer from '@/components/hubs/HubDashboardServer'

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

export default async function DashboardPage() {
  // Server-side authentication and data fetching
  const session = await ServerAuth.getAdminSession()
  
  if (!session.isLoggedIn) {
    // Handle unauthorized access
    return (
      <div className="wp-pancake-page-bg p-6 text-white">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Authentication Required</h1>
          <p className="mb-4">Please log in to access the admin dashboard.</p>
          <a href="/login" className="text-yellow-400 hover:text-yellow-300">
            Go to Login
          </a>
        </div>
      </div>
    )
  }

  // Fetch dashboard data server-side
  const dashboardData = await UnifiedDataFetchers.getDashboardData()

  return (
    <Suspense fallback={<DashboardSkeleton />}>
      <HubDashboardServer 
        session={session}
        dashboardData={dashboardData}
      />
    </Suspense>
  )
}