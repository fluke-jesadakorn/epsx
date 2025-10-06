/**
 * UNIFIED SERVER DATA UTILITIES
 * 
 * Replaces proxy-based server data fetching with direct unified client calls.
 * Eliminates the need for /api routes by using shared API clients directly.
 * 
 * Benefits:
 * - No proxy routes needed
 * - Direct backend communication
 * - Type-safe responses
 * - Consistent error handling
 * - Reduced latency (eliminates middleware hop)
 */

import { createFrontendApiClient } from '../../../shared/utils/api-client';
import { 
  createAnalyticsClient, 
  AnalyticsAPIClient,
  AnalyticsFilters 
} from '../../../shared/api/analytics';
import { 
  createAuthClient,
  AuthAPIClient 
} from '../../../shared/api/auth';
import { getOIDCAccessTokenFromCookies } from '@/lib/server/jwt';

// ============================================================================
// TYPES (for compatibility with existing frontend)
// ============================================================================

export interface EPSQueryParams {
  page: number;
  limit: number;
  country?: string;
  sector?: string;
  sort_by?: string;
  min_eps?: number;
  min_growth?: number;
  search?: string;
}

export interface QuarterlyPerformanceData {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
  announcement_date?: string;
  announcement_timestamp?: number;
  is_estimated?: boolean;
}

export interface NextQuarterEstimate {
  quarter: string;
  estimated_eps: number;
  announcement_date: string;
  announcement_timestamp: number;
  days_until_announcement: number;
  estimated_price_target?: number;
  confidence: string;
}

export interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: QuarterlyPerformanceData[];
  next_quarter_estimate?: NextQuarterEstimate;
  currency?: string;
}

export interface ServerAnalyticsResponse {
  success: boolean;
  rankings: SymbolCardData[];
  pagination?: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  metadata?: {
    available_countries: string[];
    available_sectors: string[];
    request_timestamp: string;
    data_source: string;
  };
  message?: string;
  processing_time_ms?: number;
}

export interface FilterOptions {
  countries: string[];
  sectors: string[];
  exchanges?: string[];
  stock_types?: string[];
}

// ============================================================================
// UNIFIED CLIENT FACTORIES
// ============================================================================

/**
 * Create analytics client for server-side usage
 */
async function createServerAnalyticsClient(): Promise<AnalyticsAPIClient> {
  const token = await getOIDCAccessTokenFromCookies();
  const apiClient = createFrontendApiClient({
    token: token ?? undefined,
    serverSide: true
  });
  return createAnalyticsClient(apiClient);
}

/**
 * Create auth client for server-side usage
 */
async function createServerAuthClient(): Promise<AuthAPIClient> {
  const token = await getOIDCAccessTokenFromCookies();
  const apiClient = createFrontendApiClient({
    token: token ?? undefined,
    serverSide: true
  });
  return createAuthClient(apiClient);
}

// ============================================================================
// UNIFIED SERVER DATA FUNCTIONS
// ============================================================================

/**
 * Server-side analytics data fetching using unified client
 * Replaces: /api/v1/analytics/rankings and /api/v1/public/analytics/rankings proxy routes
 */
export async function getServerAnalytics(filters: EPSQueryParams): Promise<ServerAnalyticsResponse> {
  try {
    const analyticsClient = await createServerAnalyticsClient();
    
    // Convert EPSQueryParams to AnalyticsFilters format
    const analyticsFilters: AnalyticsFilters = {
      page: filters.page,
      limit: filters.limit,
      country: filters.country,
      sector: filters.sector,
      sort_by: filters.sort_by as any,
      min_eps: filters.min_eps,
      min_growth: filters.min_growth,
    };

    let result;
    try {
      // Try authenticated endpoint first
      result = await analyticsClient.getAuthenticatedRankings(analyticsFilters);
    } catch (error) {
      // If authentication failed, try public endpoint
      console.log('Authenticated analytics failed, trying public endpoint');
      result = await analyticsClient.getPublicRankings(analyticsFilters);
    }

    // Transform unified client response to legacy format for compatibility
    // Handle cases where result.data might be undefined or not an array
    const rankingsData = result?.data || [];
    if (!Array.isArray(rankingsData)) {
      console.warn('Expected array for result.data, got:', typeof rankingsData);
      throw new Error('Invalid rankings data format received from API');
    }

    const transformedRankings: SymbolCardData[] = rankingsData.map((ranking, index) => ({
      rank: ranking?.ranking_position || index + 1,
      symbol: ranking?.symbol || '',
      latest_date: ranking?.quarterly_data?.[0]?.date || new Date().toISOString(),
      value: ranking?.price_current || 0,
      active_status: ranking?.active_status || 'unknown',
      quarterly_performance: (ranking?.quarterly_data || []).map(q => ({
        quarter: q?.quarter || '',
        date: q?.date || '',
        price: q?.price || 0,
        eps: q?.eps || 0,
        eps_growth: q?.eps_growth || 0,
        price_growth: q?.price_growth || 0,
      })),
      next_quarter_estimate: ranking?.next_quarter_estimate ? {
        quarter: ranking.next_quarter_estimate.quarter || '',
        estimated_eps: ranking.next_quarter_estimate.estimated_eps || 0,
        announcement_date: ranking.next_quarter_estimate.announcement_date || '',
        announcement_timestamp: ranking.next_quarter_estimate.announcement_timestamp || 0,
        days_until_announcement: ranking.next_quarter_estimate.days_until_announcement || 0,
        confidence: ranking.next_quarter_estimate.confidence || 'Medium'
      } : undefined,
      currency: 'USD'
    }));

    return {
      success: true,
      rankings: transformedRankings,
      pagination: result.pagination ? {
        page: result.pagination.page,
        limit: result.pagination.limit,
        total: result.pagination.total,
        totalPages: result.pagination.totalPages,
        hasNext: result.pagination.hasNext,
        hasPrev: result.pagination.hasPrev,
      } : undefined,
      metadata: {
        available_countries: [],
        available_sectors: [],
        request_timestamp: new Date().toISOString(),
        data_source: 'unified-analytics-client'
      },
      message: result.notice
    };

  } catch (error) {
    console.error('💥 Failed to fetch analytics via unified client:', error);
    
    return {
      success: false,
      rankings: [],
      message: `Failed to fetch analytics data: ${error instanceof Error ? error.message : 'Unknown error'}`,
      pagination: {
        page: filters.page || 1,
        limit: filters.limit || 10,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false
      }
    };
  }
}

/**
 * Server-side filter options fetching using unified client
 * Replaces: /api/v1/analytics/filters proxy route
 */
export async function getServerFilterOptions(): Promise<FilterOptions> {
  try {
    const analyticsClient = await createServerAnalyticsClient();
    
    let result;
    try {
      // Try authenticated filters first
      result = await analyticsClient.getAuthenticatedFilters();
    } catch (error) {
      // If authentication failed, try public filters
      console.log('Authenticated filters failed, trying public endpoint');
      result = await analyticsClient.getPublicFilters();
    }

    // Handle cases where result.data might be undefined
    const filterData = result?.data || {};
    
    return {
      countries: filterData.countries || [],
      sectors: filterData.sectors || [],
      exchanges: filterData.exchanges || [],
      stock_types: []
    };

  } catch (error) {
    console.error('💥 Failed to fetch filter options via unified client:', error);
    
    return {
      countries: [],
      sectors: [],
      exchanges: [],
      stock_types: []
    };
  }
}

/**
 * Server-side portfolio data fetching using unified client
 * Replaces: /api/v1/portfolio/rankings proxy route
 */
export async function getServerPortfolio(filters: EPSQueryParams): Promise<ServerAnalyticsResponse> {
  try {
    const analyticsClient = await createServerAnalyticsClient();
    
    // Convert EPSQueryParams to AnalyticsFilters format with positive growth filter
    const analyticsFilters: AnalyticsFilters = {
      page: filters.page,
      limit: filters.limit,
      country: filters.country,
      sector: filters.sector,
      sort_by: filters.sort_by as any,
      min_eps: filters.min_eps,
      min_growth: Math.max(filters.min_growth || 0, 0), // Ensure positive growth
    };

    let result;
    try {
      // Try authenticated endpoint first (portfolios typically need authentication)
      result = await analyticsClient.getAuthenticatedRankings(analyticsFilters);
    } catch (error) {
      // Fallback to public with positive growth filter
      console.log('Authenticated portfolio failed, trying public endpoint');
      result = await analyticsClient.getPublicRankings(analyticsFilters);
    }

    // Handle cases where result.data might be undefined or not an array
    const portfolioData = result?.data || [];
    if (!Array.isArray(portfolioData)) {
      console.warn('Expected array for portfolio result.data, got:', typeof portfolioData);
      throw new Error('Invalid portfolio data format received from API');
    }

    // Filter for positive growth only
    const positiveGrowthRankings = portfolioData.filter(ranking => 
      (ranking?.growth_factor || 0) > 0
    );

    // Transform unified client response to legacy format for compatibility
    const transformedRankings: SymbolCardData[] = positiveGrowthRankings.map((ranking, index) => ({
      rank: ranking?.ranking_position || index + 1,
      symbol: ranking?.symbol || '',
      latest_date: ranking?.quarterly_data?.[0]?.date || new Date().toISOString(),
      value: ranking?.price_current || 0,
      active_status: ranking?.active_status || 'unknown',
      quarterly_performance: (ranking?.quarterly_data || []).map(q => ({
        quarter: q?.quarter || '',
        date: q?.date || '',
        price: q?.price || 0,
        eps: q?.eps || 0,
        eps_growth: q?.eps_growth || 0,
        price_growth: q?.price_growth || 0,
      })),
      next_quarter_estimate: ranking?.next_quarter_estimate ? {
        quarter: ranking.next_quarter_estimate.quarter || '',
        estimated_eps: ranking.next_quarter_estimate.estimated_eps || 0,
        announcement_date: ranking.next_quarter_estimate.announcement_date || '',
        announcement_timestamp: ranking.next_quarter_estimate.announcement_timestamp || 0,
        days_until_announcement: ranking.next_quarter_estimate.days_until_announcement || 0,
        confidence: ranking.next_quarter_estimate.confidence || 'Medium'
      } : undefined,
      currency: 'USD'
    }));

    return {
      success: true,
      rankings: transformedRankings,
      pagination: result.pagination ? {
        page: result.pagination.page,
        limit: result.pagination.limit,
        total: positiveGrowthRankings.length,
        totalPages: Math.ceil(positiveGrowthRankings.length / result.pagination.limit),
        hasNext: result.pagination.hasNext,
        hasPrev: result.pagination.hasPrev,
      } : undefined,
      metadata: {
        available_countries: [],
        available_sectors: [],
        request_timestamp: new Date().toISOString(),
        data_source: 'unified-analytics-client-portfolio'
      },
      message: 'Positive growth portfolio data'
    };

  } catch (error) {
    console.error('💥 Failed to fetch portfolio via unified client:', error);
    
    return {
      success: false,
      rankings: [],
      message: `Failed to fetch portfolio data: ${error instanceof Error ? error.message : 'Unknown error'}`,
      pagination: {
        page: filters.page || 1,
        limit: filters.limit || 10,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false
      }
    };
  }
}


// ============================================================================
// COMPATIBILITY ALIASES
// ============================================================================

/**
 * Create aliases for the main functions for easier imports
 */
export const getAnalyticsData = getServerAnalytics;
export const getPortfolioData = getServerPortfolio;

