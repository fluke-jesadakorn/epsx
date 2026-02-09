'use client';

import { useAnalyticsData } from '@/hooks/use-analytics-data';
import { useAnalyticsFilters } from '@/hooks/use-analytics-filters';
import { calculateQoQLeaders } from '@/lib/analytics/qoq-calculations';
import type { UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';
import { useMemo, useState } from 'react';

import { AnalyticsNavigation } from '@/components/shared/analytics-navigation';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { StockDataCard } from '@/shared/components/cards/stock-data-card';
import { LayoutGrid, List } from 'lucide-react';
import { AnalyticsMetadataDisplay } from './analytics-metadata-display';
import { CardDashboardView } from './card-dashboard-view';
import FilterPanel from './filter-panel';
import Pagination from './pagination';

interface RichFilterOptions {
  countries: Array<{ value: string; label: string }>;
  sectors: string[];
  exchanges?: string[];
  stock_types?: string[];
}

interface AnalyticsDashboardProps {
  /**
   * Optional initial data for SSR scenarios
   */
  initialData?: UnifiedAnalyticsRankingsResponse | null;

  /**
   * Optional filter options for SSR scenarios
   */
  filterOptions?: RichFilterOptions;

  /**
   * Whether to show the analytics navigation component
   */
  showNavigation?: boolean;
}

/**
 * Analytics Dashboard that supports both client-side fetching
 * and server-side rendering with initial data
 */
export default function AnalyticsDashboard({
  initialData = null,
  filterOptions: initialFilterOptions,
  showNavigation = false
}: AnalyticsDashboardProps) {
  const {
    filters,
    updateFilters,
    resetFilters,
    changePage,
    hasActiveFilters: _hasActiveFilters,
    activeFilterCount: _activeFilterCount,
    isLoading: filtersLoading,
    setIsLoading: setFiltersLoading,
  } = useAnalyticsFilters();

  const {
    data,
    filterOptions,
    isLoading: dataLoading,
    error,
  } = useAnalyticsData(filters);

  // Tab state
  const [activeTab, setActiveTab] = useState('list');

  // Use initial data if provided and no new data has been loaded
  const currentData = data ?? initialData;
  const currentFilterOptions = filterOptions.countries.length > 0
    ? filterOptions
    : (initialFilterOptions ?? filterOptions);

  const isLoading = filtersLoading || dataLoading;

  // Calculate QoQ leaders using the extracted utility
  const qoqLeaders = useMemo(() => calculateQoQLeaders(currentData), [currentData]);

  const handlePageChange = (newPage: number) => {
    setFiltersLoading(true);
    changePage(newPage);
    // Loading will be handled by the useAnalyticsData hook
  };

  if (error) {
    return (
      <ErrorDisplay
        error={error}
        onRetry={() => window.location.reload()}
      />
    );
  }

  return (
    <div className="container mx-auto space-y-6 p-4">
      <div className="mx-auto max-w-7xl space-y-8">

        {/* Optional Analytics Navigation */}
        {showNavigation && <AnalyticsNavigation currentPage="analytics" />}

        {/* Header Section */}
        <HeaderSection />

        {/* Filters */}
        <FilterPanel
          filters={filters}
          options={{
            countries: currentFilterOptions.countries.map((c: string | { value: string }) => typeof c === 'string' ? c : c.value),
            sectors: currentFilterOptions.sectors,
            exchanges: currentFilterOptions.exchanges,
            stock_types: currentFilterOptions.stock_types
          }}
          onFiltersChange={updateFilters}
          isLoading={isLoading}
        />

        {/* Analytics Metadata */}
        <AnalyticsMetadataDisplay data={currentData} isLoading={isLoading} />

        {/* Main Content */}
        <div className="space-y-6">
          {/* QoQ Leaders Display */}
          {qoqLeaders.epsLeaders.length > 0 && (
            <QoQLeadersDisplay qoqLeaders={qoqLeaders} isLoading={isLoading} />
          )}

          {/* Analytics Data Display */}
          <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
            <TabNavigation />

            <TabsContent value="list" className="space-y-6">
              <RankingsList
                data={currentData}
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

// Header Section Component
function HeaderSection() {
  return (
    <div className="text-center">
      <h1 className="mb-4 text-3xl font-bold tracking-tight text-gray-900 dark:text-white md:text-4xl">
        📊 Advanced EPS Analytics Dashboard
      </h1>
      <p className="text-lg text-gray-600 dark:text-gray-300">
        Discover high-growth companies with comprehensive earnings analysis
      </p>
    </div>
  );
}

// Tab Navigation Component
function TabNavigation() {
  return (
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
  );
}

// Error Display Component
interface ErrorDisplayProps {
  error: string;
  onRetry: () => void;
}

function ErrorDisplay({ error, onRetry }: ErrorDisplayProps) {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="text-center">
        <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-red-100 dark:bg-red-900/20">
          <svg
            className="h-10 w-10 text-red-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
            />
          </svg>
        </div>
        <h3 className="mb-3 text-xl font-bold text-gray-900 dark:text-white">
          Something went wrong
        </h3>
        <p className="text-red-600 dark:text-red-400 mb-6">
          {error}
        </p>
        <button
          onClick={onRetry}
          className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg hover:shadow-xl"
        >
          Try Again
        </button>
      </div>
    </div>
  );
}

// QoQ Leaders display component
interface QoQLeadersDisplayProps {
  qoqLeaders: { epsLeaders: unknown[]; priceLeaders: unknown[] };
  isLoading: boolean;
}

function QoQLeadersDisplay({ qoqLeaders, isLoading }: QoQLeadersDisplayProps) {
  if (isLoading) {
    return <QoQLeadersSkeleton />;
  }

  return (
    <div className="rounded-2xl border border-gray-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
      <h2 className="mb-4 text-xl font-bold text-gray-900 dark:text-white">
        🏆 Quarter-over-Quarter Leaders
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <LeaderCategory
          title="Top EPS Growth"
          leaders={qoqLeaders.epsLeaders.slice(0, 3)}
          colorScheme="green"
          valueKey="analytics.growth_factor"
        />

        <LeaderCategory
          title="Top Price Growth"
          leaders={qoqLeaders.priceLeaders.slice(0, 3)}
          colorScheme="blue"
          valueKey="quarterly_data"
        />
      </div>
    </div>
  );
}

// Leader Category Component
interface LeaderCategoryProps {
  title: string;
  leaders: unknown[];
  colorScheme: 'green' | 'blue';
  valueKey: string;
}

function LeaderCategory({ title, leaders, colorScheme, valueKey }: LeaderCategoryProps) {
  const colorClasses = colorScheme === 'green'
    ? {
      bg: 'bg-green-50 dark:bg-green-900/20',
      text: 'text-green-600 dark:text-green-400'
    }
    : {
      bg: 'bg-blue-50 dark:bg-blue-900/20',
      text: 'text-blue-600 dark:text-blue-400'
    };

  return (
    <div>
      <h3 className={`mb-3 font-semibold ${colorClasses.text}`}>
        {title}
      </h3>
      <div className="space-y-2">
        {leaders.map((leaderItem) => {
          const leader = leaderItem as Record<string, unknown>;
          let displayValue = '0.0%';

          if (valueKey === 'analytics.growth_factor') {
            const analytics = leader.analytics as Record<string, unknown> | undefined;
            const growthFactor = analytics?.growth_factor as number | undefined;
            displayValue = `${growthFactor?.toFixed(1) ?? 0}%`;
          } else if (valueKey === 'quarterly_data') {
            const quarterlyData = leader.quarterly_data as Array<Record<string, unknown>> | undefined;
            const latestGrowth = (quarterlyData?.[0]?.price_growth as number) || 0;
            const previousGrowth = (quarterlyData?.[1]?.price_growth as number) || 0;
            const growth = latestGrowth === 0 ? previousGrowth : latestGrowth;
            displayValue = `${growth?.toFixed(1) || 0}%`;
          }

          return (
            <div key={leader.symbol as string} className={`flex items-center justify-between p-2 rounded-lg ${colorClasses.bg}`}>
              <span className="font-medium">{leader.symbol as string}</span>
              <span className={`${colorClasses.text} font-bold`}>
                {displayValue}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// QoQ Leaders Skeleton
function QoQLeadersSkeleton() {
  return (
    <div className="rounded-2xl border border-gray-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
      <div className="animate-pulse">
        <div className="h-6 bg-gray-300 rounded mb-4" />
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="h-32 bg-gray-300 rounded" />
          <div className="h-32 bg-gray-300 rounded" />
        </div>
      </div>
    </div>
  );
}

// Rankings list component
interface StockRanking {
  symbol: string;
  rank?: number;
  epsGrowth?: number;
  price?: number;
  price_current?: number;
  current_price?: number;
  name?: string;
  companyName?: string;
  [key: string]: unknown;
}

interface RankingsListProps {
  data: UnifiedAnalyticsRankingsResponse | null;
  isLoading: boolean;
  onPageChange: (_page: number) => void;
  onReset: () => void;
}

function RankingsList({ data, isLoading, onPageChange, onReset }: RankingsListProps) {
  if (isLoading) {
    return <RankingsListSkeleton />;
  }

  if (!data?.rankings || data.rankings.length === 0) {
    return <NoResultsState onReset={onReset} />;
  }

  return (
    <div className="space-y-6">
      {/* Mobile: Horizontal scroll layout */}
      <div className="sm:hidden">
        <div className="overflow-x-auto">
          <div className="flex gap-4 pb-4" style={{ width: 'max-content' }}>
            {data.rankings.map((ranking: StockRanking, index: number) => (
              <div key={ranking.symbol} className="w-80 flex-shrink-0">
                <StockDataCard
                  symbol={ranking.symbol}
                  rank={(ranking.rank) || index + 1}
                  epsGrowth={(ranking.epsGrowth) || 0}
                  price={ranking.price ?? ranking.price_current ?? ranking.current_price ?? 0}
                  currency="USD"
                  companyName={ranking.name ?? ranking.companyName}
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
        {data.rankings.map((ranking: StockRanking) => (
          <StockDataCard
            key={ranking.symbol}
            symbol={ranking.symbol}
            rank={(ranking.rank) || 0}
            epsGrowth={(ranking.epsGrowth) || 0}
            price={ranking.price ?? ranking.price_current ?? ranking.current_price ?? 0}
            currency="USD"
            companyName={ranking.name ?? ranking.companyName}
          />
        ))}
      </div>

      {/* Pagination */}
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
          onPageChange={onPageChange}
          isLoading={isLoading}
        />
      </div>
    </div>
  );
}

// Rankings List Skeleton
function RankingsListSkeleton() {
  return (
    <div className="space-y-4">
      {[...Array(10)].map((_, i) => (
        <div key={`skeleton-${String(i)}`} className="animate-pulse rounded-2xl border border-gray-200/50 bg-white/80 p-6 backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
          <div className="h-20 bg-gray-300 rounded" />
        </div>
      ))}
    </div>
  );
}

// No Results State
interface NoResultsStateProps {
  onReset: () => void;
}

function NoResultsState({ onReset }: NoResultsStateProps) {
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