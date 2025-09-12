'use client';

import { useAnalyticsFilters } from '@/hooks/useAnalyticsFilters';
import { useAnalyticsData } from '@/hooks/useAnalyticsData';
import { calculateQoQLeaders } from '@/lib/analytics/qoq-calculations';
import { useMemo } from 'react';

import { AnalyticsMetadataDisplay } from './AnalyticsMetadataDisplay';
import FilterPanel from './FilterPanel';
import Pagination from './Pagination';
import StockCard from './StockCard';
import { CardDashboardView } from './CardDashboardView';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { LayoutGrid, List } from 'lucide-react';

export default function AnalyticsDashboardRefactored() {
  const {
    filters,
    updateFilters,
    resetFilters,
    changePage,
    hasActiveFilters,
    activeFilterCount,
    isLoading: filtersLoading,
    setIsLoading: setFiltersLoading,
  } = useAnalyticsFilters();

  const {
    data,
    filterOptions,
    isLoading: dataLoading,
    error,
  } = useAnalyticsData(filters);

  const isLoading = filtersLoading || dataLoading;

  // Calculate QoQ leaders using the extracted utility
  const qoqLeaders = useMemo(() => calculateQoQLeaders(data), [data]);

  const handlePageChange = (newPage: number) => {
    setFiltersLoading(true);
    changePage(newPage);
    // Loading will be handled by the useAnalyticsData hook
  };

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="text-center">
          <p className="text-red-600 dark:text-red-400 mb-4">
            Error loading analytics data: {error}
          </p>
          <button
            onClick={() => window.location.reload()}
            className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg hover:shadow-xl"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto space-y-6 p-4">
      <div className="mx-auto max-w-7xl space-y-8">
        
        {/* Header Section */}
        <div className="text-center">
          <h1 className="mb-4 text-3xl font-bold tracking-tight text-gray-900 dark:text-white md:text-4xl">
            📊 Advanced EPS Analytics Dashboard
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-300">
            Discover high-growth companies with comprehensive earnings analysis
          </p>
        </div>

        {/* Filters */}
        <FilterPanel
          filters={filters}
          onFiltersChange={updateFilters}
          onReset={resetFilters}
          filterOptions={filterOptions}
          hasActiveFilters={hasActiveFilters}
          activeFilterCount={activeFilterCount}
          isLoading={isLoading}
        />

        {/* Analytics Metadata */}
        <AnalyticsMetadataDisplay data={data} isLoading={isLoading} />

        {/* Main Content */}
        <div className="space-y-6">
          {/* QoQ Leaders Display */}
          {qoqLeaders.epsLeaders.length > 0 && (
            <QoQLeadersDisplay qoqLeaders={qoqLeaders} isLoading={isLoading} />
          )}

          {/* Analytics Data Display */}
          <Tabs defaultValue="list" className="space-y-6">
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

            <TabsContent value="list" className="space-y-6">
              <RankingsList 
                data={data} 
                isLoading={isLoading} 
                onPageChange={handlePageChange}
                onReset={resetFilters}
              />
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

// QoQ Leaders display component
interface QoQLeadersDisplayProps {
  qoqLeaders: { epsLeaders: any[]; priceLeaders: any[] };
  isLoading: boolean;
}

function QoQLeadersDisplay({ qoqLeaders, isLoading }: QoQLeadersDisplayProps) {
  if (isLoading) {
    return (
      <div className="rounded-2xl border border-gray-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
        <div className="animate-pulse">
          <div className="h-6 bg-gray-300 rounded mb-4"></div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="h-32 bg-gray-300 rounded"></div>
            <div className="h-32 bg-gray-300 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-2xl border border-gray-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
      <h2 className="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        🏆 Quarter-over-Quarter Leaders
      </h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <h3 className="mb-3 font-semibold text-green-600 dark:text-green-400">
            Top EPS Growth
          </h3>
          <div className="space-y-2">
            {qoqLeaders.epsLeaders.slice(0, 3).map((leader, index) => (
              <div key={leader.symbol} className="flex items-center justify-between p-2 rounded-lg bg-green-50 dark:bg-green-900/20">
                <span className="font-medium">{leader.symbol}</span>
                <span className="text-green-600 dark:text-green-400 font-bold">
                  {leader.analytics.growth_factor?.toFixed(1)}%
                </span>
              </div>
            ))}
          </div>
        </div>

        <div>
          <h3 className="mb-3 font-semibold text-blue-600 dark:text-blue-400">
            Top Price Growth
          </h3>
          <div className="space-y-2">
            {qoqLeaders.priceLeaders.slice(0, 3).map((leader, index) => {
              const latestGrowth = leader.quarterly_data?.[0]?.price_growth || 0;
              const previousGrowth = leader.quarterly_data?.[1]?.price_growth || 0;
              const displayGrowth = latestGrowth === 0 ? previousGrowth : latestGrowth;
              
              return (
                <div key={leader.symbol} className="flex items-center justify-between p-2 rounded-lg bg-blue-50 dark:bg-blue-900/20">
                  <span className="font-medium">{leader.symbol}</span>
                  <span className="text-blue-600 dark:text-blue-400 font-bold">
                    {displayGrowth?.toFixed(1)}%
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

// Rankings list component
interface RankingsListProps {
  data: any;
  isLoading: boolean;
  onPageChange: (page: number) => void;
  onReset: () => void;
}

function RankingsList({ data, isLoading, onPageChange, onReset }: RankingsListProps) {
  if (isLoading) {
    return (
      <div className="space-y-4">
        {[...Array(10)].map((_, i) => (
          <div key={i} className="animate-pulse rounded-2xl border border-gray-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
            <div className="h-20 bg-gray-300 rounded"></div>
          </div>
        ))}
      </div>
    );
  }

  if (!data || !data.data || data.data.length === 0) {
    return (
      <div className="rounded-2xl border border-gray-200/50 bg-white/80 p-8 text-center backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
        <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600">
          <svg
            className="h-10 w-10 text-gray-400 dark:text-gray-500"
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
        </div>
        <h3 className="mb-3 bg-gradient-to-r from-gray-600 to-gray-800 bg-clip-text text-xl font-bold text-transparent dark:from-gray-300 dark:to-gray-100">
          No Results Found
        </h3>
        <p className="mb-6 max-w-md mx-auto text-gray-600 dark:text-gray-300">
          We couldn't find any companies matching your current filter criteria. Try adjusting your filters to discover more analytics data.
        </p>
        <button
          onClick={onReset}
          className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg hover:shadow-xl"
        >
          Clear All Filters
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Mobile: Horizontal scroll layout */}
      <div className="sm:hidden">
        <div className="overflow-x-auto">
          <div className="flex gap-4 pb-4" style={{ width: 'max-content' }}>
            {data.data.map((ranking: any, index: number) => (
              <div key={ranking.symbol} className="w-80 flex-shrink-0">
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
        {data.data.map((ranking: any) => (
          <StockCard
            key={ranking.symbol}
            ranking={ranking}
            rank={ranking.ranking_position || 0}
          />
        ))}
      </div>

      {/* Pagination */}
      <div className="px-2 sm:px-0">
        <Pagination
          pagination={data.pagination}
          onPageChange={onPageChange}
          isLoading={isLoading}
        />
      </div>
    </div>
  );
}