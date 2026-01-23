import { getRankingsAction } from '@/app/actions/analytics';
import { useServerActionSWR } from '@/lib/infrastructure/swr-adapter';
import type { AnalyticsFilters } from '@/types/analytics';
import { DEFAULT_FILTER_OPTIONS, type RichFilterOptions } from '@/types/dashboard';
import { useMemo } from 'react';

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useAnalyticsData(filters: AnalyticsFilters) {
  // Create a stable SWR key based on filters
  const swrKey = useMemo(() => {
    return `analytics-rankings-${JSON.stringify(filters)}`;
  }, [filters]);

  const { data: response, error: swrError, isLoading: dataLoading, mutate } = useServerActionSWR(
    swrKey,
    () => getRankingsAction(filters),
    {
      revalidateOnFocus: false,
      dedupingInterval: 10000, // 10 seconds
    }
  );

  // Extract data from the response (which is CardDashboardResponse)
  const data = response?.success ? response : null;
  const error = swrError ? 'Failed to load analytics data' : (!response?.success ? response?.message : null);
  const isLoading = dataLoading;

  const filterOptions: RichFilterOptions = DEFAULT_FILTER_OPTIONS;

  return {
    data: data as any, // Cast to any to match existing usage in components
    filterOptions,
    isLoading,
    error: (error || null) as string | null,
    refetch: mutate
  };
}