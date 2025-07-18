'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useRankingAccess } from '@/hooks/useRankingAccess';
import { usePagination } from '@/hooks/usePagination';
import { usePaginatedFeatureAccess } from '@/hooks/usePaginatedFeatureAccess';
import { fetchPaginatedStockData } from '@/app/actions/stockRankingPaginated';
import { Pagination } from '@/components/ui/pagination';
import RoleBasedFinancialTable from '@/components/shared/RoleBasedFinancialTable';
import { AnalyticsMetrics } from '@/components/analytics/AnalyticsMetrics';
import { 
  BarChart3, 
  Crown, 
  Lock,
  AlertCircle
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import type { PaginatedStockData } from '@/app/actions/stockRankingPaginated';

export function AnalyticsRankingDashboard() {
  const router = useRouter();
  const { maxRankings, userLevel, isExpired, isLoading } = useRankingAccess();
  const { 
    getMaxAllowedLimit, 
    canAccessPage, 
    getAvailablePageSizes,
    userTier 
  } = usePaginatedFeatureAccess();
  
  const [stockData, setStockData] = useState<PaginatedStockData>({
    data: [],
    pagination: { page: 1, limit: 10, total: 0, totalPages: 0, hasNext: false, hasPrev: false }
  });
  const [dataLoading, setDataLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const {
    currentPage,
    limit,
    isLoading: paginationLoading,
    setIsLoading: setPaginationLoading,
    handlePageChange,
    handleLimitChange
  } = usePagination({
    initialPage: 1,
    initialLimit: 10,
    onPageChange: async (page, limit) => {
      setPaginationLoading(true);
      setError(null);
      
      try {
        const newData = await fetchPaginatedStockData(page, limit);
        setStockData(newData);
      } catch (error) {
        console.error('Error fetching paginated data:', error);
        setError('Failed to load stock data. Please try again.');
      } finally {
        setPaginationLoading(false);
      }
    }
  });

  useEffect(() => {
    const loadInitialData = async () => {
      try {
        setDataLoading(true);
        const initialData = await fetchPaginatedStockData(1, 10);
        setStockData(initialData);
      } catch (error) {
        console.error('Failed to load initial analytics data:', error);
        setError('Failed to load initial data. Please try again.');
      } finally {
        setDataLoading(false);
      }
    };

    if (!isLoading) {
      loadInitialData();
    }
  }, [isLoading]);

  const maxAllowedLimit = getMaxAllowedLimit();
  const availablePageSizes = getAvailablePageSizes();
  const currentPageAccessible = canAccessPage(currentPage, limit);

  const handleUpgrade = () => {
    router.push('/payment');
  };

  const handleRetry = () => {
    handlePageChange(currentPage);
  };

  if (isLoading || dataLoading) {
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
    return levels[userLevel as keyof typeof levels] || levels.BRONZE;
  };

  const levelInfo = getLevelInfo();

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <BarChart3 className="h-10 w-10 text-blue-600" />
          <h1 className="text-4xl font-bold">Analytics Dashboard</h1>
        </div>
        <p className="text-xl text-muted-foreground">
          Advanced stock ranking analytics based on your subscription
        </p>
        
        {/* User Level Badge */}
        <div className="flex justify-center">
          <Badge 
            className={`${levelInfo.color} text-white px-6 py-2 text-lg gap-2`}
          >
            <Crown className="h-5 w-5" />
            {levelInfo.name} Member
            {isExpired && <span className="text-xs">(Expired)</span>}
          </Badge>
        </div>
      </div>

      {/* Analytics Metrics */}
      <AnalyticsMetrics 
        userLevel={userLevel}
        maxRankings={maxRankings}
        isExpired={isExpired}
      />

      {/* Pagination Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 bg-card p-4 rounded-lg border">
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Items per page:</span>
          <Select value={limit.toString()} onValueChange={(value) => handleLimitChange(parseInt(value))}>
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
          Showing {Math.min(((currentPage - 1) * limit) + 1, stockData.pagination.total)} to {Math.min(currentPage * limit, stockData.pagination.total)} of {stockData.pagination.total} results
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <Card className="border-red-200 bg-red-50 dark:bg-red-950/20">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-red-800 dark:text-red-200">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
              <Button variant="outline" size="sm" onClick={handleRetry} className="ml-auto">
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
                Your {userTier} plan allows access to only {maxAllowedLimit} items. 
                <Button variant="link" onClick={handleUpgrade} className="p-0 h-auto font-semibold">
                  Upgrade now
                </Button>
                {' '}to see more results.
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Main Table */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-2xl font-bold">Stock Rankings</h3>
          <Badge className={levelInfo.color}>
            {currentPageAccessible ? `Page ${currentPage}` : 'Limited Access'}
          </Badge>
        </div>
        
        {stockData.data.length > 0 ? (
          <RoleBasedFinancialTable data={stockData.data} />
        ) : (
          <Card>
            <CardContent className="p-12 text-center">
              <Lock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Data Available</h3>
              <p className="text-muted-foreground">
                No stock data available for the current selection.
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
      {stockData.pagination.totalPages > 1 && (
        <div className="space-y-4">
          <Pagination
            currentPage={currentPage}
            totalPages={stockData.pagination.totalPages}
            onPageChange={handlePageChange}
            hasNext={stockData.pagination.hasNext}
            hasPrev={stockData.pagination.hasPrev}
            isLoading={paginationLoading}
            className="mt-8"
          />
          
          {/* Upgrade prompt for pagination */}
          {stockData.pagination.totalPages > 1 && userTier === 'BASIC' && (
            <Card className="border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20">
              <CardContent className="p-6 text-center">
                <div className="space-y-4">
                  <div className="flex justify-center">
                    <Crown className="h-12 w-12 text-yellow-500" />
                  </div>
                  <div>
                    <h3 className="text-lg font-bold mb-2">
                      🚀 Unlock Full Pagination Access
                    </h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      You're seeing limited results. Upgrade to access all {stockData.pagination.total} stocks!
                    </p>
                    <div className="flex flex-wrap justify-center gap-2 text-xs">
                      <Badge variant="secondary">📊 Full Stock List</Badge>
                      <Badge variant="secondary">🎯 Advanced Filtering</Badge>
                      <Badge variant="secondary">💎 Premium Features</Badge>
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