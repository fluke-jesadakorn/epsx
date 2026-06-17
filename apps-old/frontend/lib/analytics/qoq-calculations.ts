import type { UnifiedAnalyticsRankingsResponse, UnifiedRankingItem } from '@/types';

export interface QoQLeaders {
  epsLeaders: UnifiedRankingItem[];
  priceLeaders: UnifiedRankingItem[];
}

export function calculateQoQLeaders(data: UnifiedAnalyticsRankingsResponse | null): QoQLeaders {
  if (data == null || !Array.isArray(data.data) || data.data.length === 0) {
    return { epsLeaders: [], priceLeaders: [] };
  }

  const items = data.data;

  const epsLeaders = [...items]
    .filter(item => typeof item.analytics?.growth_factor === 'number')
    .sort((a, b) => (b.analytics?.growth_factor ?? 0) - (a.analytics?.growth_factor ?? 0))
    .slice(0, 5);

  const priceLeaders = [...items]
    .filter(item => Array.isArray(item.quarterly_data) && item.quarterly_data.length > 0)
    .sort((a, b) => {
      const aGrowth = a.quarterly_data[0]?.price_growth ?? 0;
      const bGrowth = b.quarterly_data[0]?.price_growth ?? 0;
      return bGrowth - aGrowth;
    })
    .slice(0, 5);

  return { epsLeaders, priceLeaders };
}
