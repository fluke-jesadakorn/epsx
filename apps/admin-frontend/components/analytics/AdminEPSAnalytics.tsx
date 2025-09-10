import { Suspense } from 'react'
import { ServerAnalyticsAPI } from '@/lib/api/server-admin-api'
import AdminEPSCard from './AdminEPSCard'
import AdminEPSFilters from './AdminEPSFilters'
import AdminEPSPagination from './AdminEPSPagination'

interface AdminEPSAnalyticsProps {
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

interface EPSQueryParams {
  page: number
  limit: number
  country?: string
  sector?: string
  sort_by?: string
  min_eps?: number
  min_growth?: number
  search?: string
}

function parseSearchParams(searchParams: AdminEPSAnalyticsProps['searchParams']): EPSQueryParams {
  return {
    page: parseInt(searchParams.page || '1', 10),
    limit: parseInt(searchParams.limit || '12', 10),
    country: searchParams.country || undefined,
    sector: searchParams.sector || undefined,
    sort_by: searchParams.sort_by || 'growth_factor',
    min_eps: searchParams.min_eps ? parseFloat(searchParams.min_eps) : undefined,
    min_growth: searchParams.min_growth ? parseFloat(searchParams.min_growth) : undefined,
    search: searchParams.search || undefined,
  }
}

// Mock EPS data structure similar to frontend but with admin metadata
interface AdminEPSData {
  success: boolean
  data: AdminEPSCardData[]
  pagination: {
    page: number
    limit: number
    total: number
    totalPages: number
    hasNext: boolean
    hasPrev: boolean
  }
  admin_metadata: {
    system_health: string
    last_updated: string
    admin_notes?: string
  }
  processing_time_ms: number
}

interface AdminEPSCardData {
  rank: number
  symbol: string
  latest_date: string
  value: number
  active_status: string
  quarterly_performance: {
    quarter: string
    date: string
    price: number
    eps: number
    eps_growth: number
    price_growth: number
    is_estimated?: boolean
  }[]
  next_quarter_estimate?: {
    quarter: string
    announcement_date: string
    days_until_announcement: number
    estimated_eps: number
    estimated_price_target?: number
    confidence: string
  }
  admin_data: {
    admin_priority: 'high' | 'medium' | 'low'
    system_alerts: number
  }
}

// Fetch real EPS data from the same endpoint as frontend
async function getAdminEPSData(params: EPSQueryParams): Promise<AdminEPSData> {
  const baseURL = process.env.BACKEND_URL || 'http://localhost:8080'
  const queryString = new URLSearchParams()
  
  Object.keys(params).forEach(key => {
    const value = (params as any)[key]
    if (value !== undefined && value !== null) {
      queryString.append(key, String(value))
    }
  })

  const url = `${baseURL}/api/v1/public/analytics/rankings?${queryString.toString()}`

  try {
    // Use the same data fetching as frontend but with admin enhancements
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      cache: 'no-store' // No cache for real-time admin data
    })

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    const frontendData = await response.json()
    
    if (!frontendData.success || !frontendData.data) {
      throw new Error('Invalid response from EPS API')
    }

    // Transform frontend data to admin format (simplified for SSR compatibility)
    const adminData: AdminEPSData = {
      success: frontendData.success,
      data: frontendData.data,
      pagination: frontendData.pagination || {
        page: params.page,
        limit: params.limit,
        total: frontendData.data?.length || 0,
        totalPages: Math.ceil((frontendData.data?.length || 0) / params.limit),
        hasNext: false,
        hasPrev: false
      },
      admin_metadata: {
        system_health: frontendData.processing_time_ms < 1000 ? 'excellent' : 'good',
        last_updated: new Date().toISOString()
      },
      processing_time_ms: frontendData.processing_time_ms || 0
    }

    return adminData
  } catch (error) {
    // Use proper error handling for server-side rendering
    const errorMessage = error instanceof Error ? error.message : 'Unknown error'
    
    return {
      success: false,
      data: [],
      pagination: {
        page: params.page,
        limit: params.limit,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false
      },
      admin_metadata: {
        system_health: 'error',
        last_updated: new Date().toISOString(),
        admin_notes: `Fetch failed: ${errorMessage}`
      },
      processing_time_ms: 0
    }
  }
}

async function AdminEPSGrid({ params }: { params: EPSQueryParams }) {
  const data = await getAdminEPSData(params)

  if (!data.success || !data.data || data.data.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="mb-4 text-gray-600 dark:text-gray-300">No EPS data available</p>
        <div className="text-sm text-gray-500 dark:text-gray-400">
          Check system connectivity or contact technical support
        </div>
      </div>
    )
  }

  return (
    <>
      {/* Unified EPS Cards Grid */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:justify-start sm:gap-6 sm:px-0">
        {data.data.map(cardData => (
          <AdminEPSCard key={cardData.symbol} cardData={cardData} />
        ))}
      </div>

      {/* Admin Pagination */}
      {data.pagination && data.pagination.totalPages > 1 && (
        <div className="mt-8">
          <AdminEPSPagination
            pagination={data.pagination}
            currentParams={new URLSearchParams({
              page: String(params.page),
              limit: String(params.limit),
              ...(params.country && { country: params.country }),
              ...(params.sector && { sector: params.sector }),
              ...(params.sort_by && { sort_by: params.sort_by }),
              ...(params.min_eps !== undefined && { min_eps: String(params.min_eps) }),
              ...(params.min_growth !== undefined && { min_growth: String(params.min_growth) }),
              ...(params.search && { search: params.search }),
            }).toString()}
          />
        </div>
      )}

      {/* Admin metadata display */}
      <div className="mt-8 rounded-2xl border border-gray-200/50 dark:border-gray-600/50 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 p-4">
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-4 text-gray-600 dark:text-gray-300">
            <span>🔧 System Health: <span className="capitalize text-green-600 dark:text-green-400">{data.admin_metadata.system_health}</span></span>
            <span>⚡ Response: {data.processing_time_ms}ms</span>
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400">
            Updated: {new Date(data.admin_metadata.last_updated).toLocaleString()}
          </div>
        </div>
      </div>
    </>
  )
}

async function AdminStatsDisplay({ params }: { params: EPSQueryParams }) {
  const data = await getAdminEPSData(params)
  
  return (
    <p className="text-gray-600 dark:text-slate-200">
      🎯 Admin Dashboard: Showing {data.data?.length || 0} stocks with user analytics
      {data.processing_time_ms && (
        <span className="ml-2 text-sm text-gray-500 dark:text-slate-300">
          • Ultra-fast {data.processing_time_ms}ms admin query
        </span>
      )}
    </p>
  )
}

export default async function AdminEPSAnalytics({ searchParams }: AdminEPSAnalyticsProps) {
  const params = parseSearchParams(searchParams)

  return (
    <div className="relative bg-gradient-to-br from-pink-100 via-orange-50 to-yellow-100 dark:bg-gradient-to-br dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      {/* Background patterns - positioned relative to content area only */}
      <div className="absolute inset-0 -z-10 overflow-hidden">
        <div className="absolute inset-0 opacity-20 dark:opacity-10">
          <div className="absolute top-20 left-20 h-32 w-32 rotate-45 bg-gradient-to-br from-pink-400 to-rose-500 rounded-3xl animate-pulse"></div>
          <div className="absolute top-40 right-32 h-24 w-24 rotate-12 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full animate-pulse"></div>
          <div className="absolute bottom-32 left-1/3 h-28 w-28 -rotate-12 bg-gradient-to-br from-orange-400 to-yellow-500 rounded-2xl animate-pulse"></div>
        </div>
      </div>

      {/* Main content */}
      <div className="relative container mx-auto px-4 py-8">
          {/* PancakeSwap inspired header with Windows Phone structure */}
          <div className="mb-12">
            <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-pink-500 via-orange-500 to-yellow-500 p-8 shadow-2xl dark:shadow-orange-500/20">
              <div className="absolute inset-0 bg-gradient-to-r from-white/10 to-transparent dark:from-black/10"></div>
              <div className="absolute top-0 right-0 w-32 h-32 bg-white/5 dark:bg-black/5 rounded-full -translate-y-16 translate-x-16"></div>
              <div className="relative z-10">
                <h1 className="mb-4 text-4xl font-bold tracking-wide text-white sm:text-5xl drop-shadow-lg">
                  🔧 Admin EPS Hub
                </h1>
                <p className="text-lg font-medium text-white/95 max-w-2xl drop-shadow-sm">
                  Advanced EPS analytics with user insights, system monitoring, and administrative controls
                </p>
                
                {/* Admin-specific action tiles */}
                <div className="mt-8 grid grid-cols-1 sm:grid-cols-4 gap-4">
                  <div className="bg-gradient-to-br from-green-400 to-emerald-500 hover:from-green-500 hover:to-emerald-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">👥</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">User Analytics</div>
                  </div>
                  <div className="bg-gradient-to-br from-blue-400 to-cyan-500 hover:from-blue-500 hover:to-cyan-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">🔧</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">System Health</div>
                  </div>
                  <div className="bg-gradient-to-br from-orange-400 to-red-500 hover:from-orange-500 hover:to-red-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">⚡</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">Live Data</div>
                  </div>
                  <div className="bg-gradient-to-br from-purple-400 to-pink-500 hover:from-purple-500 hover:to-pink-600 transition-all duration-300 p-4 cursor-pointer group rounded-2xl shadow-lg hover:shadow-xl hover:scale-105">
                    <div className="text-3xl mb-2">📊</div>
                    <div className="text-white font-bold text-sm drop-shadow-sm">Admin Reports</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Admin stats header */}
          <div className="mb-8 rounded-2xl border-l-4 border-orange-500 bg-gradient-to-r from-orange-100 to-yellow-100 p-6 shadow-xl dark:bg-gradient-to-r dark:from-gray-800 dark:to-gray-700 dark:border-orange-400">
            <div className="flex items-center gap-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-orange-500 to-yellow-500 text-xl font-bold text-white shadow-lg">
                🍰
              </div>
              <div>
                <h2 className="text-2xl font-bold text-orange-800 dark:text-orange-200">
                  🚀 EPS Performance Analytics
                </h2>
                <Suspense
                  fallback={
                    <div className="text-sm font-medium text-orange-600 dark:text-orange-400">
                      Loading admin analytics...
                    </div>
                  }
                >
                  <AdminStatsDisplay params={params} />
                </Suspense>
              </div>
            </div>
          </div>

          {/* Filters */}
          <Suspense
            fallback={
              <div className="text-slate-600 dark:text-slate-200 mb-6">
                Loading admin filters...
              </div>
            }
          >
            <AdminEPSFilters currentParams={params} />
          </Suspense>

          {/* Status Legend with admin enhancements */}
          <div className="mb-8 rounded-3xl border border-pink-200/50 dark:border-gray-600/50 bg-gradient-to-r from-pink-50 via-orange-50 to-yellow-50 dark:bg-gradient-to-r dark:from-gray-800 dark:via-gray-700 dark:to-gray-800 p-6 shadow-xl dark:shadow-orange-500/10">
            <div className="flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-gradient-to-br from-pink-500 to-purple-600 text-lg text-white shadow-lg">
                  🎯
                </div>
                <h4 className="text-xl font-bold text-pink-700 dark:text-pink-300">
                  💫 Admin Status Legend
                </h4>
              </div>
              <div className="flex flex-wrap items-center gap-2 sm:gap-3">
                <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-green-400 to-emerald-500 px-3 py-2 shadow-lg transition-all duration-300 hover:scale-105 hover:from-green-500 hover:to-emerald-600 hover:shadow-xl sm:px-5 sm:py-3">
                  <div className="h-3 w-3 rounded-full bg-white shadow-sm"></div>
                  <span className="text-xs font-bold text-white drop-shadow-sm sm:text-sm">ACTIVE TRACKING</span>
                </div>
                <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-orange-400 to-yellow-500 px-3 py-2 shadow-lg transition-all duration-300 hover:scale-105 hover:from-orange-500 hover:to-yellow-600 hover:shadow-xl sm:px-5 sm:py-3">
                  <div className="h-3 w-3 rounded-full bg-white shadow-sm"></div>
                  <span className="text-xs font-bold text-white drop-shadow-sm sm:text-sm">WATCH LIST</span>
                </div>
                <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-red-400 to-pink-500 px-3 py-2 shadow-lg transition-all duration-300 hover:scale-105 hover:from-red-500 hover:to-pink-600 hover:shadow-xl sm:px-5 sm:py-3">
                  <div className="h-3 w-3 rounded-full bg-white shadow-sm"></div>
                  <span className="text-xs font-bold text-white drop-shadow-sm sm:text-sm">STOPPED</span>
                </div>
                <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-blue-400 to-indigo-500 px-3 py-2 shadow-lg transition-all duration-300 hover:scale-105 hover:from-blue-500 hover:to-indigo-600 hover:shadow-xl sm:px-5 sm:py-3">
                  <div className="h-3 w-3 rounded-full bg-white shadow-sm"></div>
                  <span className="text-xs font-bold text-white drop-shadow-sm sm:text-sm">ADMIN PRIORITY</span>
                </div>
              </div>
            </div>
          </div>

          {/* EPS Cards Grid */}
          <Suspense fallback={
            <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:justify-start sm:gap-6 sm:px-0">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="w-full max-w-[320px] min-w-[240px] flex-shrink-0 rounded-3xl border-2 border-gray-300/50 dark:border-gray-600/50 bg-white/80 dark:bg-gray-800/80 p-6 shadow-2xl dark:shadow-orange-500/10 animate-pulse">
                  <div className="h-32 bg-gray-300 dark:bg-gray-600 rounded-2xl mb-4"></div>
                  <div className="space-y-3">
                    <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-3/4"></div>
                    <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-1/2"></div>
                  </div>
                </div>
              ))}
            </div>
          }>
            <AdminEPSGrid params={params} />
          </Suspense>
        </div>
    </div>
  )
}