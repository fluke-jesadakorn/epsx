'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
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
// Direct fetch implementation to avoid any axios bundling conflicts
// import { AnalyticsClient, CardDashboardResponse, SymbolCardData, EPSQueryParams } from '@/lib/api-client';

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

// Simple fetch-based client to replace AnalyticsClient
class DirectFetchClient {
  private baseURL: string;

  constructor() {
    this.baseURL = 'http://localhost:8080';
  }

  async getCardDashboard(
    params: EPSQueryParams
  ): Promise<{ data: CardDashboardResponse }> {
    const queryString = new URLSearchParams();
    Object.keys(params).forEach(key => {
      const value = (params as any)[key];
      if (value !== undefined && value !== null) {
        queryString.append(key, String(value));
      }
    });

    const url = `${this.baseURL}/api/v1/analytics/eps-rankings?${queryString.toString()}`;

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    return { data };
  }

  async getAvailableCountries(): Promise<{
    data: { countries: Array<{ value: string; label: string }>; count: number };
  }> {
    const response = await fetch(
      `${this.baseURL}/api/v1/analytics/eps-rankings/countries`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      }
    );

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    return { data };
  }

  async getSectorsByCountry(
    country?: string
  ): Promise<{ data: { sectors: string[]; count: number; country?: string } }> {
    let url = `${this.baseURL}/api/v1/analytics/eps-rankings/sectors`;
    if (country) {
      url += `?country=${encodeURIComponent(country)}`;
    }

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    return { data };
  }
}
// Temporarily disabled to avoid api-client dependency chain
// import { exportCardDashboardData } from '@/lib/export-utils';

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

const analyticsClient = new DirectFetchClient();

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
          countries: countriesResponse.data.countries,
          sectors: sectorsResponse.data.sectors,
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
      setData(response.data);
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
    setFilters(prev => ({ ...prev, ...newFilters, page: 1 }));
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
      case 'WATCH':
        return 'bg-yellow-500';
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
      case 'WATCH':
        return '🟡';
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
    
    return (
      <div className="w-full max-w-sm mx-auto bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-gray-100 dark:border-slate-700 overflow-hidden touch-manipulation">
        {/* Row 1: Header with symbol and status */}
        <div className="px-4 sm:px-6 py-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-full bg-blue-500 dark:bg-blue-600 flex items-center justify-center flex-shrink-0">
              <span className="text-white font-bold text-sm">#{cardData.rank}</span>
            </div>
            <div className="min-w-0 flex-1">
              <h3 className="font-bold text-xl text-gray-900 dark:text-gray-100 truncate">{cardData.symbol}</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 truncate">
                {cardData.latest_date ? new Date(cardData.latest_date).toLocaleDateString() : 'N/A'}
              </p>
            </div>
          </div>
          <div className={`px-3 py-1 rounded-full text-sm font-medium self-start sm:self-auto ${
            cardData.active_status === 'TRACK' 
              ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400' 
              : cardData.active_status === 'WATCH'
              ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400'
              : cardData.active_status === 'STOP'
              ? 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
          }`}>
            {cardData.active_status}
          </div>
        </div>

        {quarters.length >= 2 ? (
          <>
            {/* Row 2: Quarter headers */}
            <div className="px-4 sm:px-6 py-2">
              <div className="grid grid-cols-2 gap-4">
                <div className="text-center">
                  <div className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    {previousQuarter?.quarter || '2025-Q2'}
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    {latestQuarter?.quarter || '2025-Q3'}
                  </div>
                </div>
              </div>
            </div>

            {/* Row 3: EPS Growth label */}
            <div className="px-4 sm:px-6 py-1">
              <div className="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase tracking-wide">
                EPS GROWTH
              </div>
            </div>

            {/* Row 4: EPS Growth percentages */}
            <div className="px-4 sm:px-6 py-2">
              <div className="grid grid-cols-2 gap-4 min-h-[48px] items-center">
                <div className="text-center">
                  <div className={`text-xl sm:text-2xl font-bold ${
                    (previousQuarter?.eps_growth || 0) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                  }`}>
                    {formatPercentage(previousQuarter?.eps_growth || 0)}
                  </div>
                </div>
                <div className="text-center">
                  <div className={`text-xl sm:text-2xl font-bold ${
                    (latestQuarter?.eps_growth || 0) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                  }`}>
                    {formatPercentage(latestQuarter?.eps_growth || 0)}
                  </div>
                </div>
              </div>
            </div>

            {/* Row 5: EPS values */}
            <div className="px-4 sm:px-6 py-2">
              <div className="grid grid-cols-2 gap-4">
                <div className="text-center">
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    <span className="font-medium">{(previousQuarter?.eps || 0).toFixed(2)}</span>
                    <span className="ml-1 text-gray-400 dark:text-gray-500">EPS</span>
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    <span className="font-medium">{(latestQuarter?.eps || 0).toFixed(2)}</span>
                    <span className="ml-1 text-gray-400 dark:text-gray-500">EPS</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Row 6: Price Growth label */}
            <div className="px-4 sm:px-6 py-1 pt-4 border-t border-gray-100 dark:border-slate-700">
              <div className="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase tracking-wide">
                PRICE GROWTH
              </div>
            </div>

            {/* Row 7: Price Growth percentages */}
            <div className="px-4 sm:px-6 py-2">
              <div className="grid grid-cols-2 gap-4 min-h-[44px] items-center">
                <div className="text-center">
                  <div className={`text-lg sm:text-xl font-bold ${
                    (previousQuarter?.price_growth || 0) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                  }`}>
                    {formatPercentage(previousQuarter?.price_growth || 0)}
                  </div>
                </div>
                <div className="text-center">
                  <div className={`text-lg sm:text-xl font-bold ${
                    (latestQuarter?.price_growth || 0) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                  }`}>
                    {formatPercentage(latestQuarter?.price_growth || 0)}
                  </div>
                </div>
              </div>
            </div>

            {/* Row 8: Price values */}
            <div className="px-4 sm:px-6 py-2">
              <div className="grid grid-cols-2 gap-4">
                <div className="text-center">
                  <div className="text-sm font-medium text-gray-600 dark:text-gray-400">
                    {formatCurrency(previousQuarter?.price || 0)}
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm font-medium text-gray-600 dark:text-gray-400">
                    {formatCurrency(latestQuarter?.price || 0)}
                  </div>
                </div>
              </div>
            </div>
          </>
        ) : (
          /* Fallback for single quarter */
          <div className="px-4 sm:px-6 py-4">
            <div className="text-center">
              <p className="text-sm text-gray-500 dark:text-gray-400 mb-2">Current Price</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatCurrency(cardData.value)}</p>
            </div>
          </div>
        )}

        {/* Row 9: Current Price */}
        <div className="px-4 sm:px-6 py-4 border-t border-gray-100 dark:border-slate-700">
          <div className="flex items-center justify-between min-h-[44px]">
            <span className="text-sm text-gray-600 dark:text-gray-400">Current Price:</span>
            <span className="text-xl sm:text-2xl font-bold text-gray-900 dark:text-gray-100">
              {formatCurrency(cardData.value)}
            </span>
          </div>
        </div>

        {/* Row 10: Status */}
        <div className="px-4 sm:px-6 py-3">
          <div className="flex items-center justify-between min-h-[44px]">
            <span className="text-sm text-gray-600 dark:text-gray-400">Status:</span>
            <span className={`px-3 py-2 rounded-full text-sm font-medium ${
              cardData.active_status === 'TRACK'
                ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400'
                : cardData.active_status === 'WATCH'
                ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400'
                : cardData.active_status === 'STOP'
                ? 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
            }`}>
              {cardData.active_status}
            </span>
          </div>
        </div>

        {/* Row 11: Next Check Info */}
        {cardData.next_quarter_estimate && (
          <div className="px-4 sm:px-6 py-3 border-t border-gray-100 dark:border-slate-700">
            <div className="text-center">
              <p className="text-xs text-gray-600 dark:text-gray-400">
                Next Check: Est. {cardData.next_quarter_estimate.announcement_date}
              </p>
              <p className="text-xs font-medium text-gray-700 dark:text-gray-300">
                {cardData.next_quarter_estimate.days_until_announcement} days
              </p>
            </div>
          </div>
        )}
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
          <h2 className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-2xl font-bold text-transparent">
            📋 Performance Monitor
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

      {/* Legend Section */}
      <Card className="border border-orange-200/50 bg-white/90 backdrop-blur-sm dark:border-orange-400/20 dark:bg-slate-800/90">
        <CardHeader className="pb-4">
          <CardTitle className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            📖 Status Legend
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            {/* Status Indicators */}
            <div className="space-y-3">
              <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                Status Indicators
              </h4>
              <div className="space-y-2">
                <div className="flex items-center gap-3">
                  <Badge className="bg-green-500 text-white">
                    <span className="mr-1">🟢</span>
                    TRACK
                  </Badge>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Strong performance, actively tracking
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <Badge className="bg-yellow-500 text-white">
                    <span className="mr-1">🟡</span>
                    WATCH
                  </Badge>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Monitor closely, mixed signals
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <Badge className="bg-red-500 text-white">
                    <span className="mr-1">🔴</span>
                    STOP
                  </Badge>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Weak performance, avoid investment
                  </span>
                </div>
              </div>
            </div>

            {/* Pattern Analysis */}
            <div className="space-y-3">
              <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                Pattern Analysis
              </h4>
              <div className="space-y-2">
                <div className="flex items-center gap-3">
                  <span className="font-semibold text-green-600">
                    ⬆️ POSITIVE
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Both EPS & price growing
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="font-semibold text-purple-600">
                    ↕️ MIXED
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Conflicting growth signals
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="font-semibold text-gray-600">
                    ⬇️ NEGATIVE
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Both metrics declining
                  </span>
                </div>
              </div>
            </div>

            {/* Performance Metrics */}
            <div className="space-y-3">
              <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                Key Metrics
              </h4>
              <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
                <div>
                  <strong>EPS Growth:</strong> Earnings per share change
                </div>
                <div>
                  <strong>Price Growth:</strong> Stock price change
                </div>
                <div>
                  <strong>Rank:</strong> Performance ranking position
                </div>
                <div>
                  <strong>Pattern:</strong> Recent 2-quarter trend analysis
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

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
