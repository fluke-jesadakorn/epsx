import { cache } from 'react';

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

// Cache the data fetching function
export const getAnalyticsData = cache(async (params: EPSQueryParams): Promise<CardDashboardResponse> => {
  const baseURL = process.env.BACKEND_URL || 'http://localhost:8080';
  const queryString = new URLSearchParams();
  
  Object.keys(params).forEach(key => {
    const value = (params as any)[key];
    if (value !== undefined && value !== null) {
      queryString.append(key, String(value));
    }
  });

  const url = `${baseURL}/api/v1/analytics/eps-rankings?${queryString.toString()}`;

  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      // Enable caching for 30 seconds
      next: { revalidate: 30 }
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
});

export const getFilterOptions = cache(async (): Promise<FilterOptions> => {
  const baseURL = process.env.BACKEND_URL || 'http://localhost:8080';
  
  try {
    const [countriesResponse, sectorsResponse] = await Promise.all([
      fetch(`${baseURL}/api/v1/analytics/eps-rankings/countries`, {
        next: { revalidate: 300 } // 5 minutes
      }),
      fetch(`${baseURL}/api/v1/analytics/eps-rankings/sectors`, {
        next: { revalidate: 300 } // 5 minutes
      })
    ]);

    if (!countriesResponse.ok || !sectorsResponse.ok) {
      throw new Error('Failed to fetch filter options');
    }

    const [countriesData, sectorsData] = await Promise.all([
      countriesResponse.json(),
      sectorsResponse.json()
    ]);

    return {
      countries: countriesData.countries || [],
      sectors: sectorsData.sectors || [],
    };
  } catch (error) {
    console.error('Failed to fetch filter options:', error);
    return {
      countries: [],
      sectors: [],
    };
  }
});