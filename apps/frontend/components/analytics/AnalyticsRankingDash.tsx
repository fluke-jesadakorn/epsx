'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { useAnalyticsFilters } from '@/hooks/useAnalyticsFilters';
import { formatDate, formatPrice } from '@/utils/fmt';
import {
  Activity,
  AlertCircle,
  BarChart3,
  Clock,
  Crown,
  Download,
  Globe,
  RefreshCw,
  Server,
  Zap,
} from 'lucide-react';
import { ActiveFilterIndicators } from './ActiveFilterIndicators';
import { AdvancedFilterPanel } from './AdvancedFilterPanel';
import { Pagination } from './Pagination';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  package_tier: string;
}

interface AnalyticsRankingDashboardProps {
  className?: string;
  user?: AuthUser | null;
}

export function AnalyticsRankingDashboard({
  className,
  user,
}: AnalyticsRankingDashboardProps) {
  const { toast } = useToast();

  // Replace useRankingAccess hook with inline logic
  const hasRankingAccess = user?.permissions?.includes('ranking:read') || false;
  const accessLoading = false; // No loading state needed since user is passed as prop

  const {
    filters,
    data,
    options,
    loading,
    error,
    updateFilter,
    updateFilters,
    resetFilters,
    goToPage,
    changePageSize,
    currentPage,
    setCountry,
    setSector,
    applyPreset,
    refresh,
  } = useAnalyticsFilters();

  // Get data freshness indicator
  const getDataFreshness = () => {
    if (!data?.metadata.request_timestamp) return null;

    const dataTime = new Date(data.metadata.request_timestamp);
    const now = new Date();
    const diffMs = now.getTime() - dataTime.getTime();
    const diffMinutes = Math.floor(diffMs / 60000);

    if (diffMinutes < 1) return 'Just now';
    if (diffMinutes < 60) return `${diffMinutes}m ago`;

    const diffHours = Math.floor(diffMinutes / 60);
    return `${diffHours}h ago`;
  };

  // Get data source badge color
  const getDataSourceColor = (dataSource: string) => {
    if (dataSource.includes('live_cache')) return 'bg-green-500';
    if (dataSource.includes('websocket')) return 'bg-blue-500';
    return 'bg-gray-500';
  };

  // Get rank badge style
  const getRankBadgeStyle = (rank: number) => {
    if (rank === 1) return 'bg-yellow-500 text-yellow-900'; // Gold
    if (rank === 2) return 'bg-gray-400 text-gray-900'; // Silver
    if (rank === 3) return 'bg-amber-600 text-amber-100'; // Bronze
    if (rank <= 10) return 'bg-green-500 text-white'; // Top 10
    if (rank <= 25) return 'bg-blue-500 text-white'; // Top 25
    return 'bg-gray-500 text-white'; // Others
  };

  // Get performance index color
  const getIndexColor = (index: number | null | undefined) => {
    if (index == null) return 'text-gray-600';
    if (index >= 80) return 'text-green-600';
    if (index >= 60) return 'text-blue-600';
    if (index >= 40) return 'text-yellow-600';
    return 'text-red-600';
  };

  // Get growth color
  const getGrowthColor = (growth: number | null | undefined) => {
    if (growth == null) return 'text-gray-600';
    if (growth > 0) return 'text-green-600';
    if (growth < 0) return 'text-red-600';
    return 'text-gray-600';
  };

  const handleRefresh = async () => {
    try {
      refresh();
      toast({
        title: 'Data Refreshed',
        description: 'Analytics data has been updated successfully',
        variant: 'default',
      });
    } catch (error) {
      toast({
        title: 'Refresh Failed',
        description: `Error: ${error instanceof Error ? error.message : String(error)}`,
        variant: 'destructive',
      });
    }
  };

  const handleExport = async () => {
    // Placeholder for export functionality
    toast({
      title: 'Export Started',
      description: 'Your filtered data export is being prepared',
      variant: 'default',
    });
  };

  // Handle preset application
  const handlePresetApply = (presetName: string) => {
    applyPreset(presetName as any);
    toast({
      title: 'Filter Preset Applied',
      description: `Applied ${presetName} filter preset`,
      variant: 'default',
    });
  };

  if (accessLoading || (loading && !data)) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="animate-pulse">
          <div className="mb-4 h-8 w-1/3 rounded bg-gray-200"></div>
          <div className="mb-8 h-4 w-2/3 rounded bg-gray-200"></div>
          <div className="grid gap-6 md:grid-cols-3 lg:grid-cols-4">
            {Array.from({ length: 12 }).map((_, i) => (
              <div key={i} className="h-64 rounded bg-gray-200"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="space-y-4 text-center">
        <div className="flex items-center justify-center gap-3">
          <Zap className="h-10 w-10 text-green-600" />
          <h1 className="text-4xl font-bold">Enhanced EPS Rankings</h1>
        </div>
        <p className="text-muted-foreground text-xl">
          Advanced analytics dashboard with powerful filtering and page-based
          pagination
        </p>

        {/* Data Source & Freshness Info */}
        {data && (
          <div className="flex flex-wrap justify-center gap-4">
            <Badge
              className={`${getDataSourceColor(data.metadata.data_source)} text-white`}
            >
              <Activity className="mr-1 h-3 w-3" />
              {data.metadata.data_source.replace('_', ' ').toUpperCase()}
            </Badge>
            <Badge variant="outline">
              <Clock className="mr-1 h-3 w-3" />
              Updated {getDataFreshness()}
            </Badge>
            <Badge variant="outline">
              <Server className="mr-1 h-3 w-3" />
              {data.processing_time_ms}ms response
            </Badge>
            <Badge variant="outline">
              <Globe className="mr-1 h-3 w-3" />
              {data.metadata.available_countries.length} markets
            </Badge>
          </div>
        )}

        {/* Refresh and Export Controls */}
        <div className="flex justify-center gap-2">
          <Button
            onClick={handleRefresh}
            disabled={loading}
            variant="outline"
            size="sm"
            className="gap-2"
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            {loading ? 'Refreshing...' : 'Refresh Data'}
          </Button>
          <Button
            onClick={handleExport}
            disabled={loading || !data?.data.length}
            variant="outline"
            size="sm"
            className="gap-2"
          >
            <Download className="h-4 w-4" />
            Export Results
          </Button>
        </div>
      </div>

      {/* Advanced Filter Panel */}
      <AdvancedFilterPanel
        filters={filters}
        options={options}
        loading={loading}
        onFilterChange={updateFilter}
        onFiltersChange={updateFilters}
        onResetFilters={resetFilters}
        onApplyPreset={handlePresetApply}
      />

      {/* Active Filter Indicators */}
      <ActiveFilterIndicators
        filters={filters}
        onRemoveFilter={key =>
          updateFilter(key, key === 'sort_by' ? 'market_cap' : undefined)
        }
        onClearAllFilters={resetFilters}
      />

      {/* Error Display */}
      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            {error}
            <Button
              variant="outline"
              size="sm"
              onClick={refresh}
              className="ml-2"
            >
              Retry
            </Button>
          </AlertDescription>
        </Alert>
      )}

      {/* Enhanced Pagination (Top) */}
      {data && (
        <Pagination
          pagination={data.pagination}
          filters={{ page: filters.page, limit: filters.limit }}
          loading={loading}
          userTier="BRONZE"
          maxAllowedLimit={48}
          onPageChange={goToPage}
          onPageSizeChange={changePageSize}
        />
      )}

      {/* Performance Cards Grid */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-bold">Performance Cards</h3>
          <div className="flex items-center gap-2">
            <Badge className="bg-amber-500 text-white">
              <Crown className="mr-1 h-4 w-4" />
              Bronze Member
            </Badge>
            <Badge variant="outline">Page {currentPage}</Badge>
          </div>
        </div>

        {data?.data && data.data.length > 0 ? (
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {data.data.map(card => (
              <Card
                key={card.symbol}
                className="group transition-all duration-300 hover:scale-[1.02] hover:shadow-lg"
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle className="flex items-center gap-2">
                        <span className="text-xl font-bold">{card.symbol}</span>
                        <Badge
                          className={getRankBadgeStyle(card.rank)}
                          variant="secondary"
                        >
                          #{card.rank}
                        </Badge>
                      </CardTitle>
                      <p className="text-muted-foreground mt-1 text-sm">
                        {formatDate(card.latest_date)}
                      </p>
                    </div>
                  </div>
                </CardHeader>

                <CardContent className="space-y-4">
                  {/* Key Metrics */}
                  <div className="grid grid-cols-2 gap-3">
                    <div className="rounded-lg bg-slate-50 p-3 text-center dark:bg-slate-800/50">
                      <div className="text-muted-foreground mb-1 text-xs">
                        EPS Value
                      </div>
                      <div className="text-sm font-semibold">
                        {card.value?.toFixed(4) ?? 'N/A'}
                      </div>
                    </div>
                    <div className="rounded-lg bg-slate-50 p-3 text-center dark:bg-slate-800/50">
                      <div className="text-muted-foreground mb-1 text-xs">
                        Index
                      </div>
                      <div
                        className={`text-sm font-semibold ${getIndexColor(card.index)}`}
                      >
                        {card.index?.toFixed(1) ?? 'N/A'}
                      </div>
                    </div>
                  </div>

                  {/* Average Growth */}
                  <div className="rounded-lg bg-gradient-to-r from-blue-50 to-green-50 p-3 text-center dark:from-blue-900/20 dark:to-green-900/20">
                    <div className="text-muted-foreground mb-1 text-xs">
                      Avg Growth
                    </div>
                    <div
                      className={`text-lg font-bold ${getGrowthColor(card.avg_growth)}`}
                    >
                      {card.avg_growth != null
                        ? `${card.avg_growth > 0 ? '+' : ''}${card.avg_growth.toFixed(1)}%`
                        : 'N/A'}
                    </div>
                  </div>

                  {/* Quarterly Performance Table */}
                  <div className="space-y-2">
                    <h4 className="text-muted-foreground text-sm font-semibold">
                      Quarterly Performance
                    </h4>
                    <div className="space-y-1">
                      {card.quarterly_performance
                        ?.slice(0, 3)
                        .map((quarter, idx) => (
                          <div
                            key={idx}
                            className="flex items-center justify-between rounded bg-slate-50 p-2 text-xs dark:bg-slate-800/30"
                          >
                            <span className="font-medium">
                              {quarter?.quarter ?? `Q${idx}`}
                            </span>
                            <div className="space-y-0.5 text-right">
                              <div>EPS: {quarter?.eps ?? 'N/A'}</div>
                              <div
                                className={getGrowthColor(quarter?.eps_growth)}
                              >
                                {quarter?.eps_growth != null
                                  ? `${quarter.eps_growth > 0 ? '+' : ''}${quarter.eps_growth.toFixed(1)}%`
                                  : 'N/A'}
                              </div>
                            </div>
                            <div className="text-right">
                              <div>{formatPrice(quarter?.price ?? null)}</div>
                            </div>
                          </div>
                        )) ?? []}
                    </div>
                  </div>

                  {/* TradingView Link */}
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    asChild
                  >
                    <a
                      href={`https://www.tradingview.com/chart?symbol=${card.symbol}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-2"
                    >
                      <BarChart3 className="h-4 w-4" />
                      View Data
                    </a>
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <Card>
            <CardContent className="p-12 text-center">
              <AlertCircle className="mx-auto mb-4 h-12 w-12 text-gray-400" />
              <h3 className="mb-2 text-lg font-semibold">No Results Found</h3>
              <p className="text-muted-foreground mb-4">
                No data matches your current filter criteria.
              </p>
              <Button variant="outline" onClick={resetFilters}>
                Reset Filters
              </Button>
            </CardContent>
          </Card>
        )}
      </div>

      {/* Enhanced Pagination (Bottom) */}
      {data && data.pagination.totalPages > 1 && (
        <Pagination
          pagination={data.pagination}
          filters={{ page: filters.page, limit: filters.limit }}
          loading={loading}
          userTier="BRONZE"
          maxAllowedLimit={48}
          onPageChange={goToPage}
          onPageSizeChange={changePageSize}
        />
      )}

      {/* Loading overlay */}
      {loading && data && (
        <div className="bg-background/80 fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm">
          <div className="bg-card flex items-center gap-2 rounded-lg p-4 shadow-lg">
            <RefreshCw className="h-5 w-5 animate-spin" />
            <span>Loading filtered results...</span>
          </div>
        </div>
      )}
    </div>
  );
}
