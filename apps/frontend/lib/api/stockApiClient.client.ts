import { apiClient } from '@epsx/api-client';
import type { PaginatedResponse, CountResponse, StockFinancialData } from '@epsx/api-client';
import type { StockFinancialData as _LocalStockFinancialData } from '@/types/financialChartData';

export interface StockApiParams {
  page?: number;
  limit?: number;
  country?: string;
  quarters?: number;
}

export class StockApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    // Use absolute URL for server-side requests, relative for client-side
    if (baseUrl) {
      this.baseUrl = baseUrl;
    } else if (typeof window === 'undefined') {
      // Server-side: use absolute URL
      const frontendUrl = process.env.FRONTEND_URL || 'http://localhost:3000';
      this.baseUrl = `${frontendUrl}/api/v1/market-data/stocks`;
    } else {
      // Client-side: use relative URL
      this.baseUrl = '/api/v1/market-data/stocks';
    }
  }

  /**
   * Fetch paginated stock data from API
   */
  async getPaginatedStocks(params: StockApiParams = {}): Promise<PaginatedResponse<StockFinancialData>> {
    const result = await apiClient.getPaginatedStocks(params);
    
    if (result.error) {
      throw new Error(result.error);
    }
    
    return result.data!;
  }

  /**
   * Fetch stock count from API
   */
  async getStockCount(params: Pick<StockApiParams, 'country' | 'quarters'> = {}): Promise<CountResponse> {
    const result = await apiClient.getStockCount(params);
    
    if (result.error) {
      throw new Error(result.error);
    }
    
    return result.data!;
  }

  /**
   * Fetch stocks using the original API (backward compatibility)
   */
  async getStocks(params: StockApiParams & { skip?: number; paginated?: boolean } = {}): Promise<StockFinancialData[] | PaginatedResponse<StockFinancialData>> {
    const result = await apiClient.getStocks(params);
    
    if (result.error) {
      throw new Error(result.error);
    }
    
    return result.data!;
  }
}

// Export a default instance
export const stockApiClient = new StockApiClient();

// Export individual functions for easier use
export const getPaginatedStocks = (params?: StockApiParams) => stockApiClient.getPaginatedStocks(params);
export const getStockCount = (params?: Pick<StockApiParams, 'country' | 'quarters'>) => stockApiClient.getStockCount(params);
export const getStocks = (params?: StockApiParams & { skip?: number; paginated?: boolean }) => stockApiClient.getStocks(params);
