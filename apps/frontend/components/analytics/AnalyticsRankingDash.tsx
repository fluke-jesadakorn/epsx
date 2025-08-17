'use client';

import React from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Zap,
  Crown,
  AlertCircle,
  RefreshCw,
  Activity,
  Clock,
  Server,
  Globe,
  BarChart3,
  Download,
} from 'lucide-react';
import { useAnalyticsFilters } from '@/hooks/useAnalyticsFilters';
import { AdvancedFilterPanel } from './AdvancedFilterPanel';
import { Pagination } from './Pagination';
import { ActiveFilterIndicators } from './ActiveFilterIndicators';
import { formatPrice, formatDate } from '@/utils/fmt';
import { useToast } from '@/components/ui/use-toast';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
}

interface AnalyticsRankingDashboardProps {
  className?: string;
  user?: AuthUser | null;
}

export function AnalyticsRankingDashboard({ 
  className,
  user 
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
        title: "Data Refreshed",
        description: "Analytics data has been updated successfully",
        variant: "default"
      });
    } catch (error) {
      toast({
        title: "Refresh Failed", 
        description: `Error: ${error instanceof Error ? error.message : String(error)}`,
        variant: "destructive"
      });
    }
  };

  const handleExport = async () => {
    // Placeholder for export functionality
    toast({
      title: "Export Started",
      description: "Your filtered data export is being prepared",
      variant: "default"
    });
  };

  // Handle preset application
  const handlePresetApply = (presetName: string) => {
    applyPreset(presetName as any);
    toast({
      title: "Filter Preset Applied",
      description: `Applied ${presetName} filter preset`,
      variant: "default"
    });
  };

  if (accessLoading || (loading && !data)) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="h-4 bg-gray-200 rounded w-2/3 mb-8"></div>
          <div className="grid gap-6 md:grid-cols-3 lg:grid-cols-4">
            {Array.from({ length: 12 }).map((_, i) => (
              <div key={i} className="h-64 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <Zap className="h-10 w-10 text-green-600" />
          <h1 className="text-4xl font-bold">Enhanced EPS Rankings</h1>
        </div>
        <p className="text-xl text-muted-foreground">
          Advanced analytics dashboard with powerful filtering and page-based pagination
        </p>

        {/* Data Source & Freshness Info */}
        {data && (
          <div className="flex justify-center gap-4 flex-wrap">
            <Badge className={`${getDataSourceColor(data.metadata.data_source)} text-white`}>
              <Activity className="h-3 w-3 mr-1" />
              {data.metadata.data_source.replace('_', ' ').toUpperCase()}
            </Badge>
            <Badge variant="outline">
              <Clock className="h-3 w-3 mr-1" />
              Updated {getDataFreshness()}
            </Badge>
            <Badge variant="outline">
              <Server className="h-3 w-3 mr-1" />
              {data.processing_time_ms}ms response
            </Badge>
            <Badge variant="outline">
              <Globe className="h-3 w-3 mr-1" />
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
        onRemoveFilter={(key) => updateFilter(key, key === 'sort_by' ? 'market_cap' : undefined)}
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
              <Crown className="h-4 w-4 mr-1" />
              Bronze Member
            </Badge>
            <Badge variant="outline">
              Page {currentPage}
            </Badge>
          </div>
        </div>

        {data?.data && data.data.length > 0 ? (
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {data.data.map((card) => (
              <Card key={card.symbol} className="group hover:shadow-lg transition-all duration-300 hover:scale-[1.02]">
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle className="flex items-center gap-2">
                        <span className="text-xl font-bold">{card.symbol}</span>
                        <Badge className={getRankBadgeStyle(card.rank)} variant="secondary">
                          #{card.rank}
                        </Badge>
                      </CardTitle>
                      <p className="text-sm text-muted-foreground mt-1">
                        {formatDate(card.latest_date)}
                      </p>
                    </div>
                  </div>
                </CardHeader>

                <CardContent className="space-y-4">
                  {/* Key Metrics */}
                  <div className="grid grid-cols-2 gap-3">
                    <div className="text-center p-3 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
                      <div className="text-xs text-muted-foreground mb-1">EPS Value</div>
                      <div className="font-semibold text-sm">{card.value?.toFixed(4) ?? 'N/A'}</div>
                    </div>
                    <div className="text-center p-3 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
                      <div className="text-xs text-muted-foreground mb-1">Index</div>
                      <div className={`font-semibold text-sm ${getIndexColor(card.index)}`}>
                        {card.index?.toFixed(1) ?? 'N/A'}
                      </div>
                    </div>
                  </div>

                  {/* Average Growth */}
                  <div className="text-center p-3 bg-gradient-to-r from-blue-50 to-green-50 dark:from-blue-900/20 dark:to-green-900/20 rounded-lg">
                    <div className="text-xs text-muted-foreground mb-1">Avg Growth</div>
                    <div className={`font-bold text-lg ${getGrowthColor(card.avg_growth)}`}>
                      {card.avg_growth != null ? `${card.avg_growth > 0 ? '+' : ''}${card.avg_growth.toFixed(1)}%` : 'N/A'}
                    </div>
                  </div>

                  {/* Quarterly Performance Table */}
                  <div className="space-y-2">
                    <h4 className="text-sm font-semibold text-muted-foreground">Quarterly Performance</h4>
                    <div className="space-y-1">
                      {card.quarterly_performance?.slice(0, 3).map((quarter, idx) => (
                        <div key={idx} className="flex justify-between items-center text-xs p-2 bg-slate-50 dark:bg-slate-800/30 rounded">
                          <span className="font-medium">{quarter?.quarter ?? `Q${idx}`}</span>
                          <div className="text-right space-y-0.5">
                            <div>EPS: {quarter?.eps ?? 'N/A'}</div>
                            <div className={getGrowthColor(quarter?.eps_growth)}>
                              {quarter?.eps_growth != null ? `${quarter.eps_growth > 0 ? '+' : ''}${quarter.eps_growth.toFixed(1)}%` : 'N/A'}
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
                      View Chart
                    </a>
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <Card>
            <CardContent className="p-12 text-center">
              <AlertCircle className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Results Found</h3>
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
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="flex items-center gap-2 bg-card p-4 rounded-lg shadow-lg">
            <RefreshCw className="h-5 w-5 animate-spin" />
            <span>Loading filtered results...</span>
          </div>
        </div>
      )}
    </div>
  );
}