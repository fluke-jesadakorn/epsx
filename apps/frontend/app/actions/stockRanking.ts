'use server';

import { getStockFinancialData } from '@/lib/services/stock.service';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '@/types/market';


/**
 * Reusable server action for fetching stock financial data
 * Uses the same API as /analytics page for caching support
 * Can be used across different zones/pages
 */
export async function fetchStockRankingData(
  page = 1,
  limit = 10,
  country?: typeof MarketCountry,
  quarters = 2,
): Promise<StockFinancialData[]> {
  // Use the same service as analytics page to leverage caching
  return getStockFinancialData(page, limit, country, quarters);
}

/**
 * Fetch stock ranking data
 * Legacy wrapper for backward compatibility.
 */
export async function fetchStockRankingDataWithPermissions(
  userPermissions: string[],
  isExpired = true,
  page = 1,
  country?: typeof MarketCountry,
  quarters = 2,
): Promise<StockFinancialData[]> {
  const defaultLimit = 100;
  return getStockFinancialData(page, defaultLimit, country, quarters);
}


/**
 * Fetch data with rank offset for different zones
 * The ranking difference is only in display, not in API logic
 */
export async function fetchStockRankingDataWithOffset(
  rankOffset = 0,
  page = 1,
  limit = 10,
  country?: typeof MarketCountry,
  quarters = 2,
): Promise<{ data: StockFinancialData[]; rankOffset: number }> {
  const data = await getStockFinancialData(page, limit, country, quarters);

  return {
    data,
    rankOffset,
  };
}
