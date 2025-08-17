'use client';

import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import axios from 'axios';
import type { CardDashboardResponse } from '@/types/financialChartData';

export interface AnalyticsFilters {
  // Core pagination with page-based approach
  page: number;
  limit: number;
  
  // Geographic and sector filters
  country?: string;
  sector?: string;
  
  // Additional filtering parameters
  sort_by: string;
  min_eps?: number;
  min_growth?: number;
}

export interface FilterOptions {
  countries: string[];
  sectors: string[];
}

export interface AnalyticsFiltersState {
  filters: AnalyticsFilters;
  data: CardDashboardResponse | null;
  options: FilterOptions;
  loading: boolean;
  error: string | null;
}

const DEFAULT_FILTERS: AnalyticsFilters = {
  page: 1,
  limit: 12,
  sort_by: 'market_cap',
};

export const useAnalyticsFilters = () => {
  const [state, setState] = useState<AnalyticsFiltersState>({
    filters: DEFAULT_FILTERS,
    data: null,
    options: { countries: [], sectors: [] },
    loading: false,
    error: null,
  });

  const filtersRef = useRef(DEFAULT_FILTERS);

  const analyticsClient = useMemo(() => {
    return axios.create({
      baseURL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
      timeout: 30000, // Increased to 30 seconds for analytics API calls
      headers: {
        'Content-Type': 'application/json',
      },
    });
  }, []);

  // Get current page directly from filters
  const getCurrentPage = useCallback((page: number) => {
    return page;
  }, []);

  // Fetch data with current filters
  const fetchData = useCallback(async (newFilters?: Partial<AnalyticsFilters>) => {
    // Update filters ref first
    const currentFilters = { ...filtersRef.current, ...newFilters };
    filtersRef.current = currentFilters;
    
    // Update state with new filters and loading state
    setState(prev => ({ 
      ...prev, 
      filters: currentFilters,
      loading: true, 
      error: null 
    }));

    try {
      const page = currentFilters.page;
      
      // Use Next.js API route as proxy to avoid CORS issues
      const params = new URLSearchParams({
        page: page.toString(),
        limit: currentFilters.limit.toString(),
        sort_by: currentFilters.sort_by,
        ...(currentFilters.country && { country: currentFilters.country }),
        ...(currentFilters.sector && { sector: currentFilters.sector }),
        ...(currentFilters.min_eps && { min_eps: currentFilters.min_eps.toString() }),
        ...(currentFilters.min_growth && { min_growth: currentFilters.min_growth.toString() }),
      });

      const url = `/api/analytics/rankings?${params.toString()}`;
      console.log('🔄 Fetching analytics data from:', url);

      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      console.log('✅ Analytics data received:', data);

      if (data.success) {
        setState(prev => ({
          ...prev,
          data: data,
          options: {
            countries: ['All Countries', ...data.metadata.available_countries],
            sectors: ['All Sectors', ...(data.metadata.available_sectors || [])],
          },
          loading: false,
        }));
      } else {
        throw new Error(data.error || 'Failed to fetch analytics data');
      }
    } catch (error) {
      console.error('❌ Analytics API error:', error);
      setState(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      }));
    }
  }, []); // No dependencies to avoid infinite loops

  // Update specific filter
  const updateFilter = useCallback(
    <K extends keyof AnalyticsFilters>(key: K, value: AnalyticsFilters[K]) => {
      const newFilters = { ...state.filters, [key]: value };
      
      // Reset to first page when changing filters other than pagination
      if (key !== 'page' && key !== 'limit') {
        newFilters.page = 1;
      }
      
      fetchData(newFilters);
    },
    [fetchData, state.filters]
  );

  // Update multiple filters at once
  const updateFilters = useCallback(
    (newFilters: Partial<AnalyticsFilters>) => {
      fetchData(newFilters);
    },
    [fetchData]
  );

  // Pagination helpers
  const goToPage = useCallback(
    (page: number) => {
      updateFilter('page', page);
    },
    [updateFilter]
  );

  const changePageSize = useCallback(
    (newLimit: number) => {
      const currentPage = state.filters.page;
      updateFilters({ limit: newLimit, page: currentPage });
    },
    [updateFilters, state.filters.page]
  );

  const nextPage = useCallback(() => {
    if (state.data?.pagination.hasNext) {
      const nextPage = state.filters.page + 1;
      updateFilter('page', nextPage);
    }
  }, [state.data?.pagination.hasNext, state.filters.page, updateFilter]);

  const prevPage = useCallback(() => {
    if (state.data?.pagination.hasPrev && state.filters.page > 1) {
      const prevPage = Math.max(1, state.filters.page - 1);
      updateFilter('page', prevPage);
    }
  }, [state.data?.pagination.hasPrev, state.filters.page, updateFilter]);

  // Country filter helpers
  const setCountry = useCallback(
    (country: string) => {
      const countryValue = country === 'All Countries' ? undefined : country;
      updateFilter('country', countryValue);
    },
    [updateFilter]
  );

  // Sector filter helpers
  const setSector = useCallback(
    (sector: string) => {
      const sectorValue = sector === 'All Sectors' ? undefined : sector;
      updateFilter('sector', sectorValue);
    },
    [updateFilter]
  );

  // Reset all filters
  const resetFilters = useCallback(() => {
    fetchData(DEFAULT_FILTERS);
  }, [fetchData]);

  // Get filter presets
  const getFilterPresets = useCallback(() => {
    return {
      default: DEFAULT_FILTERS,
      topPerformers: {
        ...DEFAULT_FILTERS,
        sort_by: 'market_cap',
        min_growth: 5,
      },
      usStocks: {
        ...DEFAULT_FILTERS,
        country: 'america',
      },
      techSector: {
        ...DEFAULT_FILTERS,
        sector: 'Technology',
      },
      highEps: {
        ...DEFAULT_FILTERS,
        sort_by: 'market_cap',
        min_eps: 1,
      },
    };
  }, []);

  const applyPreset = useCallback(
    (presetName: keyof ReturnType<typeof getFilterPresets>) => {
      const presets = getFilterPresets();
      const preset = presets[presetName];
      if (preset) {
        fetchData(preset);
      }
    },
    [fetchData, getFilterPresets]
  );

  // Initialize data on first load
  useEffect(() => {
    fetchData();
  }, []); // Empty dependency array to run only once on mount

  return {
    // State
    filters: state.filters,
    data: state.data,
    options: state.options,
    loading: state.loading,
    error: state.error,

    // Filter operations
    updateFilter,
    updateFilters,
    resetFilters,

    // Pagination
    goToPage,
    changePageSize,
    nextPage,
    prevPage,
    currentPage: state.filters.page,

    // Specific filter helpers
    setCountry,
    setSector,

    // Presets
    getFilterPresets,
    applyPreset,

    // Utilities
    refresh: () => fetchData(),
  };
};