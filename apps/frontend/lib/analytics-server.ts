// import { cache } from 'react'; // DISABLED: Removing cache to show real TradingView data
import { getOIDCAccessTokenFromCookies } from '@/lib/server/jwt';
import { getBackendUrl } from '../../../shared/utils/url-resolver';

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
  is_estimated?: boolean;
}

export interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: QuarterlyPerformanceData[];
  next_quarter_estimate?: {
    quarter: string;
    announcement_date: string;
    days_until_announcement: number;
    estimated_eps: number;
    estimated_price_target?: number;
    confidence: string;
  };
}

export interface CardDashboardResponse {
  success: boolean;
  data: SymbolCardData[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  metadata: {
    available_countries: string[];
    available_sectors: string[];
    request_timestamp: string;
    data_source: string;
  };
  message?: string;
  processing_time_ms: number;
}

export interface FilterOptions {
  countries: Array<{ value: string; label: string }>;
  sectors: string[];
}

// NO CACHE: Direct data fetching function for real TradingView data
export const getAnalyticsData = async (params: EPSQueryParams): Promise<CardDashboardResponse> => {
  const baseURL = getBackendUrl('server');
  const queryString = new URLSearchParams();
  
  Object.keys(params).forEach(key => {
    const value = (params as any)[key];
    if (value !== undefined && value !== null) {
      queryString.append(key, String(value));
    }
  });

  const url = `${baseURL}/api/v1/public/analytics/rankings?${queryString.toString()}`;

  try {
    // Get access token for authenticated requests
    const accessToken = await getOIDCAccessTokenFromCookies();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    // Add authorization header if we have an access token
    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    const response = await fetch(url, {
      method: 'GET',
      headers,
      // NO CACHE: Force fresh TradingView data
      cache: 'no-store'
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Failed to fetch analytics data:', error);
    // Return empty data structure to prevent crashes
    return {
      success: false,
      data: [],
      pagination: {
        page: params.page,
        limit: params.limit,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false,
      },
      metadata: {
        available_countries: [],
        available_sectors: [],
        request_timestamp: new Date().toISOString(),
        data_source: 'server-error',
      },
      processing_time_ms: 0,
    };
  }
};

export const getFilterOptions = async (): Promise<FilterOptions> => {
  const baseURL = getBackendUrl('server');
  
  try {
    console.log('🔧 Fetching filter options from:', baseURL);

    // Simplified request without OIDC token for debugging
    const [countriesResponse, sectorsResponse] = await Promise.all([
      fetch(`${baseURL}/api/v1/public/analytics/countries`, {
        headers: { 'Content-Type': 'application/json' },
        cache: 'no-store' // NO CACHE: Force fresh data
      }),
      fetch(`${baseURL}/api/v1/public/analytics/sectors`, {
        headers: { 'Content-Type': 'application/json' },
        cache: 'no-store' // NO CACHE: Force fresh data
      })
    ]);

    if (!countriesResponse.ok || !sectorsResponse.ok) {
      console.error('❌ HTTP Error - Countries:', countriesResponse.status, 'Sectors:', sectorsResponse.status);
      throw new Error(`HTTP Error - Countries: ${countriesResponse.status}, Sectors: ${sectorsResponse.status}`);
    }

    console.log('✅ Both requests successful');
    const [countriesData, sectorsData] = await Promise.all([
      countriesResponse.json(),
      sectorsResponse.json()
    ]);

    console.log('✅ Data parsed - Countries:', countriesData.count, 'Sectors:', sectorsData.count);
    return {
      countries: countriesData.countries || [],
      sectors: sectorsData.sectors || [],
    };
  } catch (error) {
    console.error('❌ Failed to fetch filter options:', error);
    return {
      countries: [],
      sectors: [],
    };
  }
};