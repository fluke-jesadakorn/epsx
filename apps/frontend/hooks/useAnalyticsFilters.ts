'use client';

import { useState, useCallback, useMemo } from 'react';
import type { AnalyticsFilters, FilterOptions } from '@/types/analytics';

const DEFAULT_FILTERS: AnalyticsFilters = {
  sort_by: 'ranking_position',
  page: 1,
  limit: 20,
};

export function useAnalyticsFilters() {
  const [filters, setFilters] = useState<AnalyticsFilters>(DEFAULT_FILTERS);
  const [isLoading, setIsLoading] = useState(false);

  const updateFilters = useCallback((newFilters: Partial<AnalyticsFilters>) => {
    setFilters(prev => ({
      ...prev,
      ...newFilters,
      // Reset to page 1 when filters change (except when changing page)
      page: 'page' in newFilters ? newFilters.page! : 1,
    }));
  }, []);

  const resetFilters = useCallback(() => {
    setFilters(DEFAULT_FILTERS);
  }, []);

  const changePage = useCallback((page: number) => {
    setFilters(prev => ({ ...prev, page }));
  }, []);

  const hasActiveFilters = useMemo(() => {
    return !!(filters.country || filters.sector || filters.min_eps || filters.min_growth);
  }, [filters]);

  const activeFilterCount = useMemo(() => {
    let count = 0;
    if (filters.country) count++;
    if (filters.sector) count++;
    if (filters.min_eps) count++;
    if (filters.min_growth) count++;
    return count;
  }, [filters]);

  const clearFilter = useCallback((filterKey: keyof AnalyticsFilters) => {
    setFilters(prev => {
      const newFilters = { ...prev };
      delete newFilters[filterKey];
      return { ...newFilters, page: 1 };
    });
  }, []);

  return {
    filters,
    updateFilters,
    resetFilters,
    changePage,
    hasActiveFilters,
    activeFilterCount,
    clearFilter,
    isLoading,
    setIsLoading,
  };
}