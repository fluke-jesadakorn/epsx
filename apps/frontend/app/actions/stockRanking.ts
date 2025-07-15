'use server';

import type { StockFinancialData } from '@/types/financialChartData';
import { getStockFinancialData } from '@/lib/services/stock.service';

/**
 * Reusable server action for fetching stock financial data
 * Uses the same API as /analytics page for caching support
 * Can be used across different zones/pages
 */
export async function fetchStockRankingData(
  skip = 0,
  limit = 10,
  country?: any,
  quarters = 4,
): Promise<StockFinancialData[]> {
  // Use the same service as analytics page to leverage caching
  return getStockFinancialData(skip, limit, country, quarters);
}

/**
 * Fetch data with rank offset for different zones
 * The ranking difference is only in display, not in API logic
 */
export async function fetchStockRankingDataWithOffset(
  rankOffset = 0,
  skip = 0,
  limit = 10,
  country?: any,
  quarters = 4,
): Promise<{ data: StockFinancialData[]; rankOffset: number }> {
  const data = await getStockFinancialData(skip, limit, country, quarters);
  
  return {
    data,
    rankOffset,
  };
}
