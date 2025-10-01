/**
 * UNIFIED ANALYTICS API CLIENT
 * 
 * Consolidates all analytics-related API calls across EPSX applications.
 * Eliminates proxy routes by providing direct backend communication.
 * 
 * Features:
 * - Public analytics (no auth required, limited data)
 * - Authenticated analytics (full data for users)
 * - Admin analytics (full management access)
 * - Consistent filtering and pagination
 * - Type-safe responses with proper error handling
 */

import { UnifiedApiClient, ApiResponse, PaginatedResponse } from '../utils/api-client';

// ============================================================================
// ANALYTICS TYPES
// ============================================================================

export interface EPSRanking {
  symbol: string;
  name: string;
  country: string;
  sector: string;
  exchange: string;
  current_eps: number | null;
  growth_factor: number | null;
  price_current: number | null;
  market_cap: number | null;
  volume: number | null;
  ranking_position: number;
  active_status: string;
  quarterly_data: QuarterlyEPSData[];
}

export interface QuarterlyEPSData {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
  volume: number;
}

export interface AnalyticsFilters {
  page?: number;
  limit?: number;
  sort_by?: 'eps_growth' | 'market_cap' | 'volume' | 'price';
  country?: string;
  sector?: string;
  min_eps?: number;
  max_eps?: number;
  min_growth?: number;
  max_growth?: number;
  min_market_cap?: number;
  max_market_cap?: number;
  min_volume?: number;
  max_volume?: number;
  min_price?: number;
  max_price?: number;
  min_pe_ratio?: number;
  max_pe_ratio?: number;
  min_dividend_yield?: number;
  max_dividend_yield?: number;
  exchange?: string;
  stock_type?: string;
}

export interface EPSRankingsResponse {
  data: EPSRanking[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  api_version?: string;
  access_level?: string;
  notice?: string;
  features?: {
    export_available?: boolean;
    real_time_updates?: boolean;
    advanced_filtering?: boolean;
    historical_data?: boolean;
  };
}

export interface AnalyticsFiltersResponse {
  success: boolean;
  data: {
    countries: string[];
    sectors: string[];
    exchanges?: string[];
    sort_options: string[];
  };
  api_version?: string;
  access_level?: string;
  notice?: string;
  features?: {
    advanced_filtering?: boolean;
    financial_metrics?: boolean;
    custom_filters?: boolean;
    filter_presets?: boolean;
  };
}

export interface ExportRequest {
  format: 'csv' | 'xlsx' | 'json';
  filters?: AnalyticsFilters;
  columns?: string[];
}

export interface ExportResponse {
  success: boolean;
  export_id: string;
  download_url: string;
  status: 'processing' | 'completed' | 'failed';
  api_version?: string;
  access_level?: string;
}

// ============================================================================
// ANALYTICS API CLIENT CLASS
// ============================================================================

export class AnalyticsAPIClient {
  constructor(private client: UnifiedApiClient) {}

  // ============================================================================
  // PUBLIC ANALYTICS (No Authentication Required)
  // ============================================================================

  /**
   * Get public analytics rankings with limited data access
   * Route: GET /api/v1/public/analytics/rankings
   */
  async getPublicRankings(filters: AnalyticsFilters = {}): Promise<EPSRankingsResponse> {
    // Apply public API limits
    const publicFilters = {
      ...filters,
      limit: Math.min(filters.limit || 10, 10), // Public API limit: max 10
      // Remove financial filters for public access
      min_eps: undefined,
      max_eps: undefined,
      min_growth: undefined,
      max_growth: undefined,
      min_market_cap: undefined,
      max_market_cap: undefined,
      min_volume: undefined,
      max_volume: undefined,
      min_price: undefined,
      max_price: undefined,
      min_pe_ratio: undefined,
      max_pe_ratio: undefined,
      min_dividend_yield: undefined,
      max_dividend_yield: undefined,
    };

    const response = await this.client.get<EPSRankingsResponse>(
      '/api/v1/public/analytics/rankings',
      publicFilters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'public',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch public rankings: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get public analytics filters with basic options only
   * Route: GET /api/v1/public/analytics/filters
   */
  async getPublicFilters(): Promise<AnalyticsFiltersResponse> {
    const response = await this.client.get<AnalyticsFiltersResponse>(
      '/api/v1/public/analytics/filters',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'public',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch public filters: ${response.error}`);
    }

    return response.data;
  }

  // ============================================================================
  // AUTHENTICATED ANALYTICS (User Authentication Required)
  // ============================================================================

  /**
   * Get authenticated analytics rankings with full data access
   * Route: GET /api/v1/analytics/rankings
   */
  async getAuthenticatedRankings(filters: AnalyticsFilters = {}): Promise<EPSRankingsResponse> {
    const response = await this.client.get<EPSRankingsResponse>(
      '/api/v1/analytics/rankings',
      filters,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch authenticated rankings: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get authenticated analytics filters with advanced options
   * Route: GET /api/v1/analytics/filters
   */
  async getAuthenticatedFilters(): Promise<AnalyticsFiltersResponse> {
    const response = await this.client.get<AnalyticsFiltersResponse>(
      '/api/v1/analytics/filters',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch authenticated filters: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Export analytics data for authenticated users
   * Route: POST /api/v1/auth/analytics/export
   */
  async exportAnalyticsData(exportRequest: ExportRequest): Promise<ExportResponse> {
    const response = await this.client.post<ExportResponse>(
      '/api/v1/auth/analytics/export',
      exportRequest,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to export analytics data: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get export status
   * Route: GET /api/v1/auth/analytics/exports/{exportId}
   */
  async getExportStatus(exportId: string): Promise<ExportResponse> {
    const response = await this.client.get<ExportResponse>(
      `/api/v1/auth/analytics/exports/${exportId}`,
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get export status: ${response.error}`);
    }

    return response.data;
  }

  // ============================================================================
  // ADMIN ANALYTICS (Admin Privileges Required)
  // ============================================================================

  /**
   * Get admin analytics overview
   * Route: GET /api/v1/admin/analytics/overview
   */
  async getAdminOverview(): Promise<{
    total_users: number;
    active_users: number;
    revenue: number;
    growth_rate: number;
  }> {
    const response = await this.client.get(
      '/api/v1/admin/analytics/overview',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin overview: ${response.error}`);
    }

    return response.data.data;
  }

  /**
   * Get admin user analytics
   * Route: GET /api/v1/admin/analytics/users
   */
  async getAdminUserAnalytics(): Promise<{
    user_metrics: {
      total: number;
      active: number;
      new_today: number;
    };
  }> {
    const response = await this.client.get(
      '/api/v1/admin/analytics/users',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin user analytics: ${response.error}`);
    }

    return response.data.data;
  }

  /**
   * Get admin permission analytics
   * Route: GET /api/v1/admin/analytics/permissions
   */
  async getAdminPermissionAnalytics(): Promise<{
    permission_distribution: Record<string, number>;
    group_membership: Array<{ group: string; count: number }>;
  }> {
    const response = await this.client.get(
      '/api/v1/admin/analytics/permissions',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin permission analytics: ${response.error}`);
    }

    return response.data.data;
  }

  /**
   * Get admin revenue analytics
   * Route: GET /api/v1/admin/analytics/revenue
   */
  async getAdminRevenueAnalytics(): Promise<{
    total_revenue: number;
    monthly_revenue: number;
    revenue_trends: Array<{ month: string; revenue: number }>;
  }> {
    const response = await this.client.get(
      '/api/v1/admin/analytics/revenue',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin revenue analytics: ${response.error}`);
    }

    return response.data.data;
  }

  /**
   * Get admin performance analytics
   * Route: GET /api/v1/admin/analytics/performance
   */
  async getAdminPerformanceAnalytics(): Promise<{
    api_response_times: Record<string, number>;
    cache_hit_rates: Record<string, number>;
    error_rates: Record<string, number>;
  }> {
    const response = await this.client.get(
      '/api/v1/admin/analytics/performance',
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin performance analytics: ${response.error}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  /**
   * Normalize country names for API consistency
   */
  static normalizeCountryName(country: string): string {
    const countryValueMap: Record<string, string> = {
      'United States': 'america',
      'United Kingdom': 'uk',
      'UK': 'uk',
      'South Korea': 'korea',
      'South Africa': 'rsa',
      'Saudi Arabia': 'ksa',
      'United Arab Emirates': 'uae',
      'New Zealand': 'newzealand',
      'Hong Kong': 'hongkong',
      'Czech Republic': 'czech',
      'Sri Lanka': 'srilanka',
    };

    // First check if it's already a mapped value (lowercase)
    if (country && country === country.toLowerCase()) {
      return country;
    }
    
    // Check direct mapping
    if (countryValueMap[country]) {
      return countryValueMap[country];
    }
    
    // Default to lowercase if no mapping found
    return country?.toLowerCase() || '';
  }

  /**
   * Build analytics URL with filters
   */
  static buildAnalyticsUrl(baseUrl: string, filters: AnalyticsFilters): string {
    const params = new URLSearchParams();
    
    if (filters.page) params.append('page', filters.page.toString());
    if (filters.limit) params.append('limit', filters.limit.toString());
    if (filters.sort_by) params.append('sort_by', filters.sort_by);
    if (filters.country) params.append('country', this.normalizeCountryName(filters.country));
    if (filters.sector) params.append('sector', filters.sector);
    if (filters.exchange) params.append('exchange', filters.exchange);
    if (filters.stock_type) params.append('stock_type', filters.stock_type);
    
    // Financial filters (only for authenticated APIs)
    if (filters.min_eps) params.append('min_eps', filters.min_eps.toString());
    if (filters.max_eps) params.append('max_eps', filters.max_eps.toString());
    if (filters.min_growth) params.append('min_growth', filters.min_growth.toString());
    if (filters.max_growth) params.append('max_growth', filters.max_growth.toString());
    if (filters.min_market_cap) params.append('min_market_cap', filters.min_market_cap.toString());
    if (filters.max_market_cap) params.append('max_market_cap', filters.max_market_cap.toString());
    if (filters.min_volume) params.append('min_volume', filters.min_volume.toString());
    if (filters.max_volume) params.append('max_volume', filters.max_volume.toString());
    if (filters.min_price) params.append('min_price', filters.min_price.toString());
    if (filters.max_price) params.append('max_price', filters.max_price.toString());
    if (filters.min_pe_ratio) params.append('min_pe_ratio', filters.min_pe_ratio.toString());
    if (filters.max_pe_ratio) params.append('max_pe_ratio', filters.max_pe_ratio.toString());
    if (filters.min_dividend_yield) params.append('min_dividend_yield', filters.min_dividend_yield.toString());
    if (filters.max_dividend_yield) params.append('max_dividend_yield', filters.max_dividend_yield.toString());

    const queryString = params.toString();
    return queryString ? `${baseUrl}?${queryString}` : baseUrl;
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create analytics API client for frontend applications
 */
export function createAnalyticsClient(client: UnifiedApiClient): AnalyticsAPIClient {
  return new AnalyticsAPIClient(client);
}

/**
 * Create analytics client with automatic platform detection
 */
export function createPlatformAnalyticsClient(platform: 'frontend' | 'admin' = 'frontend'): AnalyticsAPIClient {
  if (platform === 'admin') {
    const { createAdminApiClient } = require('../utils/api-client');
    return new AnalyticsAPIClient(createAdminApiClient());
  } else {
    const { createFrontendApiClient } = require('../utils/api-client');
    return new AnalyticsAPIClient(createFrontendApiClient());
  }
}