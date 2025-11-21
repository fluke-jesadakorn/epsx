'use server';

import type { StockFinancialData } from '@/types/financialChartData';
import { getStockFinancialData } from '@/lib/services/stock.service';
import { getRankingLimitFromPermissions, getDisplayTierFromPermissions } from '@/app/constants/packages';
import { MarketCountry } from '@/types/market';

// Helper function to extract ranking limit from permissions
function extractRankingLimitFromPermissions(permissions: string[]): number {
  return getRankingLimitFromPermissions(permissions);
}

// Helper function to derive tier from permissions
function deriveTierFromPermissions(permissions: string[]): string {
  return getDisplayTierFromPermissions(permissions);
}

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
 * Fetch data with user access control based on permissions
 * Respects user permissions for ranking limits (NEW - Permission-based)
 */
export async function fetchStockRankingDataWithPermissions(
  userPermissions: string[],
  isExpired: boolean = true,
  page = 1,
  country?: typeof MarketCountry,
  quarters = 2,
): Promise<StockFinancialData[]> {
  const maxLimit = isExpired ? 5 : extractRankingLimitFromPermissions(userPermissions);

  // Determine tier for additional logic
  const derivedTier = deriveTierFromPermissions(userPermissions);

  // Always fetch a bit more for premium users to show locked items
  const fetchLimit = derivedTier === 'BRONZE' ? maxLimit : Math.min((maxLimit === -1 ? 100 : maxLimit) + 10, 100);

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
  country?: typeof MarketCountry,
  quarters = 2,
): Promise<{ data: StockFinancialData[]; rankOffset: number }> {
  const data = await getStockFinancialData(page, limit, country, quarters);

  return {
    data,
    rankOffset,
  };
}
