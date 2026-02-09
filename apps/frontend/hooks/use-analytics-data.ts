import { TIMEOUT } from '@/shared/config/constants';
import { getRankingsAction } from '@/app/actions/analytics';
import type { AnalyticsFilters } from '@/types/analytics';
import { DEFAULT_FILTER_OPTIONS, type RichFilterOptions } from '@/types/dashboard';
import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useAnalyticsData(filters: AnalyticsFilters) {
  // Create a stable query key based on filters
  const queryKey = useMemo(() => ['analytics-rankings', filters] as const, [filters]);

  const { data: response, error: queryError, isLoading, refetch } = useQuery({
    queryKey,
    queryFn: () => getRankingsAction(filters),
    staleTime: TIMEOUT.ANALYTICS_STALE,
    refetchOnWindowFocus: false,
  });

  // Extract data from the response (which is CardDashboardResponse)
  const data = response?.success ? response : null;
  const error = queryError ? 'Failed to load analytics data' : (!response?.success ? response?.message : null);

  const filterOptions: RichFilterOptions = DEFAULT_FILTER_OPTIONS;

  return {
    data: data as any, // Cast to any to match existing usage in components
    filterOptions,
    isLoading,
    error: (error ?? null),
    refetch
  };
}