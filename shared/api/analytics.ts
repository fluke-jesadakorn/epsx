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

import { API_ROUTES } from '../config/route-constants';
import type { CardDashboardResponse } from '../types/analytics';
import type { UnifiedApiClient } from '../utils/api-client';
import { createAdminApiClient, createFrontendApiClient } from '../utils/api-client';

const API_VERSION = 'v1';
const UNKNOWN_ERR = 'Unknown error';

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
  quarterly_data?: QuarterlyEPSData[];
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
  constructor(private client: UnifiedApiClient) { }

  // ============================================================================
  // PUBLIC ANALYTICS (No Authentication Required)
  // ============================================================================

  /**
   * Get public analytics rankings with limited data access
   * Route: GET /api/public/analytics/rankings
   */
  async getPublicRankings(filters: AnalyticsFilters = {}): Promise<CardDashboardResponse> {
    // Apply public API limits
    const publicFilters = {
      ...filters,
      limit: Math.min(filters.limit ?? 10, 10), // Public API limit: max 10
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

    const response = await this.client.get<CardDashboardResponse>(
      API_ROUTES.ANALYTICS.PUBLIC_RANKINGS,
      publicFilters,
      {
        timeout: 15000,
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'public',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch public rankings: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    // The API client normalizes responses - response IS the full backend response
    // with { success, data, pagination, ... }
    return response as unknown as CardDashboardResponse;
  }

  /**
   * Get public analytics filters with basic options only
   * Route: GET /api/public/analytics/filters
   */
  async getPublicFilters(): Promise<AnalyticsFiltersResponse> {
    const response = await this.client.get<AnalyticsFiltersResponse>(
      API_ROUTES.ANALYTICS.PUBLIC_FILTERS,
      undefined,
      {
        timeout: 30000, // 30 second timeout for filter queries
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'public',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch public filters: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    // The API client normalizes responses - response IS the full backend response
    return response as unknown as AnalyticsFiltersResponse;
  }

  // ============================================================================
  // AUTHENTICATED ANALYTICS (User Authentication Required)
  // ============================================================================

  /**
   * Get authenticated analytics rankings with full data access
   * Route: GET /api/analytics/rankings
   */
  async getAuthenticatedRankings(filters: AnalyticsFilters = {}): Promise<CardDashboardResponse> {
    const response = await this.client.get<CardDashboardResponse>(
      API_ROUTES.ANALYTICS.RANKINGS,
      filters,
      {
        timeout: 30000,
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch authenticated rankings: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    // The API client normalizes responses - response IS the full backend response
    return response as unknown as CardDashboardResponse;
  }

  /**
   * Get authenticated analytics filters with advanced options
   * Route: GET /api/analytics/filters
   */
  async getAuthenticatedFilters(): Promise<AnalyticsFiltersResponse> {
    const response = await this.client.get<AnalyticsFiltersResponse>(
      API_ROUTES.ANALYTICS.FILTERS,
      undefined,
      {
        timeout: 30000, // 30 second timeout for filter queries
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch authenticated filters: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    // The API client normalizes responses - response IS the full backend response
    return response as unknown as AnalyticsFiltersResponse;
  }

  /**
   * Export analytics data for authenticated users
   * Route: POST /api/auth/analytics/export
   */
  async exportAnalyticsData(exportRequest: ExportRequest): Promise<ExportResponse> {
    const response = await this.client.post<ExportResponse>(
      API_ROUTES.ANALYTICS.EXPORT,
      exportRequest,
      {
        timeout: 90000, // 90 second timeout for export operations
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to export analytics data: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    return response.data;
  }

  /**
   * Get export status
   * Route: GET /api/auth/analytics/exports/{exportId}
   */
  async getExportStatus(exportId: string): Promise<ExportResponse> {
    const response = await this.client.get<ExportResponse>(
      `/api/auth/analytics/exports/${exportId}`,
      undefined,
      {
        timeout: 15000, // 15 second timeout for status checks
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get export status: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    return response.data;
  }

  // ============================================================================
  // ADMIN ANALYTICS (Admin Privileges Required)
  // ============================================================================

  /**
   * Get admin analytics overview
   * Route: GET /api/admin/analytics/overview
   */
  async getAdminOverview(): Promise<{
    total_users: number;
    active_users: number;
    revenue: number;
    growth_rate: number;
  }> {
    const response = await this.client.get<{
      data: {
        total_users: number;
        active_users: number;
        revenue: number;
        growth_rate: number;
      };
    }>(
      API_ROUTES.ADMIN.ANALYTICS_OVERVIEW,
      undefined,
      {
        timeout: 30000, // 30 second timeout for admin queries
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin overview: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    return response.data.data;
  }

  /**
   * Get admin user analytics
   * Route: GET /api/admin/analytics/users
   */
  async getAdminUserAnalytics(): Promise<{
    user_metrics: {
      total: number;
      active: number;
      new_today: number;
    };
  }> {
    const response = await this.client.get<{
      data: {
        user_metrics: {
          total: number;
          active: number;
          new_today: number;
        };
      };
    }>(
      API_ROUTES.ADMIN.ANALYTICS_USERS,
      undefined,
      {
        timeout: 30000, // 30 second timeout for admin queries
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin user analytics: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    return response.data.data;
  }

  /**
   * Get admin permission analytics
   * Route: GET /api/admin/analytics/permissions
   */
  async getAdminPermissionAnalytics(): Promise<{
    permission_distribution: Record<string, number>;
    group_membership: Array<{ group: string; count: number }>;
  }> {
    const response = await this.client.get<{
      data: {
        permission_distribution: Record<string, number>;
        group_membership: Array<{ group: string; count: number }>;
      };
    }>(
      API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS,
      undefined,
      {
        timeout: 30000, // 30 second timeout for admin queries
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin permission analytics: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    return response.data.data;
  }

  /**
   * Get admin revenue analytics
   * Route: GET /api/admin/analytics/revenue
   */
  async getAdminRevenueAnalytics(): Promise<{
    total_revenue: number;
    monthly_revenue: number;
    revenue_trends: Array<{ month: string; revenue: number }>;
  }> {
    const response = await this.client.get<{
      data: {
        total_revenue: number;
        monthly_revenue: number;
        revenue_trends: Array<{ month: string; revenue: number }>;
      };
    }>(
      API_ROUTES.ADMIN.ANALYTICS_REVENUE,
      undefined,
      {
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin revenue analytics: ${response.error?.message ?? UNKNOWN_ERR}`);
    }

    return response.data.data;
  }

  /**
   * Get admin performance analytics
   * Route: GET /api/admin/analytics/performance
   */
  async getAdminPerformanceAnalytics(): Promise<{
    api_response_times: Record<string, number>;
    cache_hit_rates: Record<string, number>;
    error_rates: Record<string, number>;
  }> {
    const response = await this.client.get<{
      data: {
        api_response_times: Record<string, number>;
        cache_hit_rates: Record<string, number>;
        error_rates: Record<string, number>;
      };
    }>(
      API_ROUTES.ADMIN.ANALYTICS_PERFORMANCE,
      undefined,
      {
        headers: {
          'X-API-Version': API_VERSION,
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to fetch admin performance analytics: ${response.error?.message ?? UNKNOWN_ERR}`);
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
    if (country !== '' && country === country.toLowerCase()) {
      return country;
    }

    // Check direct mapping
    if (countryValueMap[country]) {
      return countryValueMap[country];
    }

    // Default to lowercase if no mapping found
    return country.toLowerCase();
  }

  /**
   * Build analytics URL with filters
   */
  static buildAnalyticsUrl(baseUrl: string, filters: AnalyticsFilters): string {
    const params = new URLSearchParams();

    // Mapping of field names to their extraction logic
    const directFields: (keyof AnalyticsFilters)[] = ['page', 'limit', 'sort_by', 'sector', 'exchange', 'stock_type'];
    directFields.forEach(field => {
      const value = filters[field];
      if (value !== undefined) {
        params.append(field, value.toString());
      }
    });

    if (filters.country !== undefined && filters.country !== '') {
      params.append('country', this.normalizeCountryName(filters.country));
    }

    // Range filters
    const rangeFields: (keyof AnalyticsFilters)[] = [
      'min_eps', 'max_eps', 'min_growth', 'max_growth',
      'min_market_cap', 'max_market_cap', 'min_volume', 'max_volume',
      'min_price', 'max_price', 'min_pe_ratio', 'max_pe_ratio',
      'min_dividend_yield', 'max_dividend_yield'
    ];

    rangeFields.forEach(field => {
      const value = filters[field];
      if (value !== undefined) {
        params.append(field, value.toString());
      }
    });

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
    return new AnalyticsAPIClient(createAdminApiClient());
  } else {
    return new AnalyticsAPIClient(createFrontendApiClient());
  }
}

// Type alias for backward compatibility with useApiClient
export type AnalyticsApi = AnalyticsAPIClient;