'use server';

import { getStockFinancialData } from '@/lib/services/stock.service';
// SSRCache import removed - not used in current implementation
import { unstable_cache } from 'next/cache';
import type { StockFinancialData } from '@/types/financialChartData';

/**
 * Fetch public ranking data (typically lower rankings for preview)
 * This is always accessible regardless of user authentication status
 */
// Cached version using Next.js built-in caching for ISR
const getCachedPublicRankingData = unstable_cache(
  async (startRank: number, limit: number, country?: any, quarters: number = 2) => {
    const safeStartRank = Math.max(0, Math.min(startRank, 100));
    const data = await getStockFinancialData(safeStartRank, limit, country, quarters);
    
    return data.map((stock, index) => ({
      ...stock,
      publicRank: safeStartRank + index + 1,
      isPublicPreview: true,
    }));
  },
  ['public-rankings'],
  {
    revalidate: 3600, // 1 hour ISR revalidation
    tags: ['public-rankings', 'stock-data'],
  }
);

export async function fetchPublicRankingData(
  startRank: number = 0,
  limit: number = 10,
  country?: any,
  quarters: number = 2,
): Promise<StockFinancialData[]> {
  try {
    return await getCachedPublicRankingData(startRank, limit, country, quarters);
  } catch (error) {
    console.error('Error fetching public ranking data:', error);
    return [];
  }
}

/**
 * Get featured public stocks for homepage highlight
 */
// ISR cached featured stocks
const getCachedFeaturedStocks = unstable_cache(
  async (count: number) => {
    const featured = await Promise.all([
      getCachedPublicRankingData(0, 3),
      getCachedPublicRankingData(5, 3),
    ]);
    
    return featured.flat().slice(0, count);
  },
  ['featured-stocks'],
  {
    revalidate: 3600, // 1 hour ISR
    tags: ['featured-stocks', 'public-rankings'],
  }
);

export async function getFeaturedPublicStocks(count: number = 6): Promise<StockFinancialData[]> {
  try {
    return await getCachedFeaturedStocks(count);
  } catch (error) {
    console.error('Error fetching featured public stocks:', error);
    return [];
  }
}
