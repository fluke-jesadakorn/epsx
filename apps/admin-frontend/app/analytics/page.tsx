import { Suspense } from 'react'
import AdminEPSAnalytics from '@/components/analytics/AdminEPSAnalytics'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'

// Force dynamic for real-time analytics data
export const dynamic = 'force-dynamic'

function AnalyticsLoadingSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Header skeleton */}
      <div className="mb-8">
        <div className="h-12 w-80 bg-gray-300/50 rounded-lg animate-pulse mb-3"></div>
        <div className="h-6 w-96 bg-gray-300/30 rounded animate-pulse"></div>
      </div>

      {/* Stats cards skeleton */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 mb-8">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="bg-white/80 dark:bg-gray-800/80 rounded-2xl p-6 animate-pulse">
            <div className="h-4 w-20 bg-gray-300 rounded mb-2"></div>
            <div className="h-8 w-16 bg-gray-400 rounded mb-1"></div>
            <div className="h-3 w-24 bg-gray-300 rounded"></div>
          </div>
        ))}
      </div>

      {/* Widgets skeleton */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <div className="bg-white/80 dark:bg-gray-800/80 rounded-2xl p-6 animate-pulse">
          <div className="h-6 w-32 bg-gray-300 rounded mb-4"></div>
          <div className="space-y-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-16 bg-gray-200 rounded-xl"></div>
            ))}
          </div>
        </div>
        <div className="bg-white/80 dark:bg-gray-800/80 rounded-2xl p-6 animate-pulse">
          <div className="h-6 w-32 bg-gray-300 rounded mb-4"></div>
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-4">
              <div className="h-20 bg-gray-200 rounded"></div>
              <div className="h-20 bg-gray-200 rounded"></div>
            </div>
            <div className="space-y-4">
              <div className="h-20 bg-gray-200 rounded"></div>
              <div className="h-20 bg-gray-200 rounded"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default async function AnalyticsPage() {
  // Check authentication
  const session = await UnifiedAuth.getSession()
  if (!session?.user) {
    notFound()
  }

  return (
    <Suspense fallback={<AnalyticsLoadingSkeleton />}>
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <AdminEPSAnalytics searchParams={{}} />
      </div>
    </Suspense>
  )
}