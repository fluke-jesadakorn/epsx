'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
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
import { Pagination } from '@/components/ui/pagination';
import RoleBasedFinancialTable from '@/components/shared/RoleBasedFinancialTable';
import { BarChart3, Crown, Lock, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';

export function AnalyticsRankingDashboard() {
  const router = useRouter();
  const { loading } = useRankingAccess();
  const { getMaxAllowedLimit, canAccessPage, getAvailablePageSizes, userTier } =
    usePaginatedFeatureAccess();

  const [epsData, setEpsData] = useState<{
    data: Array<{
      id: string;
      symbol: string;
      company_name: string;
      current_eps: number;
      qoq_growth: number;
      market_cap: number;
      price_current: number;
      volume: number;
      country: string;
      sector: string;
      ranking_score: number;
    }>;
    pagination: {
      page: number;
      limit: number;
      total: number;
      totalPages: number;
      hasNext: boolean;
      hasPrev: boolean;
    };
  }>({
    data: [],
    pagination: {
      page: 1,
      limit: 10,
      total: 0,
      totalPages: 0,
      hasNext: false,
      hasPrev: false,
    },
  });
  const [dataLoading, setDataLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCountry, setSelectedCountry] = useState<string>('');
  const [availableCountries, setAvailableCountries] = useState<string[]>([]);

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
        const response = await analyticsClient.getEPSRankings({
          page,
          limit,
          country: selectedCountry || undefined,
          sort_by: 'qoq_growth'
        });
        
        if (response.success) {
          setEpsData(response.data);
        } else {
          throw new Error(response.error || 'Failed to fetch EPS data');
        }
      } catch (error) {
        console.error('Error fetching EPS data:', error);
        setError('Failed to load EPS data. Please try again.');
      } finally {
        setPaginationLoading(false);
      }
    },
  });

  useEffect(() => {
    const loadInitialData = async () => {
      try {
        setDataLoading(true);
        
        // Load countries list
        const countriesResponse = await analyticsClient.getEPSCountries();
        if (countriesResponse.success) {
          setAvailableCountries(['All Countries', ...countriesResponse.data.countries]);
        }
        
        // Load initial EPS data
        const epsResponse = await analyticsClient.getEPSRankings({
          page: 1,
          limit: 10,
          sort_by: 'qoq_growth'
        });
        
        if (epsResponse.success) {
          setEpsData(epsResponse.data);
        } else {
          throw new Error(epsResponse.error || 'Failed to fetch EPS data');
        }
      } catch (error) {
        console.error('Failed to load initial analytics data:', error);
        setError('Failed to load initial data. Please try again.');
      } finally {
        setDataLoading(false);
      }
    };

    if (!loading) {
      loadInitialData();
    }
  }, [loading]);

  const maxAllowedLimit = getMaxAllowedLimit();
  const availablePageSizes = getAvailablePageSizes();
  const currentPageAccessible = canAccessPage(currentPage, limit);

  const handleUpgrade = () => {
    router.push('/payment');
  };

  const handleRetry = () => {
    handlePageChange(currentPage);
  };

  const handleCountryChange = async (country: string) => {
    const countryFilter = country === 'All Countries' ? '' : country;
    setSelectedCountry(countryFilter);
    
    // Reload data with new country filter
    try {
      setPaginationLoading(true);
      setError(null);
      
      const response = await analyticsClient.getEPSRankings({
        page: 1,
        limit,
        country: countryFilter || undefined,
        sort_by: 'qoq_growth'
      });
      
      if (response.success) {
        setEpsData(response.data);
        // Reset to page 1 when country changes
        handlePageChange(1);
      } else {
        throw new Error(response.error || 'Failed to fetch EPS data');
      }
    } catch (error) {
      console.error('Error changing country filter:', error);
      setError('Failed to load data for selected country. Please try again.');
    } finally {
      setPaginationLoading(false);
    }
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
          </Badge>
        </div>
      </div>

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
            {Math.min((currentPage - 1) * limit + 1, epsData.pagination.total)}{' '}
            to {Math.min(currentPage * limit, epsData.pagination.total)} of{' '}
            {epsData.pagination.total} results
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
          <h3 className="text-2xl font-bold">Stock Rankings</h3>
          <Badge className={levelInfo.color}>
            {currentPageAccessible ? `Page ${currentPage}` : 'Limited Access'}
          </Badge>
        </div>

        {epsData.data.length > 0 ? (
          <RoleBasedFinancialTable data={epsData.data} />
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
      {epsData.pagination.totalPages > 1 && (
        <div className="space-y-4">
          <Pagination
            currentPage={currentPage}
            totalPages={epsData.pagination.totalPages}
            onPageChange={handlePageChange}
            hasNext={epsData.pagination.hasNext}
            hasPrev={epsData.pagination.hasPrev}
            isLoading={paginationLoading}
            className="mt-8"
          />

          {/* Upgrade prompt for pagination */}
          {epsData.pagination.totalPages > 1 && userTier === 'BASIC' && (
            <Card className="border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20">
              <CardContent className="p-6 text-center">
                <div className="space-y-4">
                  <div className="flex justify-center">
                    <Crown className="h-12 w-12 text-yellow-500" />
                  </div>
                  <div>
                    <h3 className="text-lg font-bold mb-2">
                      🚀 Unlock Full EPS Analytics Access
                    </h3>
                    <p className="text-sm text-muted-foreground mb-2">
                      You&apos;re seeing limited results. Upgrade to access all{' '}
                      {epsData.pagination.total} EPS rankings!
                    </p>
                    <div className="flex flex-wrap justify-center gap-2 text-xs">
                      <Badge variant="secondary">📊 Full EPS Rankings</Badge>
                      <Badge variant="secondary">🎯 Country Filtering</Badge>
                      <Badge variant="secondary">💎 Premium Analytics</Badge>
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
