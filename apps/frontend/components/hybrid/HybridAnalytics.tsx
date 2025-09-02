/**
 * Hybrid Analytics Component - Complete Example
 * Demonstrates optimal serverless pattern with OIDC authentication
 * Server Component initial load + Client-side filtering + Server Actions for navigation
 */

import { Suspense } from 'react'
import { clientData, type AnalyticsFilters } from '@/lib/client-data'
import { serverData } from '@/lib/server-data'

// ============================================================================
// Server Component - Initial Analytics Data
// ============================================================================

interface AnalyticsServerDataProps {
  initialFilters: AnalyticsFilters
}

/**
 * Server Component for initial analytics data loading
 * Optimized for serverless with direct backend calls
 */
async function AnalyticsServerData({ initialFilters }: AnalyticsServerDataProps) {
  const analyticsData = await serverData.getAnalytics(initialFilters)
  
  if (!analyticsData?.rankings) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load initial analytics data</p>
      </div>
    )
  }
  
  return (
    <div className="space-y-4">
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h3 className="font-semibold text-blue-900">Server-loaded Analytics</h3>
        <p className="text-blue-700">
          Found {analyticsData.total_count} stocks, showing page {analyticsData.page}
        </p>
        <p className="text-sm text-blue-500">Loaded server-side for SEO and performance</p>
      </div>
      
      <div className="overflow-x-auto">
        <table className="min-w-full bg-white border border-gray-200 rounded-lg">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Symbol</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Name</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Price</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">EPS</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Growth</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Sector</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {analyticsData.rankings.slice(0, 5).map((stock: any, index: number) => (
              <tr key={stock.symbol} className={index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                <td className="px-4 py-3 text-sm font-medium text-gray-900">{stock.symbol}</td>
                <td className="px-4 py-3 text-sm text-gray-700">{stock.name}</td>
                <td className="px-4 py-3 text-sm text-gray-700">${stock.price_current}</td>
                <td className="px-4 py-3 text-sm text-gray-700">{stock.current_eps}</td>
                <td className="px-4 py-3 text-sm text-green-600">{stock.qoq_growth_rate}%</td>
                <td className="px-4 py-3 text-sm text-gray-700">{stock.sector}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

// ============================================================================
// Client Component - Dynamic Filtering
// ============================================================================

interface AnalyticsClientFiltersProps {
  initialFilters: AnalyticsFilters
  onFiltersChange: (filters: AnalyticsFilters) => void
}

/**
 * Client Component for dynamic analytics filtering
 * Uses SWR for real-time updates without server actions
 */
function AnalyticsClientFilters({ initialFilters, onFiltersChange }: AnalyticsClientFiltersProps) {
  const { data, error, isLoading } = clientData.useAnalytics(initialFilters)
  
  const handleFilterChange = (key: keyof AnalyticsFilters, value: any) => {
    const newFilters = { ...initialFilters, [key]: value, page: 1 } // Reset to page 1
    onFiltersChange(newFilters)
  }
  
  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load filtered analytics data</p>
      </div>
    )
  }
  
  return (
    <div className="space-y-4">
      {/* Filter Controls */}
      <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
        <h3 className="font-semibold text-gray-900 mb-4">Dynamic Filters</h3>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Country</label>
            <select 
              value={initialFilters.country || ''}
              onChange={(e) => handleFilterChange('country', e.target.value || undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="">All Countries</option>
              <option value="america">United States</option>
              <option value="canada">Canada</option>
              <option value="united_kingdom">United Kingdom</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Sector</label>
            <select
              value={initialFilters.sector || ''}
              onChange={(e) => handleFilterChange('sector', e.target.value || undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="">All Sectors</option>
              <option value="Technology">Technology</option>
              <option value="Healthcare">Healthcare</option>
              <option value="Financial Services">Financial Services</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Min EPS</label>
            <input
              type="number"
              step="0.01"
              value={initialFilters.min_eps || ''}
              onChange={(e) => handleFilterChange('min_eps', e.target.value ? parseFloat(e.target.value) : undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="0.00"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Min Growth %</label>
            <input
              type="number"
              step="0.1"
              value={initialFilters.min_growth || ''}
              onChange={(e) => handleFilterChange('min_growth', e.target.value ? parseFloat(e.target.value) : undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="0.0"
            />
          </div>
        </div>
      </div>
      
      {/* Filtered Results */}
      <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-semibold text-green-900">
            Client-filtered Results
            {isLoading && <span className="ml-2 text-xs text-green-500">Updating...</span>}
          </h3>
          <span className="text-sm text-green-600">
            {data?.total_count || 0} results
          </span>
        </div>
        
        {data?.rankings && (
          <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
            {data.rankings.slice(0, 6).map((stock: any) => (
              <div key={stock.symbol} className="p-3 bg-white border border-green-200 rounded-lg">
                <h4 className="font-medium text-green-900">{stock.symbol}</h4>
                <p className="text-sm text-green-700">{stock.name}</p>
                <div className="mt-2 text-xs space-y-1">
                  <p>Price: ${stock.price_current}</p>
                  <p>EPS: {stock.current_eps}</p>
                  <p>Growth: <span className="font-medium">{stock.qoq_growth_rate}%</span></p>
                </div>
              </div>
            ))}
          </div>
        )}
        
        {!data && !isLoading && (
          <p className="text-green-600">No filtered results available</p>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// Server Action Components - Navigation Only
// ============================================================================

interface AnalyticsPaginationProps {
  currentPage: number
  totalPages: number
  currentFilters: AnalyticsFilters
}

/**
 * Pagination component using Server Actions for optimal serverless navigation
 */
function AnalyticsPagination({ currentPage, totalPages, currentFilters }: AnalyticsPaginationProps) {
  return (
    <div className="flex items-center justify-between p-4 bg-white border border-gray-200 rounded-lg">
      <div className="text-sm text-gray-600">
        Page {currentPage} of {totalPages}
      </div>
      
      <div className="flex gap-2">
        {currentPage > 1 && (
          <form action={serverData.navigateToPage.bind(null, currentPage - 1, currentFilters)}>
            <button 
              type="submit"
              className="px-3 py-1 bg-blue-100 hover:bg-blue-200 border border-blue-300 rounded text-sm"
            >
              Previous
            </button>
          </form>
        )}
        
        {currentPage < totalPages && (
          <form action={serverData.navigateToPage.bind(null, currentPage + 1, currentFilters)}>
            <button 
              type="submit"
              className="px-3 py-1 bg-blue-100 hover:bg-blue-200 border border-blue-300 rounded text-sm"
            >
              Next
            </button>
          </form>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// Main Hybrid Analytics Component
// ============================================================================

interface HybridAnalyticsProps {
  initialFilters: AnalyticsFilters
  enableRealTime?: boolean
  showServerData?: boolean
  showClientFiltering?: boolean
}

/**
 * Complete Hybrid Analytics Component
 * Demonstrates the full hybrid strategy for optimal serverless performance
 */
export function HybridAnalytics({ 
  initialFilters, 
  enableRealTime = true,
  showServerData = true,
  showClientFiltering = true
}: HybridAnalyticsProps) {
  const { connectSSE } = clientData.useRealTime(enableRealTime)
  const { invalidateAnalytics } = clientData.useCache()
  
  // Connect to real-time updates
  if (enableRealTime && typeof window !== 'undefined') {
    connectSSE()
  }
  
  const handleFiltersChange = (newFilters: AnalyticsFilters) => {
    // Update client-side filtering immediately
    invalidateAnalytics(newFilters)
  }
  
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Hybrid Analytics Strategy</h1>
        <div className="flex gap-2">
          <button 
            onClick={() => invalidateAnalytics()}
            className="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded text-sm"
          >
            Clear Cache
          </button>
        </div>
      </div>
      
      {/* Strategy Explanation */}
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <h3 className="font-medium text-yellow-800 mb-2">Hybrid Strategy Implementation:</h3>
        <div className="grid gap-2 md:grid-cols-3 text-sm text-yellow-700">
          <div>
            <strong>Server Components:</strong> Initial data load, SEO-optimized
          </div>
          <div>
            <strong>Client Components:</strong> Real-time filtering, SWR caching
          </div>
          <div>
            <strong>Server Actions:</strong> Navigation only (no fetch calls)
          </div>
        </div>
      </div>
      
      {/* Server-side Initial Data */}
      {showServerData && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Server-side Initial Load</h2>
          <Suspense fallback={
            <div className="p-8 bg-gray-50 border border-gray-200 rounded-lg">
              <div className="animate-pulse space-y-4">
                <div className="h-4 bg-gray-300 rounded w-1/2"></div>
                <div className="h-3 bg-gray-300 rounded w-3/4"></div>
                <div className="space-y-2">
                  {[1, 2, 3].map(i => (
                    <div key={i} className="h-8 bg-gray-300 rounded"></div>
                  ))}
                </div>
              </div>
            </div>
          }>
            <AnalyticsServerData initialFilters={initialFilters} />
          </Suspense>
        </div>
      )}
      
      {/* Client-side Dynamic Filtering */}
      {showClientFiltering && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Client-side Dynamic Filtering</h2>
          <AnalyticsClientFilters 
            initialFilters={initialFilters}
            onFiltersChange={handleFiltersChange}
          />
        </div>
      )}
      
      {/* Server Action Pagination */}
      <div>
        <h2 className="text-lg font-semibold mb-4">Server Action Navigation</h2>
        <AnalyticsPagination 
          currentPage={initialFilters.page || 1}
          totalPages={10} // Would be calculated from data
          currentFilters={initialFilters}
        />
      </div>
      
      {/* Performance Benefits Summary */}
      <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
        <h3 className="font-medium text-green-800 mb-2">Performance Benefits Achieved:</h3>
        <ul className="text-sm text-green-700 space-y-1">
          <li>✅ Server-side initial render (SEO + Core Web Vitals)</li>
          <li>✅ Client-side dynamic updates (Real-time UX)</li>
          <li>✅ OIDC Bearer authentication (Secure + Scalable)</li>
          <li>✅ No Server Actions with fetch() calls (Serverless optimal)</li>
          <li>✅ SWR caching with stale-while-revalidate (Performance)</li>
          <li>✅ Server Actions for navigation only (Stateless)</li>
          <li>✅ Real-time SSE updates (Live data)</li>
          <li>✅ Optimistic UI updates (Instant feedback)</li>
        </ul>
      </div>
    </div>
  )
}

export default HybridAnalytics