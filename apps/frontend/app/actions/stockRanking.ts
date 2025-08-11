'use server';

import type { StockFinancialData } from '@/types/financialChartData';
import { getStockFinancialData } from '@/lib/services/stock.service';
import { getPackageByLevel } from '@/app/constants/packages';
import type { UserLevelType } from '@/app/constants/packages';

/**
 * Reusable server action for fetching stock financial data
 * Uses the same API as /analytics page for caching support
 * Can be used across different zones/pages
 */
export async function fetchStockRankingData(
  page = 1,
  limit = 10,
  country?: any,
  quarters = 2,
): Promise<StockFinancialData[]> {
  // Use the same service as analytics page to leverage caching
  return getStockFinancialData(page, limit, country, quarters);
}

/**
 * Fetch data with user access control
 * Respects user subscription level for ranking limits
 */
export async function fetchStockRankingDataForUser(
  userLevel: UserLevelType = 'BRONZE',
  isExpired: boolean = true,
  page = 1,
  country?: any,
  quarters = 2,
): Promise<StockFinancialData[]> {
  const currentPackage = getPackageByLevel(userLevel);
  const maxLimit = isExpired ? 5 : (currentPackage?.rankingLimit || 5);
  
  // Always fetch a bit more for premium users to show locked items
  const fetchLimit = userLevel === 'BRONZE' ? maxLimit : Math.min(maxLimit + 10, 100);
  
  return getStockFinancialData(page, fetchLimit, country, quarters);
}

/**
 * Fetch data with rank offset for different zones
 * The ranking difference is only in display, not in API logic
 */
export async function fetchStockRankingDataWithOffset(
  rankOffset = 0,
  page = 1,
  limit = 10,
  country?: any,
  quarters = 2,
): Promise<{ data: StockFinancialData[]; rankOffset: number }> {
  const data = await getStockFinancialData(page, limit, country, quarters);
  
  return {
    data,
    rankOffset,
  };
}
