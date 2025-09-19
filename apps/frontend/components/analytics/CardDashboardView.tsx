'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Download, Filter, RefreshCw } from 'lucide-react';
import { useEffect, useState } from 'react';
import { AnalyticsClient } from '@/lib/api-client';

// Define types locally to avoid importing from api-client
interface EPSQueryParams {
  page: number;
  limit: number;
  country?: string;
  sector?: string;
  sort_by?: string;
  min_eps?: number;
  min_growth?: number;
}

interface QuarterlyPerformanceData {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
  is_estimated?: boolean;
}

interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: QuarterlyPerformanceData[];
  next_quarter_estimate?: {
    quarter: string;
    announcement_date: string;
    days_until_announcement: number;
    estimated_eps: number;
    estimated_price_target?: number;
    confidence: string;
  };
}

interface CardDashboardResponse {
  success: boolean;
  data: SymbolCardData[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  metadata: {
    available_countries: string[];
    available_sectors: string[];
    request_timestamp: string;
    data_source: string;
  };
  message?: string;
  processing_time_ms: number;
}

// Use the main AnalyticsClient for consistent API handling

interface CardDashboardViewProps {
  className?: string;
}

interface AdvancedFilters {
  country: string;
  sector: string;
  min_eps: number | undefined;
  min_growth: number | undefined;
  sort_by: string;
  page: number;
  limit: number;
}

const analyticsClient = new AnalyticsClient();

export function CardDashboardView({ className = '' }: CardDashboardViewProps) {
  const [data, setData] = useState<CardDashboardResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showFilters, setShowFilters] = useState(false);

  const [filters, setFilters] = useState<AdvancedFilters>({
    country: '',
    sector: '',
    min_eps: undefined,
    min_growth: undefined,
    sort_by: 'growth_factor',
    page: 1,
    limit: 10,
  });

  const [filterOptions, setFilterOptions] = useState({
    countries: [] as Array<{ value: string; label: string }>,
    sectors: [] as string[],
  });

  // Load filter options
  useEffect(() => {
    const loadFilterOptions = async () => {
      try {
        const [countriesResponse, sectorsResponse] = await Promise.all([
          analyticsClient.getAvailableCountries(),
          analyticsClient.getSectorsByCountry(),
        ]);

        setFilterOptions({
          countries: countriesResponse.data?.countries || [],
          sectors: sectorsResponse.data?.sectors || [],
        });
      } catch (error) {
        console.error('Failed to load filter options:', error);
      }
    };

    loadFilterOptions();
  }, []);

  // Load card dashboard data
  const loadData = async () => {
    setLoading(true);
    setError(null);

    try {
      const queryParams: EPSQueryParams = {
        page: filters.page,
        limit: filters.limit,
        sort_by: filters.sort_by,
        country: filters.country || undefined,
        sector: filters.sector || undefined,
        min_eps: filters.min_eps,
        min_growth: filters.min_growth,
      };

      const response = await analyticsClient.getCardDashboard(queryParams);
      if (response.data) {
        setData(response.data);
      }
    } catch (error) {
      console.error('Failed to load card dashboard data:', error);
      setError('Failed to load dashboard data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, [filters]);

  const updateFilters = (newFilters: Partial<AdvancedFilters>) => {
    setFilters(prev => ({
      ...prev,
      ...newFilters,
      page: 'page' in newFilters ? newFilters.page! : 1,
    }));
  };

  const resetFilters = () => {
    setFilters({
      country: '',
      sector: '',
      min_eps: undefined,
      min_growth: undefined,
      sort_by: 'growth_factor',
      page: 1,
      limit: 10,
    });
  };

  const handleExport = (format: 'json' | 'csv') => {
    if (!data) return;

    // Simple export implementation without dependencies
    const exportData = {
      ...data,
      exported_at: new Date().toISOString(),
    };

    if (format === 'json') {
      const blob = new Blob([JSON.stringify(exportData, null, 2)], {
        type: 'application/json',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `analytics-dashboard-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } else {
      // Simple CSV export
      const csvData = data.data.map(item => ({
        symbol: item.symbol,
        rank: item.rank,
        value: item.value,
        status: item.active_status,
        latest_date: item.latest_date,
        quarters: item.quarterly_performance.length,
      }));

      const headers = Object.keys(csvData[0] || {}).join(',');
      const rows = csvData.map(row => Object.values(row).join(','));
      const csv = [headers, ...rows].join('\n');

      const blob = new Blob([csv], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `analytics-dashboard-${new Date().toISOString().split('T')[0]}.csv`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  };

  const getSystemModeColor = (mode: string) => {
    switch (mode) {
      case 'TRACK':
        return 'bg-green-500';
      case 'STOP':
        return 'bg-red-500';
      default:
        return 'bg-gray-400';
    }
  };

  const getSystemModeIcon = (mode: string) => {
    switch (mode) {
      case 'TRACK':
        return '🟢';
      case 'STOP':
        return '🔴';
      default:
        return '⚪';
    }
  };

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  };

  const formatPercentage = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
  };

  const SymbolCard = ({ cardData }: { cardData: SymbolCardData }) => {
    const quarters = cardData.quarterly_performance?.slice(0, 2) || [];
    const latestQuarter = quarters[0];
    const previousQuarter = quarters[1];

    // Map status to action with visual indicators
    const getActionInfo = (status: string) => {
      switch (status) {
        case 'TRACK':
          return { action: 'KEEP', emoji: '🟢' };
        case 'STOP':
          return { action: 'PAUSE', emoji: '🔴' };
        default:
          return { action: 'KEEP', emoji: '🟢' };
      }
    };

    const actionInfo = getActionInfo(cardData.active_status);
    const daysUntil =
      cardData.next_quarter_estimate?.days_until_announcement || 185;
    const nextDate =
      cardData.next_quarter_estimate?.announcement_date || 'Feb 28, 2026';

    // Progress bar calculation using exact dates
    // Start: Latest EPS date, End: Next EPS announcement date
    const currentEPSDate = new Date(latestQuarter?.date || 'Jul 30, 2025');
    const nextEPSDate = new Date(nextDate);
    const totalDays = Math.ceil(
      (nextEPSDate.getTime() - currentEPSDate.getTime()) / (1000 * 60 * 60 * 24)
    );
    const daysPassed = Math.max(0, totalDays - daysUntil);
    const progressPercentage =
      totalDays > 0
        ? Math.max(0, Math.min(100, (daysPassed / totalDays) * 100))
        : 0;

    return (
      <div className="mx-auto w-full max-w-sm touch-manipulation overflow-hidden rounded-3xl border-2 border-transparent bg-white shadow-2xl shadow-pink-500/20 transition-all duration-300 hover:border-pink-200 dark:bg-slate-900 dark:shadow-cyan-500/20 dark:hover:border-cyan-400/50">
        {/* Header with PancakeSwap gradient */}
        <div className="bg-gradient-to-r from-pink-400 via-purple-500 to-indigo-600 px-6 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <span className="text-4xl font-black tracking-wide text-white drop-shadow-lg">
                {cardData.symbol}
              </span>
              <span className="text-2xl font-bold text-pink-100 opacity-80">
                #{cardData.rank}
              </span>
            </div>
            <a
              href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xl text-white transition-transform hover:scale-125 hover:text-yellow-300"
            >
              🔗
            </a>
          </div>
        </div>

        {/* Main Action with colorful background */}
        <div className="bg-gradient-to-br from-pink-50 to-purple-50 py-8 text-center dark:from-slate-800 dark:to-slate-700">
          <div className="inline-flex items-center gap-3 rounded-full border border-pink-200/50 bg-white px-6 py-3 shadow-lg dark:border-cyan-400/30 dark:bg-slate-800">
            <span className="text-2xl">{actionInfo.emoji}</span>
            <span className="bg-gradient-to-r from-pink-600 to-purple-600 bg-clip-text text-2xl font-bold text-transparent dark:from-cyan-400 dark:to-blue-400">
              {actionInfo.action}
            </span>
          </div>
        </div>

        {/* Progress Bar - PancakeSwap style */}
        <div className="bg-gradient-to-br from-pink-50 to-purple-50 px-6 pb-6 dark:from-slate-800 dark:to-slate-700">
          {/* Phase Label */}
          <div className="mb-3 text-center">
            <span className="text-sm font-semibold text-purple-700 dark:text-cyan-300">
              Action Phase
            </span>
          </div>

          {/* Date Labels */}
          <div className="mb-2 flex justify-between text-xs font-medium text-purple-600 dark:text-cyan-400">
            <span>{latestQuarter?.date || 'Jul 30, 2025'}</span>
            <span>
              {nextDate.includes(',') ? nextDate.split(',')[0] : nextDate}
            </span>
          </div>

          <div className="flex items-center gap-4">
            <div className="h-6 flex-1 rounded-full bg-pink-100 shadow-inner dark:bg-slate-600">
              <div
                className="h-6 rounded-full bg-gradient-to-r from-pink-400 via-purple-500 to-indigo-500 shadow-sm transition-all duration-700"
                style={{ width: `${progressPercentage}%` }}
              />
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600 dark:text-cyan-400">
                {daysUntil}
              </div>
              <div className="text-xs font-medium text-purple-500 dark:text-cyan-300">
                days
              </div>
            </div>
          </div>
        </div>

        {/* Main Growth with enhanced styling */}
        <div className="bg-white py-6 text-center dark:bg-slate-900">
          <div
            className={`inline-flex items-center gap-2 rounded-2xl px-4 py-2 shadow-lg ${
              (latestQuarter?.eps_growth || 0) >= 0
                ? 'bg-gradient-to-r from-green-400 to-emerald-500'
                : 'bg-gradient-to-r from-red-400 to-red-500'
            }`}
          >
            <span className="text-2xl">
              {(latestQuarter?.eps_growth || 0) >= 0 ? '↗️' : '↘️'}
            </span>
            <span className="text-2xl font-bold text-white drop-shadow-sm">
              {formatPercentage(latestQuarter?.eps_growth || 0)}
            </span>
          </div>
        </div>

        {/* Separator */}
        <div className="bg-white px-6 dark:bg-slate-900">
          <div className="border-t-2 border-dashed border-pink-200 dark:border-cyan-400/30" />
        </div>

        {/* Expanded Content with card-like sections */}
        <div className="space-y-4 bg-white px-6 pt-6 pb-6 dark:bg-slate-900">
          {quarters.length >= 2 && (
            <>
              {/* Previous Quarter */}
              <div className="rounded-2xl border border-purple-200/50 bg-gradient-to-r from-purple-50 to-pink-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
                <div className="mb-2 text-sm font-semibold text-purple-600 dark:text-cyan-400">
                  {previousQuarter?.date || 'Apr 30, 2025'}
                </div>
                <div className="space-y-1 text-sm text-purple-700 dark:text-cyan-200">
                  <div>
                    • Growth:{' '}
                    {formatPercentage(previousQuarter?.eps_growth || 0)} | EPS:{' '}
                    {(previousQuarter?.eps || 0).toFixed(2)}
                  </div>
                  <div>
                    • Price:{' '}
                    {formatPercentage(previousQuarter?.price_growth || 0)} |{' '}
                    {formatCurrency(previousQuarter?.price || 0)}
                  </div>
                </div>
              </div>

              {/* Latest Quarter */}
              <div className="rounded-2xl border border-green-200/50 bg-gradient-to-r from-green-50 to-emerald-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
                <div className="mb-2 text-sm font-semibold text-green-600 dark:text-cyan-400">
                  {latestQuarter?.date || 'Jul 30, 2025'}
                </div>
                <div className="space-y-1 text-sm text-green-700 dark:text-cyan-200">
                  <div>
                    • Growth: {formatPercentage(latestQuarter?.eps_growth || 0)}{' '}
                    | EPS: {(latestQuarter?.eps || 0).toFixed(2)}
                  </div>
                  <div>
                    • Price:{' '}
                    {formatPercentage(latestQuarter?.price_growth || 0)} |{' '}
                    {formatCurrency(latestQuarter?.price || 0)}
                  </div>
                </div>
              </div>
            </>
          )}

          {/* Next Checkpoint */}
          <div className="rounded-2xl border border-amber-200/50 bg-gradient-to-r from-amber-50 to-orange-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
            <div className="text-sm font-semibold text-amber-600 dark:text-cyan-400">
              Next: {nextDate.includes(',') ? nextDate.split(',')[0] : nextDate}
            </div>
          </div>
        </div>
      </div>
    );
  };

  if (loading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {Array.from({ length: 8 }).map((_, i) => (
            <Card key={i} className="animate-pulse">
              <CardHeader>
                <div className="mb-2 h-6 rounded bg-gray-200" />
                <div className="h-4 w-3/4 rounded bg-gray-200" />
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {Array.from({ length: 3 }).map((_, j) => (
                    <div key={j} className="flex justify-between">
                      <div className="h-3 w-1/3 rounded bg-gray-200" />
                      <div className="h-3 w-1/4 rounded bg-gray-200" />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`py-12 text-center ${className}`}>
        <p className="mb-4 text-red-600">{error}</p>
        <Button onClick={loadData}>Try Again</Button>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header with filters */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-2xl font-bold">
            <span className="mr-2">📋</span>
            <span className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
              Performance Watch
            </span>
          </h2>
          {data && (
            <p className="text-gray-600 dark:text-gray-300">
              Showing {data.data.length} of {data.pagination.total} companies
            </p>
          )}
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center gap-2"
          >
            <Filter className="h-4 w-4" />
            Filters
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={loadData}
            className="flex items-center gap-2"
            disabled={loading}
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>

          <Select onValueChange={handleExport} disabled={!data || loading}>
            <SelectTrigger className="w-24">
              <Download className="h-4 w-4" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="json">Export JSON</SelectItem>
              <SelectItem value="csv">Export CSV</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Advanced filters */}
      {showFilters && (
        <Card className="p-6">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
            <div>
              <Label htmlFor="country">Country</Label>
              <Select
                value={filters.country}
                onValueChange={value => updateFilters({ country: value })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All Countries" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">All Countries</SelectItem>
                  {filterOptions.countries.map(country => (
                    <SelectItem key={country.value} value={country.value}>
                      {country.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="sector">Sector</Label>
              <Select
                value={filters.sector}
                onValueChange={value => updateFilters({ sector: value })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All Sectors" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">All Sectors</SelectItem>
                  {filterOptions.sectors.map(sector => (
                    <SelectItem key={sector} value={sector}>
                      {sector}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="min_eps">Min EPS</Label>
              <Input
                type="number"
                value={filters.min_eps || ''}
                onChange={e =>
                  updateFilters({
                    min_eps: e.target.value
                      ? parseFloat(e.target.value)
                      : undefined,
                  })
                }
                placeholder="0.0"
                step="0.1"
              />
            </div>

            <div>
              <Label htmlFor="min_growth">Min Growth %</Label>
              <Input
                type="number"
                value={filters.min_growth || ''}
                onChange={e =>
                  updateFilters({
                    min_growth: e.target.value
                      ? parseFloat(e.target.value)
                      : undefined,
                  })
                }
                placeholder="0.0"
                step="0.1"
              />
            </div>
          </div>

          <div className="mt-4 flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={resetFilters}>
              Reset Filters
            </Button>
          </div>
        </Card>
      )}

      {/* Status Legend */}
      <div className="mb-6 rounded-lg border border-slate-600 bg-slate-800/90 p-4">
        <div className="mb-4 flex items-center gap-2">
          <span className="text-blue-400">📊</span>
          <h3 className="text-lg font-semibold text-white">Status Legend</h3>
        </div>

        <div className="mb-3">
          <h4 className="mb-3 text-sm font-medium text-slate-300">
            Status Indicators
          </h4>
        </div>

        <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
          <div className="flex items-center gap-3 rounded-lg border border-slate-600/40 bg-slate-700/60 p-3 transition-all duration-200 hover:bg-slate-700">
            <span className="inline-flex items-center rounded bg-green-500 px-2 py-1 text-xs font-medium text-white">
              TRACK
            </span>
            <span className="text-sm text-slate-300">
              Strong performance, actively tracking
            </span>
          </div>

          <div className="flex items-center gap-3 rounded-lg border border-slate-600/40 bg-slate-700/60 p-3 transition-all duration-200 hover:bg-slate-700 md:col-span-1">
            <span className="inline-flex items-center rounded bg-red-500 px-2 py-1 text-xs font-medium text-white">
              STOP
            </span>
            <span className="text-sm text-slate-300">
              Weak performance, avoid investment
            </span>
          </div>
        </div>
      </div>

      {/* Processing time indicator */}
      {data && (
        <div className="text-center text-sm text-gray-500 dark:text-gray-400">
          Processed in {data.processing_time_ms} ms
        </div>
      )}

      {/* Cards grid */}
      {data && data.data && data.data.length > 0 ? (
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {data.data.map(cardData =>
            cardData && cardData.symbol ? (
              <SymbolCard key={cardData.symbol} cardData={cardData} />
            ) : null
          )}
        </div>
      ) : (
        <div className="py-12 text-center">
          <p className="mb-4 text-gray-600 dark:text-gray-300">
            No data available
          </p>
          <Button onClick={resetFilters}>Reset Filters</Button>
        </div>
      )}

      {/* Pagination */}
      {data && data.pagination && data.pagination.totalPages > 1 && (
        <div className="mt-8 flex items-center justify-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => updateFilters({ page: filters.page - 1 })}
            disabled={!data.pagination.hasPrev}
          >
            Previous
          </Button>
          <span className="text-sm text-gray-600 dark:text-gray-300">
            Page {filters.page} of {data.pagination.totalPages}
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => updateFilters({ page: filters.page + 1 })}
            disabled={!data.pagination.hasNext}
          >
            Next
          </Button>
        </div>
      )}
    </div>
  );
}
