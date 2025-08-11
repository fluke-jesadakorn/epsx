// Shared stock data service

import { transformFinancialDataWithCurrentPrice } from '@/utils';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '../../../../types/marketCountries';

// Server-side cache to store data temporarily with pagination-aware keys
let serverCache: Map<
  string,
  {
    data: StockFinancialData[];
    timestamp: number;
    ttl: number;
  }
> = new Map();

// Count cache for pagination
let countCache: {
  count: number;
  timestamp: number;
  ttl: number;
} | null = null;

const CACHE_TTL = 300; // 5 minutes in seconds

export async function getStockFinancialData(
  page = 1,
  limit = 10,
  country: typeof MarketCountry = MarketCountry,
  quarters = 2,
): Promise<StockFinancialData[]> {
  try {
    // Create a cache key that includes pagination parameters
    const cacheKey = `${page}-${limit}-${country}-${quarters}`;

    // Check server-side cache first
    const now = Date.now();
    const cachedData = serverCache.get(cacheKey);
    if (cachedData && now - cachedData.timestamp < cachedData.ttl * 1000) {
      return cachedData.data;
    }

    // Unable to fetch data: rankStocksByEpsWithChart utility not found.
    return [];
  } catch (error) {
    // Return cached data if available, even if expired, as fallback
    const cacheKey = `${page}-${limit}-${country}-${quarters}`;
    const cachedData = serverCache.get(cacheKey);
    if (cachedData) {
      return cachedData.data;
    }
    return [];
  }
}

/**
 * Get total count of stocks for pagination
 */
export async function getStockFinancialDataCount(
  country: typeof MarketCountry = MarketCountry,
  quarters = 2, // Currently unused but kept for future enhancements
): Promise<number> {
  try {
    // Check count cache first
    const now = Date.now();
    if (countCache && now - countCache.timestamp < countCache.ttl * 1000) {
      return countCache.count;
    }

    // Unable to fetch count: fetchScreenerStock utility not found.
    return 0;
  } catch (error) {
    console.error('Error getting stock count:', error);
    // Return cached count if available, or fallback to 0
    return countCache?.count || 0;
  }
}
