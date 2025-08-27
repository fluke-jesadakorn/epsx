// Server component for fetching analytics data for user assets
import { AnalyticsClient } from '@/lib/api-client';
import type { UnifiedRankingItem } from '@/types/analytics';

// Server-side data fetching for user assets analytics
export async function getAssetsAnalyticsData(symbols: string[]) {
  if (symbols.length === 0) {
    return [];
  }

  try {
    const analyticsClient = new AnalyticsClient();
    
    // Fetch analytics data for assets
    // Note: In a production environment, we would have an endpoint to fetch specific symbols
    // For now, we'll fetch top rankings and filter
    const response = await analyticsClient.getUnifiedAnalyticsRankings({
      page: 1,
      limit: 100, // Fetch more to increase chances of finding our assets
      sort_by: 'growth_factor',
    });

    if (response.data && response.data.data) {
      // Filter to only include our assets
      const filteredData = response.data.data.filter(item => 
        symbols.includes(item.symbol)
      );
      return filteredData;
    }
    
    return [];
  } catch (error) {
    console.error('Error fetching assets analytics data:', error);
    return [];
  }
}