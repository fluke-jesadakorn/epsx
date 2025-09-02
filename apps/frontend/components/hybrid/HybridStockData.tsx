/**
 * Hybrid Data Strategy Example - Stock Data Component
 * Demonstrates optimal serverless pattern with OIDC authentication
 */

import { Suspense } from 'react'
import { clientData } from '@/lib/client-data'
import { serverData } from '@/lib/server-data'

// ============================================================================
// Server Component - Initial Data Loading
// ============================================================================

interface StockServerDataProps {
  symbol: string
  initialData?: any
}

/**
 * Server Component for initial stock data loading
 * Fetches data at build/request time for optimal performance
 */
async function StockServerData({ symbol, initialData }: StockServerDataProps) {
  // Server-side initial data loading (no client hydration needed)
  const serverStockData = initialData || await serverData.getStock(symbol)
  
  if (!serverStockData) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load initial data for {symbol}</p>
      </div>
    )
  }
  
  return (
    <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
      <h3 className="font-semibold text-blue-900">Server-loaded: {symbol}</h3>
      <p className="text-blue-700">Price: ${serverStockData.price}</p>
      <p className="text-blue-700">EPS: {serverStockData.eps}</p>
      <p className="text-sm text-blue-500">Loaded server-side for optimal performance</p>
    </div>
  )
}

// ============================================================================
// Client Component - Dynamic Updates  
// ============================================================================

interface StockClientDataProps {
  symbol: string
  fallbackData?: any
}

/**
 * Client Component for real-time stock data updates
 * Uses SWR with OIDC Bearer tokens for optimal client-side performance
 */
function StockClientData({ symbol, fallbackData }: StockClientDataProps) {
  // Client-side data fetching with SWR (no server actions)
  const { data, error, isLoading, refetch } = clientData.useStock(symbol)
  
  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load live data for {symbol}</p>
        <button 
          onClick={() => refetch()} 
          className="mt-2 px-3 py-1 bg-red-100 hover:bg-red-200 rounded text-sm"
        >
          Retry
        </button>
      </div>
    )
  }
  
  const stockData = data || fallbackData
  
  if (!stockData && isLoading) {
    return (
      <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg animate-pulse">
        <div className="h-4 bg-gray-300 rounded w-3/4 mb-2"></div>
        <div className="h-3 bg-gray-300 rounded w-1/2 mb-1"></div>
        <div className="h-3 bg-gray-300 rounded w-1/3"></div>
      </div>
    )
  }
  
  return (
    <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
      <h3 className="font-semibold text-green-900">
        Live: {symbol} 
        {isLoading && <span className="ml-2 text-xs text-green-500">Updating...</span>}
      </h3>
      <p className="text-green-700">Price: ${stockData?.price || 'N/A'}</p>
      <p className="text-green-700">EPS: {stockData?.eps || 'N/A'}</p>
      <p className="text-green-700">Growth: {stockData?.growth_rate || 'N/A'}%</p>
      <div className="mt-2 flex gap-2">
        <button 
          onClick={() => refetch()} 
          className="px-3 py-1 bg-green-100 hover:bg-green-200 rounded text-sm"
          disabled={isLoading}
        >
          Refresh
        </button>
        <span className="text-xs text-green-500">
          {data ? 'Live data' : 'Cached/Fallback'}
        </span>
      </div>
    </div>
  )
}

// ============================================================================
// Hybrid Component - Combines Server + Client Strategy
// ============================================================================

interface HybridStockDataProps {
  symbol: string
  enableRealTime?: boolean
  serverSideInitial?: boolean
}

/**
 * Hybrid Stock Data Component
 * Optimal serverless pattern: Server initial load + Client dynamic updates
 */
export function HybridStockData({ 
  symbol, 
  enableRealTime = true, 
  serverSideInitial = true 
}: HybridStockDataProps) {
  const { connectSSE } = clientData.useRealTime(enableRealTime)
  const { preloadStock } = clientData.useCache()
  
  // Connect to real-time updates
  if (enableRealTime && typeof window !== 'undefined') {
    connectSSE()
  }
  
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold">Hybrid Data Strategy: {symbol}</h2>
        <div className="flex gap-2">
          <button 
            onClick={() => preloadStock(symbol)}
            className="px-3 py-1 bg-blue-100 hover:bg-blue-200 rounded text-sm"
          >
            Preload
          </button>
        </div>
      </div>
      
      <div className="grid gap-4 md:grid-cols-2">
        {/* Server-side initial data */}
        {serverSideInitial ? (
          <Suspense fallback={
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
              <div className="animate-pulse space-y-2">
                <div className="h-4 bg-gray-300 rounded w-3/4"></div>
                <div className="h-3 bg-gray-300 rounded w-1/2"></div>
              </div>
            </div>
          }>
            <StockServerData symbol={symbol} />
          </Suspense>
        ) : (
          <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
            <p className="text-gray-500">Server-side loading disabled</p>
          </div>
        )}
        
        {/* Client-side dynamic data */}
        <StockClientData symbol={symbol} />
      </div>
      
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <h4 className="font-medium text-yellow-800 mb-2">Hybrid Strategy Benefits:</h4>
        <ul className="text-sm text-yellow-700 space-y-1">
          <li>✅ Server-side initial load (SEO + performance)</li>
          <li>✅ Client-side real-time updates (dynamic UX)</li>
          <li>✅ OIDC Bearer token authentication</li>
          <li>✅ No Server Actions with fetch() (serverless optimal)</li>
          <li>✅ SWR caching and revalidation</li>
          <li>✅ Error boundaries and loading states</li>
        </ul>
      </div>
    </div>
  )
}

// ============================================================================
// Batch Stock Data Example
// ============================================================================

interface BatchStockDataProps {
  symbols: string[]
  enableRealTime?: boolean
}

export function HybridBatchStockData({ symbols, enableRealTime = true }: BatchStockDataProps) {
  const { data, error, isLoading, refetch } = clientData.useBatchStocks(symbols)
  const { invalidateBatchStocks } = clientData.useCache()
  const { connectSSE } = clientData.useRealTime(enableRealTime)
  
  // Connect to real-time updates
  if (enableRealTime && typeof window !== 'undefined') {
    connectSSE()
  }
  
  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load batch stock data</p>
        <button 
          onClick={() => refetch()} 
          className="mt-2 px-3 py-1 bg-red-100 hover:bg-red-200 rounded text-sm"
        >
          Retry Batch
        </button>
      </div>
    )
  }
  
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold">Batch Stock Data ({symbols.length} symbols)</h2>
        <div className="flex gap-2">
          <button 
            onClick={() => refetch()}
            disabled={isLoading}
            className="px-3 py-1 bg-blue-100 hover:bg-blue-200 rounded text-sm disabled:opacity-50"
          >
            {isLoading ? 'Loading...' : 'Refresh All'}
          </button>
          <button 
            onClick={() => invalidateBatchStocks(symbols)}
            className="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded text-sm"
          >
            Clear Cache
          </button>
        </div>
      </div>
      
      <div className="grid gap-2 md:grid-cols-2 lg:grid-cols-3">
        {symbols.map(symbol => (
          <div key={symbol} className="p-3 bg-white border rounded-lg">
            <h4 className="font-medium">{symbol}</h4>
            {data[symbol] ? (
              <div className="text-sm space-y-1">
                <p>Price: ${data[symbol].price}</p>
                <p>EPS: {data[symbol].eps}</p>
              </div>
            ) : isLoading ? (
              <div className="text-sm text-gray-500">Loading...</div>
            ) : (
              <div className="text-sm text-red-500">No data</div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

export default HybridStockData