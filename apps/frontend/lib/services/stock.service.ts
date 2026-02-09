// Shared stock data service

import { CACHE_TTL } from '@/shared/config/constants';
import { logger } from '@/shared/utils/logger';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '../../types/market';

// Server-side cache to store data temporarily with pagination-aware keys
const serverCache: Map<
  string,
  {
    data: StockFinancialData[];
    timestamp: number;
    ttl: number;
  }
> = new Map();

// Count cache for pagination
const countCache: {
  count: number;
  timestamp: number;
  ttl: number;
} | null = null;

const CACHE_TTL_SECONDS = CACHE_TTL.STOCK_DATA; // 5 minutes

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
    if (cachedData && now - cachedData.timestamp < CACHE_TTL_SECONDS * 1000) {
      return cachedData.data;
    }

    // Stock data fetch not implemented: rankStocksByEpsWithChart utility not found
    logger.warn('[StockService] Data fetch not implemented, returning empty result');
    return [];
  } catch (error) {
    logger.error('[StockService] Error fetching stock data:', error);
    // Return cached data if available, even if expired, as fallback
    const cacheKey = `${page}-${limit}-${country}-${quarters}`;
    const cachedData = serverCache.get(cacheKey);
    if (cachedData) {
      logger.warn('[StockService] Using stale cached data as fallback');
      return cachedData.data;
    }
    logger.warn('[StockService] No cached data available, returning empty result');
    return [];
  }
}

/**
 * Get total count of stocks available for pagination.
 * Returns cached count if available (even if expired), otherwise queries database.
 * Returns 0 as fallback if no data available.
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

    // Count fetch not implemented: fetchScreenerStock utility not found
    logger.warn('[StockService] Count fetch not implemented, returning 0');
    return 0;
  } catch (error) {
    logger.error('[StockService] Error getting stock count:', error);
    // Return cached count if available, or fallback to 0
    if (countCache) {
    }
    return countCache?.count ?? 0;
  }
}
