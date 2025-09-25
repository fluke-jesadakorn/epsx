import { Suspense } from 'react'
import AdminEPSAnalytics from '@/components/analytics/AdminEPSAnalytics'

// Force dynamic for real-time admin EPS data
export const dynamic = 'force-dynamic'

interface AdminEPSPageProps {
  searchParams: {
    page?: string
    limit?: string
    country?: string
    sector?: string
    sort_by?: string
    min_eps?: string
    min_growth?: string
    search?: string
  }
}

function AdminEPSLoadingSkeleton() {
  return (
    <div className="relative bg-gradient-to-br from-pink-100 via-orange-50 to-yellow-100 dark:bg-gradient-to-br dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      {/* PancakeSwap + Windows Phone background patterns */}
      <div className="absolute inset-0 -z-10 overflow-hidden">
        <div className="absolute inset-0 opacity-20 dark:opacity-10">
          <div className="absolute top-20 left-20 h-32 w-32 bg-gradient-to-br from-pink-400 to-rose-500 rounded-3xl opacity-75"></div>
          <div className="absolute top-40 right-32 h-24 w-24 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full opacity-75"></div>
          <div className="absolute bottom-32 left-1/3 h-28 w-28 bg-gradient-to-br from-orange-400 to-yellow-500 rounded-2xl opacity-75"></div>
        </div>
      </div>

      {/* Header skeleton */}
      <div className="relative container mx-auto px-4 py-8">
        <div className="mb-12">
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-pink-500 via-orange-500 to-yellow-500 p-8 shadow-2xl">
            <div className="h-12 w-96 bg-white/20 rounded-lg opacity-75 mb-4"></div>
            <div className="h-6 w-64 bg-white/20 rounded opacity-75"></div>
            
            {/* Action tiles skeleton */}
            <div className="mt-8 grid grid-cols-1 sm:grid-cols-3 gap-4">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="h-20 bg-white/20 rounded-2xl opacity-75"></div>
              ))}
            </div>
          </div>
        </div>

        {/* Stats banner skeleton */}
        <div className="mb-8 rounded-2xl bg-gradient-to-r from-orange-100 to-yellow-100 p-6 shadow-xl dark:bg-gradient-to-r dark:from-gray-800 dark:to-gray-700">
          <div className="flex items-center gap-4">
            <div className="h-12 w-12 bg-orange-400/50 dark:bg-gray-600/50 rounded-2xl opacity-75"></div>
            <div>
              <div className="h-6 w-48 bg-orange-400/50 dark:bg-gray-600/50 rounded opacity-75 mb-2"></div>
              <div className="h-4 w-64 bg-orange-400/30 dark:bg-gray-600/30 rounded opacity-75"></div>
            </div>
          </div>
        </div>

        {/* Cards grid skeleton */}
        <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:justify-start sm:gap-6 sm:px-0">
          {Array.from({ length: 8 }).map((_, i) => (
            <div key={i} className="w-full max-w-[320px] min-w-[240px] flex-shrink-0 rounded-3xl border-2 border-gray-300/50 dark:border-gray-600/50 bg-white/80 dark:bg-gray-800/80 p-6 shadow-2xl opacity-75">
              <div className="mb-4">
                <div className="h-4 w-16 bg-gray-300 dark:bg-gray-600 rounded mb-2"></div>
                <div className="h-6 w-20 bg-gray-400 dark:bg-gray-500 rounded"></div>
              </div>
              <div className="mb-4">
                <div className="h-6 w-24 bg-gray-300 dark:bg-gray-600 rounded mb-2"></div>
                <div className="h-2 w-full bg-gray-200 dark:bg-gray-700 rounded"></div>
              </div>
              <div className="space-y-3">
                <div className="h-20 bg-gray-200 dark:bg-gray-700 rounded-2xl"></div>
                <div className="h-20 bg-gray-200 dark:bg-gray-700 rounded-2xl"></div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

export default async function EPSAnalyticsPage({ searchParams }: { searchParams: AdminEPSPageProps['searchParams'] }) {
  const resolvedSearchParams = await searchParams

  return (
    <Suspense fallback={<AdminEPSLoadingSkeleton />}>
      <AdminEPSAnalytics searchParams={resolvedSearchParams} />
    </Suspense>
  )
}