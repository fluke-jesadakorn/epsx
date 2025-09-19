/**
 * Hybrid Data Strategy - Server-side data utilities
 * Optimized for serverless deployment - NO FETCH CALLS in server actions
 */

import { redirect } from 'next/navigation'
import { getOIDCAccessTokenFromCookies } from '@/lib/server/jwt'
import { getBackendUrl } from '../../../shared/utils/url-resolver'
import { isServerComponentContext } from '@/lib/utils'

// ============================================================================
// Server-Side Data Types
// ============================================================================

export interface EPSQueryParams {
  page: number
  limit: number
  country?: string
  sector?: string
  sort_by?: string
  min_eps?: number
  min_growth?: number
  search?: string
}

export interface QuarterlyPerformanceData {
  quarter: string
  date: string
  price: number
  eps: number
  eps_growth: number
  price_growth: number
  announcement_date?: string
  announcement_timestamp?: number
  is_estimated?: boolean
}

export interface NextQuarterEstimate {
  quarter: string
  estimated_eps: number
  announcement_date: string
  announcement_timestamp: number
  days_until_announcement: number
  estimated_price_target?: number
  confidence: string
}

export interface SymbolCardData {
  rank: number
  symbol: string
  latest_date: string
  value: number
  active_status: string
  quarterly_performance: QuarterlyPerformanceData[]
  next_quarter_estimate?: NextQuarterEstimate
}

export interface ServerAnalyticsResponse {
  success: boolean
  rankings: SymbolCardData[]
  pagination?: {
    page: number
    limit: number
    total: number
    totalPages: number
    hasNext: boolean
    hasPrev: boolean
  }
  metadata?: {
    available_countries: string[]
    available_sectors: string[]
    request_timestamp: string
    data_source: string
  }
  message?: string
  processing_time_ms?: number
}

export interface FilterOptions {
  countries: string[]
  sectors: string[]
  exchanges?: string[]
  stock_types?: string[]
}

// ============================================================================
// Server-Side Initial Data Loading (Server Components Only)
// ============================================================================

/**
 * Server-side data fetcher for initial page load
 * Uses OIDC access token from cookies, only for Server Components
 * Supports both authenticated and unauthenticated requests
 */
async function serverFetcher(url: string, options: RequestInit = {}) {
  try {
    const accessToken = await getOIDCAccessTokenFromCookies()
    const backendUrl = getBackendUrl('server')
    const fullUrl = url.startsWith('http') ? url : `${backendUrl}${url}`
    
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'User-Agent': 'EPSX-Frontend-Server/1.0',
      ...options.headers,
    }
    
    // Add Authorization header only if we have a valid access token
    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`
    }
    
    const fetchConfig: RequestInit = {
      ...options,
      headers,
      // Server-side fetch should not use credentials: 'include'
      cache: 'no-store', // Ensure fresh data for serverless
    }
    
    
    const response = await fetch(fullUrl, fetchConfig)
    
    if (!response.ok) {
      // Try to get error details from response
      let errorData: any = {}
      try {
        errorData = await response.json()
      } catch (e) {
        console.warn('Could not parse error response as JSON:', e)
      }
      
      
      // For 401 Unauthorized, return null instead of throwing for graceful handling
      if (response.status === 401) {
        return null
      }
      
      // For other errors, throw with detailed information
      throw new Error(`Server API Error: ${response.status} ${response.statusText} - ${errorData.message || 'Unknown error'}`)
    }
    
    const data = await response.json()
    return data
    
  } catch (error) {
    const errorDetails = {
      message: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
      url,
      type: error instanceof Error ? error.constructor.name : typeof error,
      status: error && typeof error === 'object' && 'status' in error ? error.status : undefined,
      code: error && typeof error === 'object' && 'code' in error ? error.code : undefined,
      cause: error instanceof Error && error.cause ? error.cause : undefined
    }
    
    console.error('💥 Server fetch exception:', errorDetails)
    throw error
  }
}

/**
 * Server-side stock data fetching for initial page load
 * Use only in Server Components, never in Server Actions
 * Supports both authenticated and unauthenticated requests
 */
export async function getServerStockData(symbol: string) {
  try {
    const result = await serverFetcher(`/api/v1/stocks/${symbol}`)
    
    // Handle unauthenticated users gracefully
    if (result === null) {
      return null
    }
    
    return result
  } catch (error) {
    const errorDetails = {
      message: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
      type: error instanceof Error ? error.constructor.name : typeof error,
      symbol,
      endpoint: `/api/v1/stocks/${symbol}`,
      timestamp: new Date().toISOString()
    }
    
    console.error(`Failed to fetch server stock data for ${symbol}:`, errorDetails)
    return null
  }
}

/**
 * Server-side batch stock data fetching for initial page load
 * Use only in Server Components, never in Server Actions
 * Supports both authenticated and unauthenticated requests
 */
export async function getServerBatchStocks(symbols: string[]) {
  try {
    const symbolsParam = symbols.join(',')
    const result = await serverFetcher(`/api/v1/stocks/batch?symbols=${symbolsParam}`)
    
    // Handle unauthenticated users gracefully
    if (result === null) {
      return { stocks: {} }
    }
    
    return result
  } catch (error) {
    const errorDetails = {
      message: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
      type: error instanceof Error ? error.constructor.name : typeof error,
      symbols,
      symbolsParam: symbols.join(','),
      endpoint: `/api/v1/stocks/batch`,
      timestamp: new Date().toISOString()
    }
    
    console.error('Failed to fetch server batch stocks:', errorDetails)
    return { stocks: {} }
  }
}

/**
 * Server-side analytics data fetching for initial page load
 * Use only in Server Components, never in Server Actions
 * Supports both authenticated and unauthenticated requests
 */
export async function getServerAnalytics(filters: EPSQueryParams): Promise<ServerAnalyticsResponse> {
  try {
    
    const params = new URLSearchParams()
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== null && value !== '') {
        params.append(key, String(value))
      }
    })
    
    const endpoint = `/api/v1/public/analytics/rankings?${params.toString()}`
    
    const result = await serverFetcher(endpoint)
    
    // Handle unauthenticated users gracefully with mock data for development
    if (result === null) {
      
      // Generate mock data for development
      const mockRankings: SymbolCardData[] = [
        {
          rank: 1,
          symbol: 'AAPL',
          latest_date: '2024-Q3',
          value: 150.25,
          active_status: 'TRACK',
          quarterly_performance: [
            {
              quarter: 'Q3 2024',
              date: 'Oct 31, 2024',
              price: 150.25,
              eps: 1.46,
              eps_growth: 12.5,
              price_growth: 8.3,
              is_estimated: false
            },
            {
              quarter: 'Q2 2024',
              date: 'Jul 31, 2024',
              price: 142.80,
              eps: 1.30,
              eps_growth: 8.2,
              price_growth: 5.1,
              is_estimated: false
            }
          ],
          next_quarter_estimate: {
            quarter: '2025-Q1',
            estimated_eps: 1.55,
            announcement_date: 'Est. Jan 30, 2025',
            announcement_timestamp: Date.now() + (45 * 24 * 60 * 60 * 1000),
            days_until_announcement: 45,
            confidence: 'High'
          }
        },
        {
          rank: 2,
          symbol: 'MSFT',
          latest_date: '2024-Q3',
          value: 342.75,
          active_status: 'TRACK',
          quarterly_performance: [
            {
              quarter: 'Q3 2024',
              date: 'Oct 24, 2024',
              price: 342.75,
              eps: 2.95,
              eps_growth: 15.7,
              price_growth: 12.4,
              is_estimated: false
            }
          ],
          next_quarter_estimate: {
            quarter: '2025-Q1',
            estimated_eps: 3.10,
            announcement_date: 'Est. Jan 24, 2025',
            announcement_timestamp: Date.now() + (38 * 24 * 60 * 60 * 1000),
            days_until_announcement: 38,
            confidence: 'High'
          }
        },
        {
          rank: 3,
          symbol: 'GOOGL',
          latest_date: '2024-Q3',
          value: 138.45,
          active_status: 'TRACK',
          quarterly_performance: [
            {
              quarter: 'Q3 2024',
              date: 'Oct 29, 2024',
              price: 138.45,
              eps: 1.55,
              eps_growth: 9.8,
              price_growth: 6.2,
              is_estimated: false
            }
          ],
          next_quarter_estimate: {
            quarter: '2025-Q1',
            estimated_eps: 1.68,
            announcement_date: 'Est. Feb 4, 2025',
            announcement_timestamp: Date.now() + (50 * 24 * 60 * 60 * 1000),
            days_until_announcement: 50,
            confidence: 'Medium'
          }
        }
      ]
      
      return {
        success: true,
        rankings: mockRankings,
        pagination: {
          page: filters.page || 1,
          limit: filters.limit || 10,
          total: mockRankings.length,
          totalPages: 1,
          hasNext: false,
          hasPrev: false
        },
        metadata: {
          available_countries: ['United States', 'Canada', 'United Kingdom'],
          available_sectors: ['Technology', 'Healthcare', 'Financial Services'],
          request_timestamp: new Date().toISOString(),
          data_source: 'mock-development-data'
        },
        message: 'Showing mock data for development (authentication required for live data)'
      }
    }
    
    // Transform the response to match our expected format
    if (result && typeof result === 'object') {
      // If the result has the expected structure, return it
      if (result.rankings || result.data) {
        const transformedResult: ServerAnalyticsResponse = {
          success: true,
          rankings: result.rankings || result.data || [],
          pagination: result.pagination,
          metadata: result.metadata,
          message: result.message,
          processing_time_ms: result.processing_time_ms
        }
        
        
        return transformedResult
      }
      
      // If it's an array, wrap it in our expected structure
      if (Array.isArray(result)) {
        return {
          success: true,
          rankings: result,
          pagination: {
            page: filters.page || 1,
            limit: filters.limit || 10,
            total: result.length,
            totalPages: 1,
            hasNext: false,
            hasPrev: false
          }
        }
      }
    }
    
    console.warn('⚠️ Unexpected analytics result format:', result)
    return {
      success: false,
      rankings: [],
      message: 'Unexpected response format from analytics API'
    }
    
  } catch (error) {
    const errorDetails = {
      message: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
      type: error instanceof Error ? error.constructor.name : typeof error,
      status: error && typeof error === 'object' && 'status' in error ? error.status : undefined,
      code: error && typeof error === 'object' && 'code' in error ? error.code : undefined,
      cause: error instanceof Error && error.cause ? error.cause : undefined,
      filters,
      endpoint: `/api/v1/analytics/rankings`,
      timestamp: new Date().toISOString()
    }
    
    console.error('💥 Failed to fetch server analytics:', errorDetails)
    
    return {
      success: false,
      rankings: [],
      message: `Failed to fetch analytics data: ${errorDetails.message}`,
      pagination: {
        page: filters.page || 1,
        limit: filters.limit || 10,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false
      }
    }
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Server-side filter options fetching
 * Get available countries and sectors for filtering
 */
export async function getServerFilterOptions(): Promise<FilterOptions> {
  try {
    
    const result = await serverFetcher('/api/v1/public/analytics/filters')
    
    if (result === null) {
      return {
        countries: [],
        sectors: [],
        exchanges: [],
        stock_types: []
      }
    }
    
    if (result && typeof result === 'object') {
      return {
        countries: result.countries || [],
        sectors: result.sectors || [],
        exchanges: result.exchanges || [],
        stock_types: result.stock_types || []
      }
    }
    
    console.warn('⚠️ Unexpected filter options format:', result)
    return {
      countries: [],
      sectors: [],
      exchanges: [],
      stock_types: []
    }
    
  } catch (error) {
    // Log the original error first
    console.error('💥 Failed to fetch server filter options - Original Error:', error)
    
    // Create detailed error info
    const errorDetails = {
      message: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined,
      type: error instanceof Error ? error.constructor.name : typeof error,
      status: error && typeof error === 'object' && 'status' in error ? error.status : undefined,
      code: error && typeof error === 'object' && 'code' in error ? error.code : undefined,
      cause: error instanceof Error && error.cause ? error.cause : undefined,
      endpoint: '/api/v1/analytics/filters',
      timestamp: new Date().toISOString(),
      errorString: JSON.stringify(error, Object.getOwnPropertyNames(error)),
      errorKeys: error && typeof error === 'object' ? Object.keys(error) : [],
      isError: error instanceof Error,
      valueType: typeof error
    }
    
    console.error('💥 Failed to fetch server filter options - Details:', errorDetails)
    
    return {
      countries: [],
      sectors: [],
      exchanges: [],
      stock_types: []
    }
  }
}

/**
 * Create an alias for the main analytics function for easier imports
 */
export const getAnalyticsData = getServerAnalytics