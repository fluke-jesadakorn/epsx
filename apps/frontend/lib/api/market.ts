/**
 * Market Data API Client
 * Stock market data, financial data, and trading information
 */

import { apiClient } from './client';
import type { CountResponse, StockFinancialData } from './client';
import type { PaginatedResponse } from '../../../../shared/types/api';
import type { StockFinancialData as LocalStockFinancialData } from '@/types/financialChartData';
import { apiLogger, safeError } from '@/lib/utils/logging';

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
  async getStocks(params: StockApiParams = {}): Promise<PaginatedResponse<StockFinancialData>> {
    try {
      const searchParams = new URLSearchParams();
      
      // Add pagination parameters
      if (params.page) searchParams.set('page', params.page.toString());
      if (params.limit) searchParams.set('limit', params.limit.toString());
      if (params.country) searchParams.set('country', params.country);
      if (params.quarters) searchParams.set('quarters', params.quarters.toString());
      if (params.sector) searchParams.set('sector', params.sector);
      if (params.min_market_cap) searchParams.set('min_market_cap', params.min_market_cap.toString());
      if (params.sort_by) searchParams.set('sort_by', params.sort_by);
      if (params.sort_order) searchParams.set('sort_order', params.sort_order);

      const url = `${this.baseUrl}?${searchParams.toString()}`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      
      apiLogger.debug('Market data fetched', { 
        params, 
        totalItems: data.pagination?.total_items 
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
      const searchParams = new URLSearchParams();
      
      if (params.country) searchParams.set('country', params.country);
      if (params.quarters) searchParams.set('quarters', params.quarters.toString());
      if (params.sector) searchParams.set('sector', params.sector);
      if (params.min_market_cap) searchParams.set('min_market_cap', params.min_market_cap.toString());

      const url = `${this.baseUrl}/count?${searchParams.toString()}`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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
      const url = `${this.baseUrl}/${symbol}`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      
      apiLogger.debug('Individual stock data fetched', { symbol });

      return data;
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
      const url = `${this.baseUrl}/summary`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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
      const url = `/api/v1/market-data/prices/${symbol}`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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
      const url = `/api/v1/market-data/charts/${symbol}?timeframe=${timeframe}`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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
      const url = `/api/v1/market-data/search?q=${encodeURIComponent(query)}&limit=${limit}`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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
      const url = `/api/v1/market-data/sectors`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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
      const url = `/api/v1/market-data/countries`;
      
      const response = await fetch(url, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
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