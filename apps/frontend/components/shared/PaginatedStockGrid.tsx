'use client';

import { useState } from 'react';
import { usePagination } from '@/hooks/usePagination';
import { usePaginatedFeatureAccess } from '@/hooks/usePaginatedFeatureAccess';
import { fetchPaginatedStockDataFromAPI } from '@/app/actions/stockRankingPaginated';
import type { PaginatedStockData } from '@/app/actions/stockRankingPaginated';
import { Pagination } from '@/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { TrendingUp, TrendingDown, Crown, Lock, AlertCircle } from 'lucide-react';
import { useRouter } from 'next/navigation';

interface PaginatedStockGridProps {
  initialData?: PaginatedStockData;
  className?: string;
  useApi?: boolean; // Toggle between server-side and API calls
}

export function PaginatedStockGrid({ 
  initialData, 
  className = '', 
  useApi = true 
}: PaginatedStockGridProps) {
  const router = useRouter();
  const [stockData, setStockData] = useState<PaginatedStockData>(
    initialData || {
      data: [],
      pagination: { page: 1, limit: 10, total: 0, totalPages: 0, hasNext: false, hasPrev: false }
    }
  );
  const [error, setError] = useState<string | null>(null);

  const { 
    getMaxAllowedLimit, 
    canAccessPage, 
    getAvailablePageSizes, 
    userTier 
  } = usePaginatedFeatureAccess();

  const {
    currentPage,
    limit,
    isLoading,
    setIsLoading,
    handlePageChange,
    handleLimitChange
  } = usePagination({
    initialPage: initialData?.pagination.page || 1,
    initialLimit: initialData?.pagination.limit || 10,
    onPageChange: async (page, limit) => {
      setIsLoading(true);
      setError(null);
      
      try {
        let newData: PaginatedStockData;
        
        if (useApi) {
          // Use API client for client-side pagination
          newData = await fetchPaginatedStockDataFromAPI(page, limit);
        } else {
          // Use server action for server-side pagination
          const { fetchPaginatedStockData } = await import('@/app/actions/stockRankingPaginated');
          newData = await fetchPaginatedStockData(page, limit);
        }
        
        setStockData(newData);
      } catch (error) {
        console.error('Error fetching paginated data:', error);
        setError('Failed to load stock data. Please try again.');
      } finally {
        setIsLoading(false);
      }
    }
  });

  const maxAllowedLimit = getMaxAllowedLimit();
  const availablePageSizes = getAvailablePageSizes();
  const currentPageAccessible = canAccessPage(currentPage, limit);

  const handleUpgrade = () => {
    router.push('/payment');
  };

  const handleRetry = () => {
    handlePageChange(currentPage);
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
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
          {useApi && (
            <Badge variant="secondary" className="text-xs">
              API
            </Badge>
          )}
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

      {/* Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 sm:gap-6 auto-rows-max w-full max-w-full">
        {stockData.data.map((stock, index) => {
          const latestQuarter = stock.quarters[stock.quarters.length - 1];
          const previousQuarter = stock.quarters[stock.quarters.length - 2];
          const epsGrowth = previousQuarter 
            ? ((latestQuarter?.eps - previousQuarter.eps) / previousQuarter.eps * 100)
            : 0;

          const globalRank = (currentPage - 1) * limit + index + 1;
          const isLocked = !currentPageAccessible && globalRank > maxAllowedLimit;

          return (
            <Card key={`${stock.symbol}-${index}`} className={`relative w-full max-w-full overflow-hidden transition-all duration-300 ${
              isLocked 
                ? 'opacity-50 border-dashed cursor-not-allowed' 
                : 'hover:shadow-lg group'
            }`}>
              {isLocked && (
                <div className="absolute inset-0 bg-black/10 flex items-center justify-center z-10">
                  <div className="bg-white/90 rounded-lg p-2 shadow-lg">
                    <Lock className="h-6 w-6 text-gray-600" />
                  </div>
                </div>
              )}
              
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg flex items-center gap-2">
                    <Badge variant="outline" className="text-xs">
                      #{globalRank}
                    </Badge>
                    {stock.symbol}
                  </CardTitle>
                  <div className="flex items-center gap-1">
                    {epsGrowth > 0 ? (
                      <TrendingUp className="h-4 w-4 text-green-500" />
                    ) : (
                      <TrendingDown className="h-4 w-4 text-red-500" />
                    )}
                  </div>
                </div>
              </CardHeader>
              
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">EPS Growth</span>
                    <span className={`font-semibold ${
                      epsGrowth > 0 ? 'text-green-600' : 'text-red-600'
                    }`}>
                      {epsGrowth ? `${epsGrowth.toFixed(2)}%` : 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Current Price</span>
                    <span className="font-medium">
                      ${stock.currentPrice ? stock.currentPrice.toFixed(2) : latestQuarter?.price?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Latest EPS</span>
                    <span className="text-sm font-medium">
                      ${latestQuarter?.eps?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Previous EPS</span>
                    <span className="text-sm font-medium">
                      ${previousQuarter?.eps?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Loading state */}
      {isLoading && (
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
            isLoading={isLoading}
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
