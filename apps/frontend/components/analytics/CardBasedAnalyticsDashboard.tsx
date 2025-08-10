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
  CardDashboardResponse, 
  SymbolCardData,
  CardDashboardMetadata 
} from '@/types/financialChartData';
import { Pagination } from '@/components/ui/pagination';
import { 
  BarChart3, 
  Crown, 
  Lock, 
  AlertCircle, 
  RefreshCw, 
  Activity, 
  Clock,
  Zap,
  Server,
  TrendingUp,
  Globe,
  DollarSign
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useToast } from '@/components/ui/use-toast';
import { formatPrice, formatDate } from '@/utils/fmt';

interface CardBasedAnalyticsDashboardProps {
  className?: string;
}

export function CardBasedAnalyticsDashboard({ className }: CardBasedAnalyticsDashboardProps) {
  const router = useRouter();
  const { toast } = useToast();
  const { loading } = useRankingAccess();
  const { getMaxAllowedLimit, canAccessPage, getAvailablePageSizes, userTier } =
    usePaginatedFeatureAccess();

  const [cardData, setCardData] = useState<CardDashboardResponse | null>(null);
  const [dataLoading, setDataLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCountry, setSelectedCountry] = useState<string>('');
  const [availableCountries, setAvailableCountries] = useState<string[]>([]);
  const [refreshing, setRefreshing] = useState(false);

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
    initialLimit: 12,
    onPageChange: async (page, limit) => {
      setPaginationLoading(true);
      setError(null);

      try {
        const response = await analyticsClient.getUnifiedAnalyticsRankings({
          page,
          limit,
          country: selectedCountry || undefined,
          sort_by: 'ranking_score'
        });
        
        if (response.data) {
          setCardData(response.data);
        } else {
          throw new Error(response.error || 'Failed to fetch card dashboard data');
        }
      } catch (error) {
        console.error('Error fetching card dashboard:', error);
        setError(`Failed to load card dashboard: ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setPaginationLoading(false);
      }
    },
  });

  // Load initial data
  useEffect(() => {
    const loadInitialData = async () => {
      try {
        setDataLoading(true);
        
        const response = await analyticsClient.getUnifiedAnalyticsRankings({
          page: 1,
          limit: 12,
          sort_by: 'ranking_score'
        });
        
        if (response.data) {
          setCardData(response.data);
          // Set available countries from metadata
          setAvailableCountries(['All Countries', ...response.data.metadata.available_countries]);
        } else {
          throw new Error(response.error || 'Failed to fetch card dashboard data');
        }
      } catch (error) {
        console.error('Failed to load initial card dashboard:', error);
        setError(`Failed to load initial data: ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setDataLoading(false);
      }
    };

    if (!loading) {
      loadInitialData();
    }
  }, [loading]);

  const handleCountryChange = async (country: string) => {
    const countryFilter = country === 'All Countries' ? '' : country;
    setSelectedCountry(countryFilter);
    
    try {
      setPaginationLoading(true);
      setError(null);
      
      const response = await analyticsClient.getUnifiedAnalyticsRankings({
        page: 1,
        limit,
        country: countryFilter || undefined,
        sort_by: 'ranking_score'
      });
      
      if (response.data) {
        setCardData(response.data);
        handlePageChange(1);
      } else {
        throw new Error(response.error || 'Failed to fetch card dashboard data');
      }
    } catch (error) {
      console.error('Error changing country filter:', error);
      setError(`Failed to load data for selected country: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setPaginationLoading(false);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    
    try {
      const response = await analyticsClient.getUnifiedAnalyticsRankings({
        page: currentPage,
        limit,
        country: selectedCountry || undefined,
        sort_by: 'ranking_score'
      });
      
      if (response.data) {
        setCardData(response.data);
        toast({
          title: "Data Refreshed",
          description: "Card dashboard data has been updated",
          variant: "default"
        });
      } else {
        throw new Error('Failed to refresh data');
      }
    } catch (error) {
      toast({
        title: "Refresh Failed",
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

  // Get data freshness indicator
  const getDataFreshness = () => {
    if (!cardData?.metadata.request_timestamp) return null;
    
    const dataTime = new Date(cardData.metadata.request_timestamp);
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

  if (loading || dataLoading) {
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
    <div className={`space-y-8 ${className}`}>
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <Zap className="h-10 w-10 text-green-600" />
          <h1 className="text-4xl font-bold">EPS Rankings Cards</h1>
        </div>
        <p className="text-xl text-muted-foreground">
          Multi-symbol financial performance dashboard
        </p>

        {/* Data Source & Freshness Info */}
        {cardData && (
          <div className="flex justify-center gap-4 flex-wrap">
            <Badge className={`${getDataSourceColor(cardData.metadata.data_source)} text-white`}>
              <Activity className="h-3 w-3 mr-1" />
              {cardData.metadata.data_source.replace('_', ' ').toUpperCase()}
            </Badge>
            <Badge variant="outline">
              <Clock className="h-3 w-3 mr-1" />
              Updated {getDataFreshness()}
            </Badge>
            <Badge variant="outline">
              <Server className="h-3 w-3 mr-1" />
              {cardData.processing_time_ms}ms response
            </Badge>
            <Badge variant="outline">
              <Globe className="h-3 w-3 mr-1" />
              {cardData.metadata.available_countries.length} markets
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

      {/* Controls */}
      <div className="flex flex-col gap-4">
        {/* Filter and Refresh Controls */}
        <div className="bg-card p-4 rounded-lg border">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div className="flex items-center gap-4">
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

            <Button 
              onClick={handleRefresh}
              disabled={refreshing}
              variant="outline"
              size="sm"
              className="gap-2"
            >
              <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
              {refreshing ? 'Refreshing...' : 'Refresh Data'}
            </Button>
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
            {cardData ? Math.min((currentPage - 1) * limit + 1, cardData.pagination.total) : 0}{' '}
            to {cardData ? Math.min(currentPage * limit, cardData.pagination.total) : 0} of{' '}
            {cardData?.pagination.total || 0} results
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

      {/* Cards Grid */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-bold">Performance Cards</h3>
          <Badge className={levelInfo.color}>
            {currentPageAccessible ? `Page ${currentPage}` : 'Limited Access'}
          </Badge>
        </div>

        {cardData?.data && cardData.data.length > 0 ? (
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {cardData.data.map((card) => (
              <Card key={card.symbol} className="group hover:shadow-lg transition-all duration-300">
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
                            <div>EPS: {quarter?.eps?.toFixed(3) ?? 'N/A'}</div>
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
              <Lock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Data Available</h3>
              <p className="text-muted-foreground">
                No EPS card data available for the current selection.
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
      {cardData && cardData.pagination.totalPages > 1 && (
        <div className="space-y-4">
          <Pagination
            currentPage={currentPage}
            totalPages={cardData.pagination.totalPages}
            onPageChange={handlePageChange}
            hasNext={cardData.pagination.hasNext}
            hasPrev={cardData.pagination.hasPrev}
            isLoading={paginationLoading}
            className="mt-8"
          />

          {/* Upgrade prompt for pagination */}
          {cardData.pagination.totalPages > 1 && userTier === 'BASIC' && (
            <Card className="border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20">
              <CardContent className="p-6 text-center">
                <div className="space-y-4">
                  <div className="flex justify-center">
                    <Crown className="h-12 w-12 text-yellow-500" />
                  </div>
                  <div>
                    <h3 className="text-lg font-bold mb-2">
                      🚀 Unlock Full Card Dashboard Access
                    </h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      You&apos;re seeing limited results. Upgrade to access all{' '}
                      {cardData.pagination.total} ranking cards!
                    </p>
                    <div className="flex flex-wrap justify-center gap-2 text-xs">
                      <Badge variant="secondary">📊 Full Card Dashboard</Badge>
                      <Badge variant="secondary">🎯 Performance Rankings</Badge>
                      <Badge variant="secondary">⚡ Real-time Data</Badge>
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