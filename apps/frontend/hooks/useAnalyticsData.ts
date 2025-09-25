'use client';

import { useState, useEffect, useMemo } from 'react';
import { analyticsClient, UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';
import type { AnalyticsFilters } from '@/types/analytics';
import { analyticsLogger } from '@/lib/utils';

interface RichFilterOptions {
  countries: Array<{ value: string; label: string }>;
  sectors: string[];
  exchanges?: string[];
  stock_types?: string[];
}

interface QoQLeaders {
  epsLeaders: any[];
  priceLeaders: any[];
}


export function useAnalyticsData(filters: AnalyticsFilters) {
  const [data, setData] = useState<UnifiedAnalyticsRankingsResponse | null>(null);
  const [filterOptions, setFilterOptions] = useState<RichFilterOptions>({
    countries: [],
    sectors: [],
    exchanges: [],
    stock_types: [],
  });
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
      return response;
    } catch (error) {
      analyticsLogger.error('Failed to fetch EPS rankings', error);
      return null;
    }
  };

  // Fetch filter options - use fallback data since API methods don't exist
  const fetchFilterOptions = async (): Promise<RichFilterOptions> => {
    // Use fallback data directly since getAvailableCountries/getSectorsByCountry don't exist
    return {
      countries: [
        { value: 'america', label: 'United States' },
        { value: 'canada', label: 'Canada' },
        { value: 'united_kingdom', label: 'United Kingdom' },
        { value: 'germany', label: 'Germany' },
        { value: 'france', label: 'France' },
        { value: 'japan', label: 'Japan' },
        { value: 'australia', label: 'Australia' }
      ],
      sectors: [
        'Technology',
        'Healthcare', 
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ],
      exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
      stock_types: ['common', 'preferred', 'reit', 'etf'],
    };
  };

  // Load initial data
  useEffect(() => {
    const loadInitialData = async () => {
      setIsLoading(true);
      try {
        const [rankingsData, filterOptionsData] = await Promise.all([
          fetchEPSRankings(filters),
          fetchFilterOptions()
        ]);
        
        setData(rankingsData);
        setFilterOptions(filterOptionsData);
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