import type { UnifiedAnalyticsRankingsResponse, UnifiedRankingItem } from '@/lib/api-client';

export interface QoQLeaders {
  epsLeaders: UnifiedRankingItem[];
  priceLeaders: UnifiedRankingItem[];
}

/**
 * Calculate Quarter-over-Quarter leaders from analytics data
 * Uses rich UnifiedRankingItem data with quarterly information
 */
export function calculateQoQLeaders(data: UnifiedAnalyticsRankingsResponse | null): QoQLeaders {
  if (!data?.data || data.data.length === 0) {
    return { epsLeaders: [], priceLeaders: [] };
  }

  // Filter companies with quarterly data
  const companiesWithQoQ = data.data.filter(ranking => 
    ranking.quarterly_data && ranking.quarterly_data.length >= 2
  );

  // Calculate EPS QoQ leaders using analytics.growth_factor for better accuracy
  const epsLeaders = companiesWithQoQ
    .filter(ranking => 
      ranking.analytics.growth_factor !== null && 
      ranking.analytics.growth_factor !== undefined
    )
    .sort((a, b) => b.analytics.growth_factor - a.analytics.growth_factor)
    .slice(0, 3);

  // Calculate Price QoQ leaders using quarterly_data
  const priceLeaders = companiesWithQoQ
    .filter(ranking => {
      const latestQuarter = ranking.quarterly_data?.[0];
      const previousQuarter = ranking.quarterly_data?.[1];
      const latestGrowth = latestQuarter?.price_growth || 0;
      const previousGrowth = previousQuarter?.price_growth || 0;
      const displayGrowth = latestGrowth === 0 ? previousGrowth : latestGrowth;
      return displayGrowth !== null && displayGrowth !== undefined && displayGrowth !== 0;
    })
    .sort((a, b) => {
      const aLatest = a.quarterly_data?.[0]?.price_growth || 0;
      const aPrevious = a.quarterly_data?.[1]?.price_growth || 0;
      const aGrowth = aLatest === 0 ? aPrevious : aLatest;
      
      const bLatest = b.quarterly_data?.[0]?.price_growth || 0;
      const bPrevious = b.quarterly_data?.[1]?.price_growth || 0;
      const bGrowth = bLatest === 0 ? bPrevious : bLatest;
      
      return bGrowth - aGrowth;
    })
    .slice(0, 3);

  return { epsLeaders, priceLeaders };
}

/**
 * Get the display growth percentage for a ranking item
 * Falls back to previous quarter if current quarter is 0
 */
export function getDisplayGrowthPercentage(ranking: UnifiedRankingItem): number {
  if (!ranking.quarterly_data || ranking.quarterly_data.length === 0) {
    return ranking.analytics.growth_factor || 0;
  }

  const latestQuarter = ranking.quarterly_data[0];
  const previousQuarter = ranking.quarterly_data[1];
  
  const latestGrowth = latestQuarter?.price_growth || 0;
  const previousGrowth = previousQuarter?.price_growth || 0;
  
  return latestGrowth === 0 ? previousGrowth : latestGrowth;
}

/**
 * Format growth percentage for display
 */
export function formatGrowthPercentage(growth: number): string {
  if (growth === 0) return '0.0%';
  return `${growth > 0 ? '+' : ''}${growth.toFixed(1)}%`;
}

/**
 * Get quarter label for display (e.g., "Q1 2024")
 */
export function getQuarterLabel(quarter: string): string {
  // Assuming quarter format is "2024-Q1" or similar
  const parts = quarter.split('-');
  if (parts.length === 2) {
    return `${parts[1]} ${parts[0]}`;
  }
  return quarter;
}