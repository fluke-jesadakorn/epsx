import type { StockFinancialData } from '@/types/financialChartData';

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
    startIndex: number;
    endIndex: number;
    currentPageSize: number;
  };
}

export interface CountResponse {
  count: number;
  timestamp: string;
  filters: {
    country: string;
    quarters: number;
  };
}

export interface StockApiParams {
  page?: number;
  limit?: number;
  country?: string;
  quarters?: number;
}

export class StockApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = '/api/stock') {
    this.baseUrl = baseUrl;
  }

  /**
   * Fetch paginated stock data from API
   */
  async getPaginatedStocks(params: StockApiParams = {}): Promise<PaginatedResponse<StockFinancialData>> {
    const searchParams = new URLSearchParams();
    
    if (params.page) searchParams.set('page', params.page.toString());
    if (params.limit) searchParams.set('limit', params.limit.toString());
    if (params.country) searchParams.set('country', params.country);
    if (params.quarters) searchParams.set('quarters', params.quarters.toString());

    const response = await fetch(`${this.baseUrl}/paginated?${searchParams.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to fetch paginated stocks');
    }

    return response.json();
  }

  /**
   * Fetch stock count from API
   */
  async getStockCount(params: Pick<StockApiParams, 'country' | 'quarters'> = {}): Promise<CountResponse> {
    const searchParams = new URLSearchParams();
    
    if (params.country) searchParams.set('country', params.country);
    if (params.quarters) searchParams.set('quarters', params.quarters.toString());

    const response = await fetch(`${this.baseUrl}/count?${searchParams.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to fetch stock count');
    }

    return response.json();
  }

  /**
   * Fetch stocks using the original API (backward compatibility)
   */
  async getStocks(params: StockApiParams & { skip?: number; paginated?: boolean } = {}): Promise<StockFinancialData[] | PaginatedResponse<StockFinancialData>> {
    const searchParams = new URLSearchParams();
    
    if (params.skip) searchParams.set('skip', params.skip.toString());
    if (params.limit) searchParams.set('limit', params.limit.toString());
    if (params.page) searchParams.set('page', params.page.toString());
    if (params.country) searchParams.set('country', params.country);
    if (params.quarters) searchParams.set('quarters', params.quarters.toString());
    if (params.paginated) searchParams.set('paginated', 'true');

    const response = await fetch(`${this.baseUrl}?${searchParams.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to fetch stocks');
    }

    return response.json();
  }
}

// Export a default instance
export const stockApiClient = new StockApiClient();

// Export individual functions for easier use
export const getPaginatedStocks = (params?: StockApiParams) => stockApiClient.getPaginatedStocks(params);
export const getStockCount = (params?: Pick<StockApiParams, 'country' | 'quarters'>) => stockApiClient.getStockCount(params);
export const getStocks = (params?: StockApiParams & { skip?: number; paginated?: boolean }) => stockApiClient.getStocks(params);
