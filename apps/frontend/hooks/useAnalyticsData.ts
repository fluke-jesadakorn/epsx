'use client';

import { useEffect, useState } from 'react';

import { analyticsClient, UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';
import { analyticsLogger } from '@/lib/utils';
import {
  type RichFilterOptions,
  DEFAULT_FILTER_OPTIONS
} from '@/shared/hooks';
import type { AnalyticsFilters } from '@/types/analytics';

// ============================================================================
// TYPES
// ============================================================================

interface QoQLeaders {
  epsLeaders: any[];
  priceLeaders: any[];
}

// ============================================================================
// FETCHER FOR SWR
// ============================================================================

const rankingsFetcher = async (url: string): Promise<UnifiedAnalyticsRankingsResponse | null> => {
  try {
    const response = await analyticsClient.getRankings({});
    return response || null;
  } catch (error) {
    analyticsLogger.error('Failed to fetch rankings', error);
    return null;
  }
};

// ============================================================================
// MAIN HOOK
// ============================================================================

export function useAnalyticsData(filters: AnalyticsFilters) {
  const [data, setData] = useState<UnifiedAnalyticsRankingsResponse | null>(null);
  const [filterOptions, setFilterOptions] = useState<RichFilterOptions>(DEFAULT_FILTER_OPTIONS);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch EPS rankings
  const fetchEPSRankings = async (currentFilters: AnalyticsFilters): Promise<UnifiedAnalyticsRankingsResponse | null> => {
    try {
      const queryParams = {
        page: currentFilters.page,
        per_page: currentFilters.limit,
        sort_by: currentFilters.sort_by,
        country: currentFilters.country,
        sector: currentFilters.sector,
        min_market_cap: currentFilters.min_eps,
        sort_order: 'desc' as const,
      };

      const response = await analyticsClient.getRankings(queryParams);
      return response || null;
    } catch (error) {
      analyticsLogger.error('Failed to fetch EPS rankings', error);
      return null;
    }
  };

  // Load initial data
  useEffect(() => {
    const loadInitialData = async () => {
      setIsLoading(true);
      try {
        const rankingsData = await fetchEPSRankings(filters);
        setData(rankingsData);
        // Use default filter options from shared (no API call needed)
        setFilterOptions(DEFAULT_FILTER_OPTIONS);
        setError(null);
      } catch (err) {
        setError('Failed to load analytics data');
        analyticsLogger.error('Failed to load initial analytics data', err);
      } finally {
        setIsLoading(false);
      }
    };

    loadInitialData();
  }, []);

  // Reload data when filters change
  useEffect(() => {
    const loadFilteredData = async () => {
      if (isLoading) return; // Prevent multiple simultaneous requests

      setIsLoading(true);
      try {
        const rankingsData = await fetchEPSRankings(filters);
        setData(rankingsData);
        setError(null);
      } catch (err) {
        setError('Failed to load filtered data');
        analyticsLogger.error('Failed to load filtered analytics data', err);
      } finally {
        setIsLoading(false);
      }
    };

    loadFilteredData();
  }, [filters.page, filters.limit, filters.sort_by, filters.country, filters.sector, filters.min_eps, filters.min_growth]);

  return {
    data,
    filterOptions,
    isLoading,
    error,
    refetch: () => fetchEPSRankings(filters)
  };
}