'use client';

import { AnalyticsNavigation } from '@/components/shared/analytics-navigation';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useAnalyticsFilters } from '@/hooks/use-analytics-filters';
import type {
  // AnalyticsClient, 
  UnifiedAnalyticsRankingsResponse
} from '@/lib/api-client';
import { analyticsClient } from '@/lib/api-client';
import type {
  ExportFormat} from '@/lib/export-utils';
import {
  exportCurrentViewData,
  exportFilteredData,
  exportGrowthLeadersData,
  exportUnifiedAnalyticsData
} from '@/lib/export-utils';
import { StockDataCard } from '@/shared/components/cards/stock-data-card';
import type { AnalyticsFilters } from '@/types/analytics';
import { Download, FileDown, LayoutGrid, List } from 'lucide-react';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';
import { CardDashboardView } from './card-dashboard-view';
import FilterPanel from './filter-panel';
import Pagination from './pagination';

// Rich filter options interface
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

// API helper functions using new AnalyticsClient
async function fetchEPSRankings(filters: AnalyticsFilters): Promise<UnifiedAnalyticsRankingsResponse | null> {
  try {
    const queryParams = {
      page: filters.page,
      per_page: filters.limit,
      sort_by: filters.sort_by,
      country: filters.country,
      sector: filters.sector,
      min_market_cap: filters.min_eps,
      sort_order: 'desc' as const,
    };

    const response = await analyticsClient.getRankings(queryParams);
    return response || null;
  } catch (error) {
    console.error('Error fetching EPS rankings:', error);
    return null;
  }
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

  // Memoized calculation of Growth leaders - expensive array operations
  const calculateGrowthLeaders = useCallback((data: UnifiedAnalyticsRankingsResponse | null) => {
    if (!data?.rankings || data.rankings.length === 0) {return { growthLeaders: [], priceLeaders: [] };}

    // Calculate Growth Factor leaders using epsGrowth
    const growthLeaders = data.rankings
      .filter(ranking => ranking.epsGrowth !== null && ranking.epsGrowth !== undefined)
      .sort((a, b) => (b.epsGrowth || 0) - (a.epsGrowth || 0))
      .slice(0, 3);

    // For price leaders, use momentum data since quarterly_data is not available
    const priceLeaders = data.rankings
      .filter(ranking => ranking.momentum_1m !== null && ranking.momentum_1m !== undefined)
      .sort((a, b) => (b.momentum_1m || 0) - (a.momentum_1m || 0))
      .slice(0, 3);

    return { growthLeaders, priceLeaders };
  }, []);

  const [data, setData] = useState<UnifiedAnalyticsRankingsResponse | null>(initialData);
  const [error, setError] = useState<string | null>(null);
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [activeTab, setActiveTab] = useState('list');

  // Export functionality state
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [exportFormat, setExportFormat] = useState<ExportFormat>('json');
  const [exportFilename, setExportFilename] = useState('');
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeQuarterlyData, setIncludeQuarterlyData] = useState(true);
  const [exportType, setExportType] = useState<'current' | 'filtered' | 'leaders' | 'full'>('current');

  // Memoized growth leaders calculation to avoid repeated expensive operations
  const { growthLeaders, priceLeaders } = useMemo(() => {
    return calculateGrowthLeaders(data);
  }, [data, calculateGrowthLeaders]);

  // Load data when filters change (skip on initial load if we have initialData)
  useEffect(() => {
    // Only fetch if filters have changed from default
    const hasNonDefaultFilters = filters.page !== 1 ||
      filters.country ||
      filters.sector ||
      filters.min_eps ||
      filters.min_growth;

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
        } catch (err) {
          console.error('Error loading analytics data:', err);
          setError('An error occurred while loading data');
        } finally {
          setIsLoading(false);
        }
      };

      loadData();
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

  // Export handlers
  const handleExport = () => {
    if (!data) {return;}

    const options = {
      format: exportFormat,
      filename: exportFilename || undefined,
      includeMetadata,
      includeQuarterlyData,
    };

    switch (exportType) {
      case 'current':
        exportCurrentViewData(data.rankings, options);
        break;
      case 'filtered':
        exportFilteredData(data.rankings, filters, options);
        break;
      case 'leaders':
        // Use memoized growth leaders
        exportGrowthLeadersData([...growthLeaders, ...priceLeaders], options);
        break;
      case 'full':
        exportUnifiedAnalyticsData(data.rankings, options);
        break;
    }

    setShowExportDialog(false);
  };

  const getExportDescription = () => {
    switch (exportType) {
      case 'current':
        return `Export current page data (${data?.rankings?.length || 0} records)`;
      case 'filtered':
        return `Export all filtered data (${data?.pagination?.total_items || 0} total records)`;
      case 'leaders':
        return 'Export Growth performance leaders only';
      case 'full':
        return 'Export complete dataset with metadata';
      default:
        return '';
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

          {/* Enhanced Growth Leaders Section */}
          {!isLoading && data && data.rankings.length > 0 && (() => {
            // Use memoized growth leaders
            return (
              <div className="mb-8">
                <div className="relative overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-6 sm:p-8 shadow-2xl backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
                  {/* Enhanced background decorations */}
                  <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-yellow-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-yellow-900/10" />
                  <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 blur-2xl" />
                  <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 blur-2xl" />

                  <div className="relative z-10">
                    <div className="mb-6 text-center sm:text-left">
                      <h2 className="mb-3 text-xl font-bold sm:text-2xl">
                        <span className="mr-2">🏆</span>
                        <span className="animate-gradient-x bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent">
                          Growth Performance Leaders
                        </span>
                      </h2>
                      <p className="text-gray-600 dark:text-gray-300">
                        Top performers in growth factor and price growth
                      </p>
                    </div>

                    <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
                      {/* Enhanced Growth Factor Leaders */}
                      <div className="rounded-2xl border border-green-200/50 bg-gradient-to-br from-green-50/80 to-emerald-50/80 p-5 backdrop-blur-sm transition-all duration-300 hover:scale-[1.02] dark:border-green-400/20 dark:from-green-900/20 dark:to-emerald-900/20">
                        <h3 className="mb-4 flex items-center gap-3 text-sm font-bold text-green-700 dark:text-green-400">
                          <div className="rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 p-2">
                            <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                            </svg>
                          </div>
                          Best Growth Factor
                        </h3>
                        <div className="space-y-3">
                          {growthLeaders.slice(0, 3).map((leader, index) => (
                            <div key={leader.symbol} className="flex items-center justify-between rounded-xl bg-white/60 p-3 backdrop-blur-sm dark:bg-slate-800/60">
                              <div className="flex items-center gap-3">
                                <span className="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-green-500 to-emerald-500 text-xs font-bold text-white shadow-lg">
                                  {index + 1}
                                </span>
                                <div>
                                  <span className="font-semibold text-gray-900 dark:text-gray-100">{leader.symbol}</span>
                                  <p className="text-xs text-gray-500 dark:text-gray-400">{leader.companyName}</p>
                                </div>
                              </div>
                              <div className="text-right">
                                <span className="rounded-lg bg-gradient-to-r from-green-500 to-emerald-500 px-3 py-1 text-sm font-bold text-white shadow-md">
                                  +{(leader.epsGrowth || 0).toFixed(1)}%
                                </span>
                                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                  Score: {(leader.score || 0).toFixed(1)}
                                </p>
                              </div>
                            </div>
                          ))}
                          {growthLeaders.length === 0 && (
                            <div className="rounded-xl bg-white/60 p-4 text-center backdrop-blur-sm dark:bg-slate-800/60">
                              <p className="text-sm text-gray-500 dark:text-gray-400">No growth factor data available</p>
                            </div>
                          )}
                        </div>
                      </div>

                      {/* Enhanced Price Growth Leaders */}
                      <div className="rounded-2xl border border-blue-200/50 bg-gradient-to-br from-blue-50/80 to-cyan-50/80 p-5 backdrop-blur-sm transition-all duration-300 hover:scale-[1.02] dark:border-blue-400/20 dark:from-blue-900/20 dark:to-cyan-900/20">
                        <h3 className="mb-4 flex items-center gap-3 text-sm font-bold text-blue-700 dark:text-blue-400">
                          <div className="rounded-xl bg-gradient-to-r from-blue-500 to-cyan-500 p-2">
                            <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
                            </svg>
                          </div>
                          Best Price Growth
                        </h3>
                        <div className="space-y-3">
                          {priceLeaders.slice(0, 3).map((leader, index) => {
                            // Use momentum data since quarterly_data is not available
                            const priceGrowth = leader.momentum_1m || 0;

                            return (
                              <div key={leader.symbol} className="flex items-center justify-between rounded-xl bg-white/60 p-3 backdrop-blur-sm dark:bg-slate-800/60">
                                <div className="flex items-center gap-3">
                                  <span className="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-blue-500 to-cyan-500 text-xs font-bold text-white shadow-lg">
                                    {index + 1}
                                  </span>
                                  <div>
                                    <span className="font-semibold text-gray-900 dark:text-gray-100">{leader.symbol}</span>
                                    <p className="text-xs text-gray-500 dark:text-gray-400">{leader.companyName}</p>
                                  </div>
                                </div>
                                <div className="text-right">
                                  <span className={`rounded-lg px-3 py-1 text-sm font-bold text-white shadow-md ${priceGrowth >= 0
                                      ? 'bg-gradient-to-r from-blue-500 to-cyan-500'
                                      : 'bg-gradient-to-r from-red-500 to-pink-500'
                                    }`}>
                                    {priceGrowth >= 0 ? '+' : ''}{priceGrowth.toFixed(1)}%
                                  </span>
                                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                    Vol: {(leader.volatility || 0).toFixed(1)}
                                  </p>
                                </div>
                              </div>
                            );
                          })}
                          {priceLeaders.length === 0 && (
                            <div className="rounded-xl bg-white/60 p-4 text-center backdrop-blur-sm dark:bg-slate-800/60">
                              <p className="text-sm text-gray-500 dark:text-gray-400">No price growth data available</p>
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            );
          })()}

          {/* Rich Backend Analytics Metadata Section */}
          {!isLoading && data && data.metadata && (
            <div className="mb-8">
              <div className="relative overflow-hidden rounded-3xl border border-purple-200/50 bg-white/80 p-6 shadow-2xl backdrop-blur-xl dark:border-purple-400/20 dark:bg-slate-800/80">
                <div className="absolute inset-0 bg-gradient-to-br from-purple-50/50 via-transparent to-blue-50/50 dark:from-purple-900/10 dark:via-transparent dark:to-blue-900/10" />

                <div className="relative z-10">
                  <div className="mb-6 text-center sm:text-left">
                    <h2 className="mb-3 text-xl font-bold sm:text-2xl">
                      <span className="mr-2">🚀</span>
                      <span className="animate-gradient-x bg-gradient-to-r from-purple-500 via-blue-500 to-purple-600 bg-clip-text text-transparent">
                        Advanced Analytics Engine
                      </span>
                    </h2>
                    <p className="text-gray-600 dark:text-gray-300">
                      Powered by Diesel ORM with server-side rendering and intelligent caching
                    </p>
                  </div>

                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    <div className="rounded-2xl border border-green-200/50 bg-gradient-to-br from-green-50/80 to-emerald-50/80 p-4 backdrop-blur-sm dark:border-green-400/20 dark:from-green-900/20 dark:to-emerald-900/20">
                      <div className="flex items-center gap-3 mb-2">
                        <div className="rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 p-2">
                          <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                          </svg>
                        </div>
                        <span className="text-sm font-semibold text-green-700 dark:text-green-400">Processing Time</span>
                      </div>
                      <p className="text-lg font-bold text-green-800 dark:text-green-300">{data.metadata.query_time}ms</p>
                    </div>

                    <div className="rounded-2xl border border-blue-200/50 bg-gradient-to-br from-blue-50/80 to-cyan-50/80 p-4 backdrop-blur-sm dark:border-blue-400/20 dark:from-blue-900/20 dark:to-cyan-900/20">
                      <div className="flex items-center gap-3 mb-2">
                        <div className="rounded-xl bg-gradient-to-r from-blue-500 to-cyan-500 p-2">
                          <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
                          </svg>
                        </div>
                        <span className="text-sm font-semibold text-blue-700 dark:text-blue-400">Architecture</span>
                      </div>
                      <p className="text-sm font-bold text-blue-800 dark:text-blue-300">
                        Server Components
                      </p>
                    </div>

                    <div className="rounded-2xl border border-orange-200/50 bg-gradient-to-br from-orange-50/80 to-yellow-50/80 p-4 backdrop-blur-sm dark:border-orange-400/20 dark:from-orange-900/20 dark:to-yellow-900/20">
                      <div className="flex items-center gap-3 mb-2">
                        <div className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 p-2">
                          <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                          </svg>
                        </div>
                        <span className="text-sm font-semibold text-orange-700 dark:text-orange-400">Data Source</span>
                      </div>
                      <p className="text-sm font-bold text-orange-800 dark:text-orange-300">Analytics API</p>
                    </div>

                    <div className="rounded-2xl border border-purple-200/50 bg-gradient-to-br from-purple-50/80 to-pink-50/80 p-4 backdrop-blur-sm dark:border-purple-400/20 dark:from-purple-900/20 dark:to-pink-900/20">
                      <div className="flex items-center gap-3 mb-2">
                        <div className="rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 p-2">
                          <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                          </svg>
                        </div>
                        <span className="text-sm font-semibold text-purple-700 dark:text-purple-400">Markets</span>
                      </div>
                      <p className="text-sm font-bold text-purple-800 dark:text-purple-300">
                        Global Markets
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
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
                            Page {filters.page} of {data?.pagination.total_pages || 1}
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

                        <Dialog open={showExportDialog} onOpenChange={setShowExportDialog}>
                          <DialogTrigger asChild>
                            <button
                              className="flex items-center gap-2 rounded-xl border border-purple-200 bg-white/80 px-4 py-2 text-sm font-semibold text-purple-700 shadow-lg backdrop-blur-sm transition-all duration-300 hover:bg-purple-50 hover:scale-105 dark:border-purple-400/20 dark:bg-slate-800/80 dark:text-purple-400 dark:hover:bg-slate-700/80"
                              disabled={!data || isLoading}
                            >
                              <Download className="h-4 w-4" />
                              <span className="hidden sm:inline">Export Data</span>
                              <span className="sm:hidden">Export</span>
                            </button>
                          </DialogTrigger>
                          <DialogContent className="sm:max-w-md">
                            <DialogHeader>
                              <DialogTitle className="flex items-center gap-2">
                                <FileDown className="h-5 w-5 text-purple-600" />
                                Export Analytics Data
                              </DialogTitle>
                            </DialogHeader>

                            <div className="space-y-6 py-4">
                              {/* Export type selection */}
                              <div>
                                <Label className="text-sm font-medium">Export Type</Label>
                                <Select value={exportType} onValueChange={(value: any) => setExportType(value)}>
                                  <SelectTrigger>
                                    <SelectValue />
                                  </SelectTrigger>
                                  <SelectContent>
                                    <SelectItem value="current">Current Page</SelectItem>
                                    <SelectItem value="filtered">Filtered Data</SelectItem>
                                    <SelectItem value="leaders">Growth Leaders</SelectItem>
                                    <SelectItem value="full">Full Dataset</SelectItem>
                                  </SelectContent>
                                </Select>
                                <p className="text-xs text-gray-500 mt-1">{getExportDescription()}</p>
                              </div>

                              {/* Format selection */}
                              <div>
                                <Label className="text-sm font-medium">Format</Label>
                                <Select value={exportFormat} onValueChange={(value: ExportFormat) => setExportFormat(value)}>
                                  <SelectTrigger>
                                    <SelectValue />
                                  </SelectTrigger>
                                  <SelectContent>
                                    <SelectItem value="json">JSON</SelectItem>
                                    <SelectItem value="csv">CSV</SelectItem>
                                  </SelectContent>
                                </Select>
                              </div>

                              {/* Filename input */}
                              <div>
                                <Label className="text-sm font-medium">Filename (optional)</Label>
                                <Input
                                  value={exportFilename}
                                  onChange={(e) => setExportFilename(e.target.value)}
                                  placeholder="Leave empty for auto-generated name"
                                />
                              </div>

                              {/* Options */}
                              <div className="space-y-3">
                                <div className="flex items-center space-x-2">
                                  <Checkbox
                                    id="metadata"
                                    checked={includeMetadata}
                                    onCheckedChange={(checked) => setIncludeMetadata(Boolean(checked))}
                                  />
                                  <Label htmlFor="metadata" className="text-sm">Include metadata</Label>
                                </div>
                                <div className="flex items-center space-x-2">
                                  <Checkbox
                                    id="quarterly"
                                    checked={includeQuarterlyData}
                                    onCheckedChange={(checked) => setIncludeQuarterlyData(Boolean(checked))}
                                  />
                                  <Label htmlFor="quarterly" className="text-sm">Include quarterly data</Label>
                                </div>
                              </div>

                              {/* Export button */}
                              <div className="flex justify-end space-x-2">
                                <Button variant="outline" onClick={() => setShowExportDialog(false)}>
                                  Cancel
                                </Button>
                                <Button
                                  onClick={handleExport}
                                  className="bg-gradient-to-r from-purple-500 to-purple-600 text-white"
                                >
                                  <Download className="h-4 w-4 mr-2" />
                                  Export
                                </Button>
                              </div>
                            </div>
                          </DialogContent>
                        </Dialog>
                      </div>
                    </div>
                  </div>

                  {/* Enhanced Error state */}
                  {error && (
                    <div className="mb-6 rounded-2xl border border-red-200/50 bg-gradient-to-br from-red-50/80 to-pink-50/80 p-6 backdrop-blur-sm dark:border-red-400/20 dark:from-red-900/20 dark:to-pink-900/20">
                      <div className="flex items-center gap-4">
                        <div className="rounded-xl bg-gradient-to-r from-red-500 to-pink-500 p-3">
                          <svg
                            className="h-6 w-6 text-white"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                          </svg>
                        </div>
                        <div className="flex-1">
                          <p className="font-bold text-red-700 dark:text-red-400">Unable to Load Data</p>
                          <p className="mt-1 text-red-600 dark:text-red-300">{error}</p>
                        </div>
                        <button
                          onClick={refreshData}
                          className="rounded-xl bg-gradient-to-r from-red-500 to-pink-500 px-4 py-2 text-sm font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
                        >
                          Try Again
                        </button>
                      </div>
                    </div>
                  )}

                  {/* Loading state - Mobile optimized */}
                  {isLoading && (
                    <>
                      {/* Mobile: Horizontal scroll skeleton */}
                      <div className="mb-6 block sm:hidden">
                        <div className="overflow-x-auto pb-4">
                          <div className="flex gap-3">
                            {Array.from({ length: 4 }).map((_, index) => (
                              <div
                                key={index}
                                className="w-72 flex-shrink-0 animate-pulse rounded-lg border border-gray-200 bg-white p-4"
                              >
                                <div className="mb-3 flex items-center gap-3">
                                  <div className="h-8 w-8 rounded-full bg-gray-200" />
                                  <div className="flex-1">
                                    <div className="mb-1 h-4 rounded bg-gray-200" />
                                    <div className="h-3 w-3/4 rounded bg-gray-200" />
                                  </div>
                                  <div className="h-6 w-16 rounded-full bg-gray-200" />
                                </div>
                                <div className="mb-3 grid grid-cols-2 gap-3">
                                  <div className="rounded-lg bg-gray-100 p-3">
                                    <div className="mb-1 h-3 rounded bg-gray-200" />
                                    <div className="h-4 rounded bg-gray-200" />
                                  </div>
                                  <div className="rounded-lg bg-gray-100 p-3">
                                    <div className="mb-1 h-3 rounded bg-gray-200" />
                                    <div className="h-4 rounded bg-gray-200" />
                                  </div>
                                </div>
                                <div className="mt-4 h-10 w-full rounded-lg bg-gray-200" />
                              </div>
                            ))}
                          </div>
                        </div>
                      </div>

                      {/* Desktop: Grid skeleton */}
                      <div className="mb-6 hidden sm:grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                        {Array.from({ length: 6 }).map((_, index) => (
                          <div
                            key={index}
                            className="animate-pulse rounded-lg border border-gray-200 bg-white p-4"
                          >
                            <div className="mb-3 flex items-center gap-3">
                              <div className="h-8 w-8 rounded-full bg-gray-200" />
                              <div className="flex-1">
                                <div className="mb-1 h-4 rounded bg-gray-200" />
                                <div className="h-3 w-3/4 rounded bg-gray-200" />
                              </div>
                              <div className="h-6 w-16 rounded-full bg-gray-200" />
                            </div>
                            <div className="mb-3 grid grid-cols-2 gap-3">
                              <div className="rounded-lg bg-gray-100 p-3">
                                <div className="mb-1 h-3 rounded bg-gray-200" />
                                <div className="h-4 rounded bg-gray-200" />
                              </div>
                              <div className="rounded-lg bg-gray-100 p-3">
                                <div className="mb-1 h-3 rounded bg-gray-200" />
                                <div className="h-4 rounded bg-gray-200" />
                              </div>
                            </div>
                            <div className="space-y-2">
                              {Array.from({ length: 4 }).map((_, i) => (
                                <div key={i} className="flex justify-between">
                                  <div className="h-3 w-1/3 rounded bg-gray-200" />
                                  <div className="h-3 w-1/4 rounded bg-gray-200" />
                                </div>
                              ))}
                            </div>
                            <div className="mt-4 h-10 w-full rounded-lg bg-gray-200" />
                          </div>
                        ))}
                      </div>
                    </>
                  )}

                  {/* Results grid - Responsive HTTP Data */}
                  {!isLoading && data && data.rankings.length > 0 && (
                    <>
                      {/* Mobile: Horizontal scrolling cards */}
                      <div className="mb-6 block sm:hidden">
                        <div className="overflow-x-auto pb-4">
                          <div className="flex gap-3">
                            {data.rankings.map((ranking, index) => (
                              <div key={ranking.symbol} className="w-72 flex-shrink-0">
                                <StockDataCard
                                  symbol={ranking.symbol}
                                  rank={ranking.rank || index + 1}
                                  epsGrowth={ranking.epsGrowth || 0}
                                  price={0} // Price not available in API
                                  currency="USD"
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
                        {data.rankings.map(ranking => (
                          <StockDataCard
                            key={ranking.symbol}
                            symbol={ranking.symbol}
                            rank={ranking.rank || 0}
                            epsGrowth={ranking.epsGrowth || 0}
                            price={0} // Price not available in API
                            currency="USD"
                          />
                        ))}
                      </div>

                      {/* Mobile-optimized Pagination */}
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

                  {/* Enhanced Empty state */}
                  {!isLoading && data?.rankings.length === 0 && (
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
                        onClick={resetFilters}
                        className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
                      >
                        Clear All Filters
                      </button>
                    </div>
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