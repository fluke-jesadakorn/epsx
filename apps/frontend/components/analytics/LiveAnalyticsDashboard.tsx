'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useRankingAccess } from '@/hooks/useRankingAccess';
import { usePagination } from '@/hooks/usePagination';
import usePaginatedFeatureAccess from '@/hooks/usePaginatedFeatureAccess';
import { AnalyticsClient } from '@epsx/api-client';
import type { 
  UnifiedAnalyticsRankingsResponse, 
  UnifiedRankingItem, 
  CacheStatsResponse,
  CacheHealthResponse 
} from '@epsx/api-client';
import { Pagination } from '@/components/ui/pagination';
import RoleBasedFinancialTable from '@/components/shared/RoleBasedFinancialTable';
import { 
  BarChart3, 
  Crown, 
  Lock, 
  AlertCircle, 
  RefreshCw, 
  Database, 
  Activity, 
  Clock,
  Zap,
  Server,
  TrendingUp,
  Globe
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useToast } from '@/components/ui/use-toast';

export function LiveAnalyticsDashboard() {
  const router = useRouter();
  const { toast } = useToast();
  const { loading } = useRankingAccess();
  const { getMaxAllowedLimit, canAccessPage, getAvailablePageSizes, userTier } =
    usePaginatedFeatureAccess();

  const [analyticsData, setAnalyticsData] = useState<UnifiedAnalyticsRankingsResponse | null>(null);
  const [dataLoading, setDataLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCountry, setSelectedCountry] = useState<string>('');
  const [availableCountries, setAvailableCountries] = useState<string[]>([]);
  
  // Cache management state
  const [cacheStats, setCacheStats] = useState<CacheStatsResponse | null>(null);
  const [cacheHealth, setCacheHealth] = useState<CacheHealthResponse | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [showCacheDetails, setShowCacheDetails] = useState(false);

  // Create AnalyticsClient instance
  const analyticsClient = new AnalyticsClient();

  const {
    currentPage,
    limit,
    isLoading: paginationLoading,
    setIsLoading: setPaginationLoading,
    handlePageChange,
    handleLimitChange,
  } = usePagination({
    initialPage: 1,
    initialLimit: 10,
    onPageChange: async (page, limit) => {
      setPaginationLoading(true);
      setError(null);

      try {
        const response = await analyticsClient.getUnifiedAnalyticsRankings({
          page,
          limit,
          country: selectedCountry || undefined,
          sort_by: 'qoq_growth'
        });
        
        if (response.data) {
          setAnalyticsData(response.data);
        } else {
          throw new Error(response.error || 'Failed to fetch analytics data');
        }
      } catch (error) {
        console.error('Error fetching analytics data:', error);
        setError(`Failed to load analytics data: ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setPaginationLoading(false);
      }
    },
  });

  // Load cache statistics
  const loadCacheStats = useCallback(async () => {
    try {
      const [statsResponse, healthResponse] = await Promise.all([
        analyticsClient.getCacheStats(),
        analyticsClient.getCacheHealth()
      ]);
      
      if (statsResponse.data) {
        setCacheStats(statsResponse.data);
      }
      
      if (healthResponse.data) {
        setCacheHealth(healthResponse.data);
      }
    } catch (error) {
      console.error('Failed to load cache stats:', error);
    }
  }, []);

  // Load initial data
  useEffect(() => {
    const loadInitialData = async () => {
      try {
        setDataLoading(true);
        
        // Load initial unified analytics data and cache stats simultaneously
        const [analyticsResponse] = await Promise.all([
          analyticsClient.getUnifiedAnalyticsRankings({
            page: 1,
            limit: 10,
            sort_by: 'qoq_growth'
          }),
          loadCacheStats()
        ]);
        
        if (analyticsResponse.data) {
          setAnalyticsData(analyticsResponse.data);
          // Set available countries from metadata
          setAvailableCountries(['All Countries', ...analyticsResponse.data.metadata.available_countries]);
        } else {
          throw new Error(analyticsResponse.error || 'Failed to fetch analytics data');
        }
      } catch (error) {
        console.error('Failed to load initial analytics data:', error);
        setError(`Failed to load initial data: ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setDataLoading(false);
      }
    };

    if (!loading) {
      loadInitialData();
    }
  }, [loading, loadCacheStats]);

  // Refresh cache manually
  const handleCacheRefresh = async () => {
    setRefreshing(true);
    
    try {
      const response = await analyticsClient.refreshCache();
      
      if (response.data?.success) {
        toast({
          title: "Cache Refreshed Successfully",
          description: `Refreshed ${response.data.refreshed_entries} entries in ${response.data.duration_ms}ms`,
          variant: "default"
        });
        
        // Reload data after cache refresh
        handlePageChange(currentPage);
        loadCacheStats();
      } else {
        throw new Error('Failed to refresh cache');
      }
    } catch (error) {
      toast({
        title: "Cache Refresh Failed",
        description: `Error: ${error instanceof Error ? error.message : String(error)}`,
        variant: "destructive"
      });
    } finally {
      setRefreshing(false);
    }
  };

  const maxAllowedLimit = getMaxAllowedLimit();
  const availablePageSizes = getAvailablePageSizes();
  const currentPageAccessible = canAccessPage(currentPage, limit);

  const handleUpgrade = () => {
    router.push('/payment');
  };

  const handleRetry = () => {
    handlePageChange(currentPage);
  };

  // Transform unified analytics data to StockFinancialData format for the table
  const transformUnifiedDataForTable = (unifiedItems: UnifiedRankingItem[]) => {
    return unifiedItems.map(item => ({
      symbol: item.symbol,
      quarters: item.quarterly_data.map(qData => ({
        price: qData.price,
        date: qData.date,
        eps: parseFloat(qData.eps.toFixed(4)),
        quarter: qData.quarter,
        eps_growth: parseFloat(qData.eps_growth.toFixed(1)),
        price_growth: qData.price_growth,
        last_eps_vs_current_price: {
          lastEpsGrowth: parseFloat(qData.eps_growth.toFixed(1)),
          currentPriceGrowth: qData.price_growth,
        },
      })),
      currentPrice: item.current_price,
      currentPriceDate: item.current_price_date,
    }));
  };

  const handleCountryChange = async (country: string) => {
    const countryFilter = country === 'All Countries' ? '' : country;
    setSelectedCountry(countryFilter);
    
    // Reload data with new country filter
    try {
      setPaginationLoading(true);
      setError(null);
      
      const response = await analyticsClient.getUnifiedAnalyticsRankings({
        page: 1,
        limit,
        country: countryFilter || undefined,
        sort_by: 'qoq_growth'
      });
      
      if (response.data) {
        setAnalyticsData(response.data);
        // Reset to page 1 when country changes
        handlePageChange(1);
      } else {
        throw new Error(response.error || 'Failed to fetch analytics data');
      }
    } catch (error) {
      console.error('Error changing country filter:', error);
      setError(`Failed to load data for selected country: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setPaginationLoading(false);
    }
  };

  // Get data freshness indicator
  const getDataFreshness = () => {
    if (!analyticsData?.metadata.request_timestamp) return null;
    
    const dataTime = new Date(analyticsData.metadata.request_timestamp);
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

  if (loading || dataLoading) {
    return (
      <div className="space-y-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="h-4 bg-gray-200 rounded w-2/3 mb-8"></div>
          <div className="grid gap-6 md:grid-cols-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-32 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  const getLevelInfo = () => {
    const levels = {
      BRONZE: { color: 'bg-amber-500', name: 'Bronze', maxRank: 5 },
      SILVER: { color: 'bg-gray-400', name: 'Silver', maxRank: 25 },
      GOLD: { color: 'bg-yellow-500', name: 'Gold', maxRank: 50 },
      PLATINUM: { color: 'bg-purple-500', name: 'Platinum', maxRank: 100 },
    };
    return levels.BRONZE;
  };

  const levelInfo = getLevelInfo();

  return (
    <div className="space-y-8">
      {/* Header with Live Data Indicators */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <Zap className="h-10 w-10 text-green-600" />
          <h1 className="text-4xl font-bold">Live Analytics Dashboard</h1>
        </div>
        <p className="text-xl text-muted-foreground">
          Real-time stock rankings with live cache-based data
        </p>

        {/* Data Source & Freshness Info */}
        {analyticsData && (
          <div className="flex justify-center gap-4 flex-wrap">
            <Badge className={`${getDataSourceColor(analyticsData.metadata.data_source)} text-white`}>
              <Activity className="h-3 w-3 mr-1" />
              {analyticsData.metadata.data_source.replace('_', ' ').toUpperCase()}
            </Badge>
            <Badge variant="outline">
              <Clock className="h-3 w-3 mr-1" />
              Updated {getDataFreshness()}
            </Badge>
            <Badge variant="outline">
              <Server className="h-3 w-3 mr-1" />
              {analyticsData.processing_time_ms}ms response
            </Badge>
            <Badge variant="outline">
              <Globe className="h-3 w-3 mr-1" />
              {analyticsData.metadata.available_countries.length} markets
            </Badge>
          </div>
        )}

        {/* User Level Badge */}
        <div className="flex justify-center">
          <Badge className={`${levelInfo.color} text-white px-6 py-2 text-lg gap-2`}>
            <Crown className="h-5 w-5" />
            {levelInfo.name} Member
          </Badge>
        </div>
      </div>

      {/* Cache Stats Cards */}
      {(cacheStats || cacheHealth) && (
        <div className="grid gap-4 md:grid-cols-4">
          {cacheStats && (
            <>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Active Entries</CardTitle>
                  <Database className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{cacheStats.stats.active_entries}</div>
                  <p className="text-xs text-muted-foreground">
                    {cacheStats.stats.total_entries} total
                  </p>
                </CardContent>
              </Card>
              
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Hit Ratio</CardTitle>
                  <TrendingUp className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{(cacheStats.stats.hit_ratio * 100).toFixed(1)}%</div>
                  <p className="text-xs text-muted-foreground">
                    Cache efficiency
                  </p>
                </CardContent>
              </Card>
            </>
          )}
          
          {cacheHealth && (
            <>
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Cache Health</CardTitle>
                  <Activity className={`h-4 w-4 ${cacheHealth.healthy ? 'text-green-500' : 'text-red-500'}`} />
                </CardHeader>
                <CardContent>
                  <div className={`text-2xl font-bold ${cacheHealth.healthy ? 'text-green-600' : 'text-red-600'}`}>
                    {cacheHealth.status.toUpperCase()}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {cacheHealth.cache_stats.cache_size_mb.toFixed(1)}MB used
                  </p>
                </CardContent>
              </Card>
              
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Cache Control</CardTitle>
                  <RefreshCw className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <Button 
                    onClick={handleCacheRefresh}
                    disabled={refreshing}
                    className="w-full"
                    variant="outline"
                    size="sm"
                  >
                    {refreshing ? (
                      <>
                        <RefreshCw className="h-3 w-3 mr-1 animate-spin" />
                        Refreshing...
                      </>
                    ) : (
                      <>
                        <RefreshCw className="h-3 w-3 mr-1" />
                        Refresh Now
                      </>
                    )}
                  </Button>
                  <Button 
                    onClick={() => setShowCacheDetails(!showCacheDetails)}
                    variant="link"
                    size="sm"
                    className="w-full p-0 h-auto mt-1 text-xs"
                  >
                    {showCacheDetails ? 'Hide' : 'Show'} Details
                  </Button>
                </CardContent>
              </Card>
            </>
          )}
        </div>
      )}

      {/* Cache Details (Expandable) */}
      {showCacheDetails && cacheHealth && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              Cache Details & Recommendations
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-2">
              <div>
                <h4 className="font-semibold mb-2">Statistics</h4>
                <div className="space-y-1 text-sm">
                  <div>Miss Ratio: {(cacheHealth.cache_stats.miss_ratio * 100).toFixed(1)}%</div>
                  <div>Expired Entries: {cacheHealth.cache_stats.expired_entries}</div>
                  <div>Memory Usage: {cacheHealth.cache_stats.cache_size_mb.toFixed(2)}MB</div>
                </div>
              </div>
              <div>
                <h4 className="font-semibold mb-2">Recommendations</h4>
                <div className="space-y-1">
                  {cacheHealth.recommendations.length > 0 ? 
                    cacheHealth.recommendations.map((rec, index) => (
                      <Badge key={index} variant="outline" className="text-xs">
                        {rec}
                      </Badge>
                    )) : 
                    <Badge variant="outline" className="text-xs text-green-600">
                      Cache is performing optimally
                    </Badge>
                  }
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Controls */}
      <div className="flex flex-col gap-4">
        {/* Country Filter */}
        <div className="bg-card p-4 rounded-lg border">
          <div className="flex flex-col sm:flex-row sm:items-center gap-4">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Country:</span>
              <Select
                value={selectedCountry === '' ? 'All Countries' : selectedCountry}
                onValueChange={handleCountryChange}
              >
                <SelectTrigger className="w-48">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {availableCountries.map((country) => (
                    <SelectItem key={country} value={country}>
                      {country}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>

        {/* Pagination Controls */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 bg-card p-4 rounded-lg border">
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">Items per page:</span>
            <Select
              value={limit.toString()}
              onValueChange={(value) => handleLimitChange(parseInt(value))}
            >
              <SelectTrigger className="w-20">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {availablePageSizes.map((size) => (
                  <SelectItem key={size} value={size.toString()}>
                    {size}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Badge variant="outline" className="ml-2">
              {userTier} Plan
            </Badge>
          </div>

          <div className="text-sm text-muted-foreground">
            Showing{' '}
            {analyticsData ? Math.min((currentPage - 1) * limit + 1, analyticsData.pagination.total) : 0}{' '}
            to {analyticsData ? Math.min(currentPage * limit, analyticsData.pagination.total) : 0} of{' '}
            {analyticsData?.pagination.total || 0} results
          </div>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <Card className="border-red-200 bg-red-50 dark:bg-red-950/20">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-red-800 dark:text-red-200">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
              <Button
                variant="outline"
                size="sm"
                onClick={handleRetry}
                className="ml-auto"
              >
                Retry
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Access Restriction Message */}
      {!currentPageAccessible && (
        <Card className="border-orange-200 bg-orange-50 dark:bg-orange-950/20">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-orange-800 dark:text-orange-200">
              <Lock className="h-4 w-4" />
              <span className="text-sm">
                Your {userTier} plan allows access to only {maxAllowedLimit}{' '}
                items.
                <Button
                  variant="link"
                  onClick={handleUpgrade}
                  className="p-0 h-auto font-semibold"
                >
                  Upgrade now
                </Button>{' '}
                to see more results.
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Main Table */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-bold">Live Stock Rankings</h3>
          <Badge className={levelInfo.color}>
            {currentPageAccessible ? `Page ${currentPage}` : 'Limited Access'}
          </Badge>
        </div>

        {analyticsData?.data && analyticsData.data.length > 0 ? (
          <RoleBasedFinancialTable data={transformUnifiedDataForTable(analyticsData.data)} />
        ) : (
          <Card>
            <CardContent className="p-12 text-center">
              <Lock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Data Available</h3>
              <p className="text-muted-foreground">
                No EPS data available for the current selection.
              </p>
            </CardContent>
          </Card>
        )}
      </div>

      {/* Loading state */}
      {paginationLoading && (
        <div className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      )}

      {/* Pagination */}
      {analyticsData && analyticsData.pagination.totalPages > 1 && (
        <div className="space-y-4">
          <Pagination
            currentPage={currentPage}
            totalPages={analyticsData.pagination.totalPages}
            onPageChange={handlePageChange}
            hasNext={analyticsData.pagination.hasNext}
            hasPrev={analyticsData.pagination.hasPrev}
            isLoading={paginationLoading}
            className="mt-8"
          />

          {/* Upgrade prompt for pagination */}
          {analyticsData.pagination.totalPages > 1 && userTier === 'BASIC' && (
            <Card className="border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20">
              <CardContent className="p-6 text-center">
                <div className="space-y-4">
                  <div className="flex justify-center">
                    <Crown className="h-12 w-12 text-yellow-500" />
                  </div>
                  <div>
                    <h3 className="text-lg font-bold mb-2">
                      🚀 Unlock Full Live Analytics Access
                    </h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      You&apos;re seeing limited results. Upgrade to access all{' '}
                      {analyticsData.pagination.total} live analytics rankings!
                    </p>
                    <div className="flex flex-wrap justify-center gap-2 text-xs">
                      <Badge variant="secondary">📊 Live Analytics Rankings</Badge>
                      <Badge variant="secondary">🎯 Global Market Data</Badge>
                      <Badge variant="secondary">⚡ Real-time Cache</Badge>
                      <Badge variant="secondary">🌍 Multi-country Access</Badge>
                    </div>
                  </div>
                  <Button onClick={handleUpgrade} className="gap-2">
                    <Crown className="h-4 w-4" />
                    Upgrade Now
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  );
}