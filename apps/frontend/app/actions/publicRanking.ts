'use server';

import { getStockFinancialData } from '@/lib/services/stock.service';
import type { StockFinancialData } from '@/types/financialChartData';

/**
 * Fetch public ranking data (typically lower rankings for preview)
 * This is always accessible regardless of user authentication status
 */
export async function fetchPublicRankingData(
  startRank: number = 100,
  limit: number = 10,
  country?: any,
  quarters: number = 4,
): Promise<StockFinancialData[]> {
  try {
    // Fetch data starting from the specified rank
    const data = await getStockFinancialData(startRank, limit, country, quarters);
    
    // Add public ranking metadata
    return data.map((stock, index) => ({
      ...stock,
      publicRank: startRank + index + 1,
      isPublicPreview: true,
    }));
  } catch (error) {
    console.error('Error fetching public ranking data:', error);
    return [];
  }
}

/**
 * Get featured public stocks for homepage highlight
 */
export async function getFeaturedPublicStocks(count: number = 6): Promise<StockFinancialData[]> {
  try {
    // Get a mix of rankings for variety (e.g., 100-102, 105-107, 108-110)
    const featured = await Promise.all([
      fetchPublicRankingData(100, 3),
      fetchPublicRankingData(105, 3),
    ]);
    
    return featured.flat().slice(0, count);
  } catch (error) {
    console.error('Error fetching featured public stocks:', error);
    return [];
  }
}
