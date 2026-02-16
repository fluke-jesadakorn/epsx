 
'use client';

import { AnalyticsNavigation } from '@/components/shared/analytics-navigation';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useAnalyticsFilters } from '@/hooks/use-analytics-filters';
import type { UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';
import { LayoutGrid, List } from 'lucide-react';
import { memo, useEffect, useState } from 'react';
import { AnalyticsExportDialog } from './analytics-export-dialog';
import {
  EmptyState,
  ErrorState,
  GrowthLeadersSection,
  LoadingState,
  MetadataSection,
  RankingsList
} from './analytics-wrapper-sections';
import { CardDashboardView } from './card-dashboard-view';
import FilterPanel from './filter-panel';
import { fetchEPSRankings, useGrowthLeaders } from './hooks/use-analytics-wrapper-data';
import Pagination from './pagination';

interface RichFilterOptions {
  countries: Array<{ value: string; label: string }>;
  sectors: string[];
  exchanges?: string[];
  stock_types?: string[];
}

interface AnalyticsClientWrapperProps {
  initialData: UnifiedAnalyticsRankingsResponse | null;
  filterOptions: RichFilterOptions;
}
 
function AnalyticsClientWrapper({
  initialData,
  filterOptions
}: AnalyticsClientWrapperProps) {
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

  const [data, setData] = useState<UnifiedAnalyticsRankingsResponse | null>(initialData);
  const [error, setError] = useState<string | null>(null);
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [activeTab, setActiveTab] = useState('list');

  const { growthLeaders, priceLeaders } = useGrowthLeaders(data);

  // Load data when filters change (skip on initial load if we have initialData)
  useEffect(() => {
    // Only fetch if filters have changed from default
    const hasNonDefaultFilters = filters.page !== 1 ||
      filters.country !== undefined ||
      filters.sector !== undefined ||
      filters.min_eps !== undefined ||
      filters.min_growth !== undefined;

    if (!initialData || hasNonDefaultFilters) {
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
        } catch (_err) {
          setError('An error occurred while loading data');
        } finally {
          setIsLoading(false);
        }
      };

      void loadData();
    }
  }, [filters, initialData, setIsLoading]);

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
    } catch {
      setError('Failed to refresh data');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="animate-bounce-slow absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="animate-float absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="animate-pulse-gentle absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
        <div className="animate-float-reverse absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]"
          style={{ animationDelay: '2s' }}
        />

        {/* Decorative geometric shapes */}
        <div className="animate-spin-slow absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
        <div className="animate-bounce-gentle absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
      </div>

      <div className="relative z-10">
        <div className="container mx-auto px-4 py-6">
          {/* Enhanced Header with gradient text */}
          <div className="mb-6 text-center">
            <h1 className="animate-gradient-x mb-4 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-3xl font-bold text-transparent sm:text-4xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
              <span className="bg-gradient-to-r from-purple-500 to-purple-600 bg-clip-text text-transparent">
                Growth Factor
              </span>{' '}
              <span className="bg-gradient-to-r from-blue-500 to-blue-600 bg-clip-text text-transparent">
                Analytics
              </span>
            </h1>
            <p className="mx-auto max-w-2xl text-lg text-gray-600 dark:text-gray-300">
              Stateless HTTP analytics with server-side data fetching and advanced filtering
            </p>
            <AnalyticsNavigation currentPage="analytics" />
            {/* Decorative elements */}
            <div className="mt-4 flex items-center justify-center gap-4">
              <div className="h-2 w-2 animate-pulse rounded-full bg-orange-400" />
              <div
                className="h-3 w-3 animate-pulse rounded-full bg-purple-400"
                style={{ animationDelay: '0.5s' }}
              />
              <div
                className="h-2 w-2 animate-pulse rounded-full bg-blue-400"
                style={{ animationDelay: '1s' }}
              />
            </div>
          </div>

          {/* Enhanced Mobile Filter Toggle */}
          <div className="mb-4 lg:hidden">
            <button
              onClick={() => setShowMobileFilters(!showMobileFilters)}
              className="flex w-full items-center justify-between rounded-2xl border border-orange-200/50 bg-white/80 p-4 text-sm font-medium backdrop-blur-md transition-all duration-300 hover:bg-white/90 hover:scale-[1.02] dark:border-orange-400/20 dark:bg-slate-800/80 dark:hover:bg-slate-800/90"
            >
              <span className="flex items-center gap-3">
                <div className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                  </svg>
                </div>
                <span className="font-semibold text-gray-900 dark:text-gray-100">Filters & Search</span>
                {hasActiveFilters && (
                  <span className="rounded-full bg-gradient-to-r from-orange-500 to-red-500 px-3 py-1 text-xs font-bold text-white shadow-lg">
                    {activeFilterCount}
                  </span>
                )}
              </span>
              <svg
                className={`h-5 w-5 transition-transform duration-300 ${showMobileFilters ? 'rotate-180' : ''} text-gray-500 dark:text-gray-400`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
              </svg>
            </button>
          </div>

          {/* Enhanced Mobile Filters Panel */}
          {showMobileFilters && (
            <div className="mb-6 lg:hidden">
              <div className="rounded-2xl border border-orange-200/50 bg-white/80 p-6 backdrop-blur-md dark:border-orange-400/20 dark:bg-slate-800/80">
                <FilterPanel
                  filters={filters}
                  options={{
                    countries: filterOptions.countries.map(c => typeof c === 'string' ? { value: c, label: c } : c),
                    sectors: filterOptions.sectors,
                    exchanges: filterOptions.exchanges,
                    stock_types: filterOptions.stock_types
                  }}
                  onFiltersChange={updateFilters}
                  isLoading={isLoading}
                />
              </div>
            </div>
          )}

          {!isLoading && data && data.rankings.length > 0 && (
            <GrowthLeadersSection growthLeaders={growthLeaders} priceLeaders={priceLeaders} />
          )}

          {data?.metadata && (
            <MetadataSection metadata={data.metadata} />
          )}

          {/* Enhanced Layout with Tabbed Views */}
          <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
            <div className="flex justify-center">
              <TabsList className="grid w-full max-w-md grid-cols-2 rounded-2xl bg-white/80 backdrop-blur-xl border border-orange-200/50 dark:bg-slate-800/80 dark:border-orange-400/20">
                <TabsTrigger
                  value="list"
                  className="flex items-center gap-2 data-[state=active]:bg-gradient-to-r data-[state=active]:from-orange-500 data-[state=active]:to-yellow-500 data-[state=active]:text-white"
                >
                  <List className="h-4 w-4" />
                  List View
                </TabsTrigger>
                <TabsTrigger
                  value="cards"
                  className="flex items-center gap-2 data-[state=active]:bg-gradient-to-r data-[state=active]:from-orange-500 data-[state=active]:to-yellow-500 data-[state=active]:text-white"
                >
                  <LayoutGrid className="h-4 w-4" />
                  Card View
                </TabsTrigger>
              </TabsList>
            </div>

            <TabsContent value="list" className="space-y-0">
              <div className="lg:grid lg:grid-cols-4 lg:gap-8">
                {/* Enhanced Desktop Filters sidebar */}
                <div className="hidden lg:block lg:col-span-1">
                  <div className="sticky top-4">
                    <div className="rounded-2xl border border-orange-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
                      <FilterPanel
                        filters={filters}
                        options={{
                          countries: filterOptions.countries.map(c => typeof c === 'string' ? { value: c, label: c } : c),
                          sectors: filterOptions.sectors,
                          exchanges: filterOptions.exchanges,
                          stock_types: filterOptions.stock_types
                        }}
                        onFiltersChange={updateFilters}
                        isLoading={isLoading}
                      />
                    </div>
                  </div>
                </div>

                {/* Enhanced Main content */}
                <div className="lg:col-span-3">
                  {/* Enhanced Results header */}
                  <div className="mb-6 rounded-2xl border border-orange-200/50 bg-white/80 p-4 sm:p-6 backdrop-blur-xl shadow-lg dark:border-orange-400/20 dark:bg-slate-800/80">
                    <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                      <div className="flex-1">
                        <h2 className="mb-2 bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-lg font-bold text-transparent sm:text-xl">
                          {data
                            ? `${data.pagination.total_items} Companies Found`
                            : 'Loading Analytics...'}
                        </h2>
                        <div className="flex flex-wrap items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
                          <span className="flex items-center gap-1">
                            <div className="h-2 w-2 rounded-full bg-blue-400" />
                            Page {filters.page} of {data?.pagination.total_pages ?? 1}
                          </span>
                          {hasActiveFilters && (
                            <span className="flex items-center gap-1">
                              <div className="h-2 w-2 rounded-full bg-orange-400" />
                              {activeFilterCount} filter{activeFilterCount !== 1 ? 's' : ''} applied
                            </span>
                          )}
                        </div>
                      </div>

                      <div className="flex items-center gap-3">
                        {hasActiveFilters && (
                          <button
                            onClick={resetFilters}
                            className="rounded-xl border border-gray-300 bg-white/60 px-4 py-2 text-sm font-medium text-gray-700 backdrop-blur-sm transition-all duration-300 hover:bg-white/80 hover:scale-105 disabled:opacity-50 dark:border-gray-600 dark:bg-slate-700/60 dark:text-gray-300 dark:hover:bg-slate-700/80"
                            disabled={isLoading}
                          >
                            Clear All
                          </button>
                        )}

                        <button
                          onClick={refreshData}
                          className="flex items-center gap-2 rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-4 py-2 text-sm font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105 disabled:opacity-50"
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
                          <span className="hidden sm:inline">Refresh Data</span>
                          <span className="sm:hidden">Refresh</span>
                        </button>

                        <AnalyticsExportDialog
                          data={data}
                          isLoading={isLoading}
                          filters={filters}
                          growthLeaders={growthLeaders}
                          priceLeaders={priceLeaders}
                        />
                      </div>
                    </div>
                  </div>

                  {error && (
                    <ErrorState error={error} onRetry={refreshData} />
                  )}

                  {isLoading && <LoadingState />}

                  {!isLoading && data && data.rankings.length > 0 && (
                    <>
                      <RankingsList data={data} />
                      <div className="px-2 sm:px-0">
                        <Pagination
                          pagination={{
                            page: data.pagination.page,
                            limit: data.pagination.per_page,
                            total: data.pagination.total_items,
                            totalPages: data.pagination.total_pages,
                            hasNext: data.pagination.page < data.pagination.total_pages,
                            hasPrev: data.pagination.page > 1
                          }}
                          onPageChange={changePage}
                          onLimitChange={(limit) => updateFilters({ limit })}
                          isLoading={isLoading}
                        />
                      </div>
                    </>
                  )}

                  {!isLoading && data?.rankings.length === 0 && (
                    <EmptyState onReset={resetFilters} />
                  )}
                </div>
              </div>
            </TabsContent>

            <TabsContent value="cards" className="space-y-0">
              <CardDashboardView />
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  );
}

// Export memoized component to prevent unnecessary re-renders
export default memo(AnalyticsClientWrapper);