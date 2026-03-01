import { getRankingsAction } from '@/app/actions/analytics';
import type { AnalyticsFilters } from '@/shared/api/analytics';
import type { UnifiedAnalyticsRankingsResponse } from '@/types';
import { DEFAULT_FILTER_OPTIONS, type RichFilterOptions } from '@/types/dashboard';
import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useAnalyticsData(filters: Partial<AnalyticsFilters>) {
  // Create a stable query key based on filters
  const queryKey = useMemo(() => ['analytics-rankings', filters] as const, [filters]);

  const { data: response, error: queryError, isLoading, refetch } = useQuery({
    queryKey,
    queryFn: () => getRankingsAction(filters),
    staleTime: 5 * 60_000,
    gcTime: 10 * 60_000,
    refetchOnWindowFocus: false,
  });

  // Extract data from the response (which is CardDashboardResponse)
  const data = (response?.success ?? false) ? response : null;
  const error = queryError ? 'Failed to load analytics data' : ((response?.success ?? true) ? null : response?.message);

  const filterOptions: RichFilterOptions = DEFAULT_FILTER_OPTIONS;

  return {
    data: data as unknown as UnifiedAnalyticsRankingsResponse | null,
    filterOptions,
    isLoading,
    error: (error ?? null),
    refetch
  };
}