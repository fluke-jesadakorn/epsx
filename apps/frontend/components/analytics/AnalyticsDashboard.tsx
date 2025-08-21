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

  // Helper function to calculate QoQ leaders
  const calculateQoQLeaders = (data: EPSRankingsResponse | null) => {
    if (!data?.data || data.data.length === 0) return { epsLeaders: [], priceLeaders: [] };

    // Filter companies with QoQ data
    const companiesWithQoQ = data.data.filter(ranking => 
      ranking.quarterly_data && ranking.quarterly_data.length >= 2
    );

    // Calculate EPS QoQ leaders (highest EPS growth)
    const epsLeaders = companiesWithQoQ
      .filter(ranking => ranking.qoq_growth !== null && ranking.qoq_growth !== undefined)
      .sort((a, b) => (b.qoq_growth || 0) - (a.qoq_growth || 0))
      .slice(0, 3);

    // Calculate Price QoQ leaders (highest price growth from latest quarter)
    const priceLeaders = companiesWithQoQ
      .filter(ranking => {
        const latestQuarter = ranking.quarterly_data?.[0];
        return latestQuarter?.price_growth !== null && latestQuarter?.price_growth !== undefined;
      })
      .sort((a, b) => {
        const aGrowth = a.quarterly_data?.[0]?.price_growth || 0;
        const bGrowth = b.quarterly_data?.[0]?.price_growth || 0;
        return bGrowth - aGrowth;
      })
      .slice(0, 3);

    return { epsLeaders, priceLeaders };
  };

  const [data, setData] = useState<EPSRankingsResponse | null>(null);
  const [filterOptions, setFilterOptions] = useState<FilterOptions>({
    countries: [],
    sectors: [],
  });
  const [error, setError] = useState<string | null>(null);
  const [showMobileFilters, setShowMobileFilters] = useState(false);

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
            Quarter-over-Quarter (QoQ) Analytics
          </h1>
          <p className="text-gray-600">
            Real-time EPS and price QoQ growth comparison analysis
          </p>
        </div>

        {/* Mobile Filter Toggle */}
        <div className="mb-4 lg:hidden">
          <button
            onClick={() => setShowMobileFilters(!showMobileFilters)}
            className="flex w-full items-center justify-between rounded-lg border border-gray-200 bg-white p-3 text-sm font-medium hover:bg-gray-50"
          >
            <span className="flex items-center gap-2">
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
              </svg>
              Filters & Search
              {hasActiveFilters && (
                <span className="rounded-full bg-orange-500 px-2 py-0.5 text-xs text-white">
                  {activeFilterCount}
                </span>
              )}
            </span>
            <svg
              className={`h-4 w-4 transition-transform ${showMobileFilters ? 'rotate-180' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
        </div>

        {/* Mobile Filters Panel */}
        {showMobileFilters && (
          <div className="mb-6 lg:hidden">
            <div className="rounded-lg border border-gray-200 bg-white p-4">
              <FilterPanel
                filters={filters}
                options={filterOptions}
                onFiltersChange={updateFilters}
                isLoading={isLoading}
                isMobile={true}
              />
            </div>
          </div>
        )}

        {/* QoQ Leaders Section */}
        {!isLoading && data && data.data.length > 0 && (() => {
          const { epsLeaders, priceLeaders } = calculateQoQLeaders(data);
          return (
            <div className="mb-6">
              <div className="rounded-lg border border-orange-200 bg-gradient-to-r from-orange-50 to-yellow-50 p-4 sm:p-6">
                <div className="mb-4">
                  <h2 className="text-lg font-bold text-orange-900 sm:text-xl">
                    🏆 QoQ Performance Leaders
                  </h2>
                  <p className="text-sm text-orange-700">
                    Top performers in EPS and price quarter-over-quarter growth
                  </p>
                </div>

                <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                  {/* EPS QoQ Leaders */}
                  <div className="rounded-lg bg-white p-4 shadow-sm">
                    <h3 className="mb-3 flex items-center gap-2 text-sm font-semibold text-green-700">
                      📈 Best EPS QoQ Growth
                    </h3>
                    <div className="space-y-2">
                      {epsLeaders.slice(0, 3).map((leader, index) => (
                        <div key={leader.symbol} className="flex items-center justify-between py-1">
                          <div className="flex items-center gap-2">
                            <span className="flex h-6 w-6 items-center justify-center rounded-full bg-green-100 text-xs font-bold text-green-700">
                              {index + 1}
                            </span>
                            <span className="font-medium text-gray-900">{leader.symbol}</span>
                          </div>
                          <span className="font-bold text-green-600">
                            +{(leader.qoq_growth || 0).toFixed(1)}%
                          </span>
                        </div>
                      ))}
                      {epsLeaders.length === 0 && (
                        <p className="text-xs text-gray-500">No EPS growth data available</p>
                      )}
                    </div>
                  </div>

                  {/* Price QoQ Leaders */}
                  <div className="rounded-lg bg-white p-4 shadow-sm">
                    <h3 className="mb-3 flex items-center gap-2 text-sm font-semibold text-blue-700">
                      💰 Best Price QoQ Growth
                    </h3>
                    <div className="space-y-2">
                      {priceLeaders.slice(0, 3).map((leader, index) => {
                        const priceGrowth = leader.quarterly_data?.[0]?.price_growth || 0;
                        return (
                          <div key={leader.symbol} className="flex items-center justify-between py-1">
                            <div className="flex items-center gap-2">
                              <span className="flex h-6 w-6 items-center justify-center rounded-full bg-blue-100 text-xs font-bold text-blue-700">
                                {index + 1}
                              </span>
                              <span className="font-medium text-gray-900">{leader.symbol}</span>
                            </div>
                            <span className="font-bold text-blue-600">
                              {priceGrowth >= 0 ? '+' : ''}{priceGrowth.toFixed(1)}%
                            </span>
                          </div>
                        );
                      })}
                      {priceLeaders.length === 0 && (
                        <p className="text-xs text-gray-500">No price growth data available</p>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          );
        })()}

        {/* Layout: Responsive grid */}
        <div className="lg:grid lg:grid-cols-4 lg:gap-6">
          {/* Desktop Filters sidebar */}
          <div className="hidden lg:block lg:col-span-1">
            <div className="sticky top-4">
              <FilterPanel
                filters={filters}
                options={filterOptions}
                onFiltersChange={updateFilters}
                isLoading={isLoading}
                isMobile={false}
              />
            </div>
          </div>

          {/* Main content */}
          <div className="lg:col-span-3">
            {/* Mobile-optimized Results header */}
            <div className="mb-4 sm:mb-6 rounded-lg border border-gray-200 bg-white p-3 sm:p-4">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <h2 className="text-sm sm:text-base font-semibold text-gray-900">
                    {data
                      ? `${data.pagination.total} Companies Found`
                      : 'Loading...'}
                  </h2>
                  <p className="text-xs sm:text-sm text-gray-600 mt-1">
                    Page {filters.page} of {data?.pagination.totalPages || 1}
                    {hasActiveFilters &&
                      ` • ${activeFilterCount} filter${activeFilterCount !== 1 ? 's' : ''} applied`}
                  </p>
                </div>

                <div className="flex items-center gap-2 w-full sm:w-auto justify-between sm:justify-end">
                  {hasActiveFilters && (
                    <button
                      onClick={resetFilters}
                      className="rounded-lg border border-gray-300 px-2 sm:px-3 py-1.5 text-xs sm:text-sm text-gray-600 hover:bg-gray-50 hover:text-gray-800 flex-1 sm:flex-none"
                      disabled={isLoading}
                    >
                      Clear All
                    </button>
                  )}

                  <button
                    onClick={refreshData}
                    className="flex items-center justify-center gap-1 rounded-lg bg-orange-500 px-3 py-1.5 text-xs sm:text-sm text-white hover:bg-orange-600 disabled:opacity-50 flex-1 sm:flex-none min-w-0"
                    disabled={isLoading}
                  >
                    <svg
                      className={`h-3 w-3 sm:h-4 sm:w-4 ${isLoading ? 'animate-spin' : ''}`}
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
                    <span className="hidden sm:inline">Refresh</span>
                    <span className="sm:hidden">↻</span>
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

            {/* Loading state - Mobile optimized */}
            {isLoading && (
              <>
                {/* Mobile: Horizontal scroll skeleton */}
                <div className="mb-6 block sm:hidden">
                  <div className="overflow-x-auto pb-4">
                    <div className="flex gap-3">
                      {Array.from({ length: 4 }).map((_, index) => (
                        <div
                          key={index}
                          className="w-72 flex-shrink-0 animate-pulse rounded-lg border border-gray-200 bg-white p-4"
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
                          <div className="mt-4 h-10 w-full rounded-lg bg-gray-200"></div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Desktop: Grid skeleton */}
                <div className="mb-6 hidden sm:grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
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
              </>
            )}

            {/* Results grid - Responsive */}
            {!isLoading && data && data.data.length > 0 && (
              <>
                {/* Mobile: Horizontal scrolling cards */}
                <div className="mb-6 block sm:hidden">
                  <div className="overflow-x-auto pb-4">
                    <div className="flex gap-3">
                      {data.data.map((ranking, index) => (
                        <div key={ranking.symbol} className="w-72 flex-shrink-0">
                          <StockCard
                            ranking={ranking}
                            rank={ranking.ranking_position || index + 1}
                          />
                        </div>
                      ))}
                    </div>
                    {/* Scroll indicator */}
                    <div className="mt-4 flex justify-center">
                      <p className="text-xs text-gray-500">
                        👈 Swipe to see more stocks →
                      </p>
                    </div>
                  </div>
                </div>

                {/* Desktop: Grid layout */}
                <div className="mb-6 hidden sm:grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                  {data.data.map(ranking => (
                    <StockCard
                      key={ranking.symbol}
                      ranking={ranking}
                      rank={ranking.ranking_position || 0}
                    />
                  ))}
                </div>

                {/* Mobile-optimized Pagination */}
                <div className="px-2 sm:px-0">
                  <Pagination
                    pagination={data.pagination}
                    onPageChange={changePage}
                    isLoading={isLoading}
                  />
                </div>
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
