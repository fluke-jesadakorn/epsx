/**
 * Market Data API Client
 * Stock market data, financial data, and trading information
 */

import { apiLogger, safeError } from '@/lib/utils/logging';
import { UnifiedApiClient } from '@/shared/api';
import type { PaginatedResponse } from '@/shared/types/api';
import type { CountResponse, StockFinancialData } from './client';

// ============================================================================
// Types and Interfaces
// ============================================================================

export interface StockApiParams {
  page?: number;
  limit?: number;
  country?: string;
  quarters?: number;
  sector?: string;
  min_market_cap?: number;
  sort_by?: string;
  sort_order?: 'asc' | 'desc';
}

export interface MarketSummary {
  totalStocks: number;
  sectors: string[];
  countries: string[];
  lastUpdated: string;
  marketStatus: 'open' | 'closed' | 'pre_market' | 'after_hours';
}

export interface StockPrice {
  symbol: string;
  price: number;
  change: number;
  changePercent: number;
  volume: number;
  timestamp: string;
}

export interface StockChart {
  symbol: string;
  timeframe: '1d' | '5d' | '1m' | '3m' | '6m' | '1y' | '2y' | '5y';
  data: Array<{
    timestamp: string;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
  }>;
}

// ============================================================================
// Market Data API Client Class
// ============================================================================

export class MarketApiClient {
  private client: UnifiedApiClient;

  constructor(baseUrl?: string) {
    let clientBaseUrl = baseUrl;
    if (!clientBaseUrl) {
      if (typeof window === 'undefined') {
        // Server-side: use absolute URL
        clientBaseUrl = process.env.FRONTEND_URL || 'http://localhost:3000';
      } else {
        // Client-side: use relative URL (empty string works for UnifiedApiClient to use relative paths)
        clientBaseUrl = '';
      }
    }

    // We use UnifiedApiClient but pointing to our Frontend API Routes
    this.client = new UnifiedApiClient({
      baseURL: clientBaseUrl,
      platform: 'frontend'
    });
  }

  /**
   * Fetch paginated stock data from API
   */
  async getStocks(params: StockApiParams = {}): Promise<PaginatedResponse<StockFinancialData>> {
    try {
      const response = await this.client.get<PaginatedResponse<StockFinancialData>>(
        '/api/market-data/stocks',
        params as Record<string, any>
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      const data = response.data!;

      apiLogger.debug('Market data fetched', {
        params,
        totalItems: data.pagination?.total
      });

      return data;
    } catch (error) {
      apiLogger.error('Failed to fetch stock data', {
        params,
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get total count of stocks matching criteria
   */
  async getStockCount(params: Omit<StockApiParams, 'page' | 'limit'> = {}): Promise<CountResponse> {
    try {
      const response = await this.client.get<CountResponse>(
        '/api/market-data/stocks/count',
        params as Record<string, any>
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch stock count', {
        params,
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get individual stock data
   */
  async getStock(symbol: string): Promise<StockFinancialData> {
    try {
      const response = await this.client.get<StockFinancialData>(
        `/api/market-data/stocks/${symbol}`
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      apiLogger.debug('Individual stock data fetched', { symbol });

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch individual stock data', {
        symbol,
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get market summary information
   */
  async getMarketSummary(): Promise<MarketSummary> {
    try {
      const response = await this.client.get<MarketSummary>(
        '/api/market-data/stocks/summary'
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch market summary', {
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get real-time stock price
   */
  async getStockPrice(symbol: string): Promise<StockPrice> {
    try {
      const response = await this.client.get<StockPrice>(
        `/api/market-data/prices/${symbol}`
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch stock price', {
        symbol,
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get stock chart data
   */
  async getStockChart(symbol: string, timeframe: StockChart['timeframe'] = '1d'): Promise<StockChart> {
    try {
      const response = await this.client.get<StockChart>(
        `/api/market-data/charts/${symbol}`,
        { timeframe }
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch stock chart', {
        symbol,
        timeframe,
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Search stocks by symbol or company name
   */
  async searchStocks(query: string, limit: number = 10): Promise<Array<{
    symbol: string;
    companyName: string;
    sector: string;
    country: string;
  }>> {
    try {
      const response = await this.client.get<Array<{
        symbol: string;
        companyName: string;
        sector: string;
        country: string;
      }>>(
        '/api/market-data/search',
        { q: query, limit }
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to search stocks', {
        query,
        limit,
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get available sectors
   */
  async getSectors(): Promise<string[]> {
    try {
      const response = await this.client.get<string[]>(
        '/api/market-data/sectors'
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch sectors', {
        error: safeError(error).message
      });
      throw error;
    }
  }

  /**
   * Get available countries
   */
  async getCountries(): Promise<string[]> {
    try {
      const response = await this.client.get<string[]>(
        '/api/market-data/countries'
      );

      if (!response.success) {
        throw new Error(response.error?.message || 'Request failed');
      }

      return response.data!;
    } catch (error) {
      apiLogger.error('Failed to fetch countries', {
        error: safeError(error).message
      });
      throw error;
    }
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Format stock price for display
 */
export function formatStockPrice(price: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(price);
}

/**
 * Format percentage change for display
 */
export function formatPercentageChange(change: number): string {
  const formatted = new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
    signDisplay: 'always',
  }).format(change / 100);

  return formatted;
}

/**
 * Format market cap for display
 */
export function formatMarketCap(marketCap: number): string {
  if (marketCap >= 1e12) {
    return `$${(marketCap / 1e12).toFixed(1)}T`;
  } else if (marketCap >= 1e9) {
    return `$${(marketCap / 1e9).toFixed(1)}B`;
  } else if (marketCap >= 1e6) {
    return `$${(marketCap / 1e6).toFixed(1)}M`;
  } else {
    return `$${marketCap.toLocaleString()}`;
  }
}

/**
 * Get color for price change
 */
export function getPriceChangeColor(change: number): string {
  if (change > 0) return 'text-green-500';
  if (change < 0) return 'text-red-500';
  return 'text-gray-500';
}

// ============================================================================
// Exports
// ============================================================================

// Export singleton instance
export const marketApiClient = new MarketApiClient();

// Legacy export for backward compatibility
export const StockApiClient = MarketApiClient;

// Types are available from their respective modules