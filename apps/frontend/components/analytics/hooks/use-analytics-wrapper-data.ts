import { analyticsClient } from '@/lib/api-client';
import type { UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';
import type { AnalyticsFilters } from '@/types/analytics';
import { useCallback, useMemo } from 'react';

export async function fetchEPSRankings(filters: AnalyticsFilters): Promise<UnifiedAnalyticsRankingsResponse | null> {
  try {
    const queryParams = {
      page: filters.page,
      per_page: filters.limit,
      sort_by: filters.sort_by,
      country: filters.country,
      sector: filters.sector,
      min_market_cap: filters.min_eps,
      sort_order: 'desc' as const,
    };

    const response = await analyticsClient.getRankings(queryParams);
    return response ?? null;
  } catch (_error) {
    return null;
  }
}

export function useGrowthLeadersCalculation() {
  const calculateGrowthLeaders = useCallback((data: UnifiedAnalyticsRankingsResponse | null) => {
    if (!data?.rankings || data.rankings.length === 0) {return { growthLeaders: [], priceLeaders: [] };}

    const growthLeaders = data.rankings
      .sort((a, b) => (b.epsGrowth || 0) - (a.epsGrowth || 0))
      .slice(0, 3);

    const priceLeaders = data.rankings
      .sort((a, b) => (b.momentum_1m ?? 0) - (a.momentum_1m ?? 0))
      .slice(0, 3);

    return { growthLeaders, priceLeaders };
  }, []);

  return calculateGrowthLeaders;
}

export function useGrowthLeaders(data: UnifiedAnalyticsRankingsResponse | null) {
  const calculateGrowthLeaders = useGrowthLeadersCalculation();

  const { growthLeaders, priceLeaders } = useMemo(() => {
    return calculateGrowthLeaders(data);
  }, [data, calculateGrowthLeaders]);

  return { growthLeaders, priceLeaders };
}
