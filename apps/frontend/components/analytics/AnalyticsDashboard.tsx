'use client';

import { fetchEPSRankings, fetchFilterOptions } from '@/app/actions/analytics';
import { useAnalyticsFilters } from '@/hooks/useAnalyticsFilters';
import type { EPSRankingsResponse, FilterOptions } from '@/types/analytics';
import { useEffect, useState } from 'react';
import FilterPanel from './FilterPanel';
import Pagination from './Pagination';
import StockCard from './StockCard';

export default function AnalyticsDashboard() {
  const {
    filters,
    updateFilters,
    resetFilters,
    changePage,
    hasActiveFilters,
    activeFilterCount,
    isLoading,
    setIsLoading,
  } = useAnalyticsFilters();

  const [data, setData] = useState<EPSRankingsResponse | null>(null);
  const [filterOptions, setFilterOptions] = useState<FilterOptions>({
    countries: [],
    sectors: [],
  });
  const [error, setError] = useState<string | null>(null);

  // Load filter options on mount
  useEffect(() => {
    const loadFilterOptions = async () => {
      try {
        const options = await fetchFilterOptions();
        setFilterOptions(options);
      } catch (err) {
        console.error('Failed to load filter options:', err);
      }
    };

    loadFilterOptions();
  }, []);

  // Load data when filters change
  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const result = await fetchEPSRankings(filters);
        if (result) {
          setData(result);
        } else {
          setError('Failed to load rankings data');
        }
      } catch (err) {
        console.error('Error loading analytics data:', err);
        setError('An error occurred while loading data');
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [filters, setIsLoading]);

  const refreshData = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const result = await fetchEPSRankings(filters);
      if (result) {
        setData(result);
      } else {
        setError('Failed to refresh data');
      }
    } catch (err) {
      setError('Failed to refresh data');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto px-4 py-6">
        {/* Header */}
        <div className="mb-6">
          <h1 className="mb-2 text-2xl font-bold text-gray-900 sm:text-3xl">
            EPS Rankings
          </h1>
          <p className="text-gray-600">
            Real-time EPS growth analysis and stock performance rankings
          </p>
        </div>

        {/* Layout: Sidebar on desktop, stacked on mobile */}
        <div className="lg:grid lg:grid-cols-4 lg:gap-6">
          {/* Filters sidebar */}
          <div className="mb-6 lg:col-span-1 lg:mb-0">
            <FilterPanel
              filters={filters}
              options={filterOptions}
              onFiltersChange={updateFilters}
              isLoading={isLoading}
            />
          </div>

          {/* Main content */}
          <div className="lg:col-span-3">
            {/* Results header */}
            <div className="mb-6 rounded-lg border border-gray-200 bg-white p-4">
              <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <h2 className="font-semibold text-gray-900">
                    {data
                      ? `${data.pagination.total} Companies Found`
                      : 'Loading...'}
                  </h2>
                  <p className="text-sm text-gray-600">
                    Page {filters.page} of {data?.pagination.totalPages || 1}
                    {hasActiveFilters &&
                      ` • ${activeFilterCount} filter${activeFilterCount !== 1 ? 's' : ''} applied`}
                  </p>
                </div>

                <div className="flex items-center gap-2">
                  {hasActiveFilters && (
                    <button
                      onClick={resetFilters}
                      className="rounded-lg border border-gray-300 px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-50 hover:text-gray-800"
                      disabled={isLoading}
                    >
                      Clear All
                    </button>
                  )}

                  <button
                    onClick={refreshData}
                    className="flex items-center gap-1 rounded-lg bg-orange-500 px-3 py-1.5 text-sm text-white hover:bg-orange-600 disabled:opacity-50"
                    disabled={isLoading}
                  >
                    <svg
                      className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`}
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                      />
                    </svg>
                    Refresh
                  </button>
                </div>
              </div>
            </div>

            {/* Error state */}
            {error && (
              <div className="mb-6 rounded-lg border border-red-200 bg-red-50 p-4">
                <div className="flex items-center gap-2">
                  <svg
                    className="h-5 w-5 text-red-500"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  <p className="font-medium text-red-700">Error</p>
                </div>
                <p className="mt-1 text-red-600">{error}</p>
                <button
                  onClick={refreshData}
                  className="mt-2 rounded bg-red-100 px-3 py-1 text-sm text-red-700 hover:bg-red-200"
                >
                  Try Again
                </button>
              </div>
            )}

            {/* Loading state */}
            {isLoading && (
              <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                {Array.from({ length: 6 }).map((_, index) => (
                  <div
                    key={index}
                    className="animate-pulse rounded-lg border border-gray-200 bg-white p-4"
                  >
                    <div className="mb-3 flex items-center gap-3">
                      <div className="h-8 w-8 rounded-full bg-gray-200"></div>
                      <div className="flex-1">
                        <div className="mb-1 h-4 rounded bg-gray-200"></div>
                        <div className="h-3 w-3/4 rounded bg-gray-200"></div>
                      </div>
                      <div className="h-6 w-16 rounded-full bg-gray-200"></div>
                    </div>
                    <div className="mb-3 grid grid-cols-2 gap-3">
                      <div className="rounded-lg bg-gray-100 p-3">
                        <div className="mb-1 h-3 rounded bg-gray-200"></div>
                        <div className="h-4 rounded bg-gray-200"></div>
                      </div>
                      <div className="rounded-lg bg-gray-100 p-3">
                        <div className="mb-1 h-3 rounded bg-gray-200"></div>
                        <div className="h-4 rounded bg-gray-200"></div>
                      </div>
                    </div>
                    <div className="space-y-2">
                      {Array.from({ length: 4 }).map((_, i) => (
                        <div key={i} className="flex justify-between">
                          <div className="h-3 w-1/3 rounded bg-gray-200"></div>
                          <div className="h-3 w-1/4 rounded bg-gray-200"></div>
                        </div>
                      ))}
                    </div>
                    <div className="mt-4 h-10 w-full rounded-lg bg-gray-200"></div>
                  </div>
                ))}
              </div>
            )}

            {/* Results grid */}
            {!isLoading && data && data.data.length > 0 && (
              <>
                <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                  {data.data.map(ranking => (
                    <StockCard
                      key={ranking.symbol}
                      ranking={ranking}
                      rank={ranking.ranking_position || 0}
                    />
                  ))}
                </div>

                {/* Pagination */}
                <Pagination
                  pagination={data.pagination}
                  onPageChange={changePage}
                  isLoading={isLoading}
                />
              </>
            )}

            {/* Empty state */}
            {!isLoading && data && data.data.length === 0 && (
              <div className="rounded-lg border border-gray-200 bg-white p-8 text-center">
                <svg
                  className="mx-auto mb-4 h-12 w-12 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
                <h3 className="mb-2 text-lg font-medium text-gray-900">
                  No Results Found
                </h3>
                <p className="mb-4 text-gray-600">
                  Try adjusting your filters to find more companies.
                </p>
                <button
                  onClick={resetFilters}
                  className="rounded-lg bg-orange-500 px-4 py-2 text-white hover:bg-orange-600"
                >
                  Clear Filters
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
