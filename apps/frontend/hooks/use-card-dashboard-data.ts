import { analyticsClient } from '@/lib/api-client';
import { useEffect, useState } from 'react';

interface AdvancedFilters {
  country: string;
  sector: string;
  min_eps: number | undefined;
  min_growth: number | undefined;
  sort_by: string;
  page: number;
  limit: number;
}

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
    announcement_timestamp: number;
    days_until_announcement: number;
    estimated_eps: number;
    estimated_price_target?: number;
    confidence: string;
  };
  next_earnings_date?: number;
  last_earnings_date?: number;
  next_earnings_date_formatted?: string;
  days_until_next_earnings?: number;
  progress_percentage?: number;
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

interface FilterOptions {
  countries: Array<{ value: string; label: string }>;
  sectors: string[];
}

export function useCardDashboardData() {
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

  const [filterOptions, setFilterOptions] = useState<FilterOptions>({
    countries: [],
    sectors: [],
  });

  useEffect(() => {
    const loadFilterOptions = () => {
      try {
        setFilterOptions({
          countries: [
            { value: 'america', label: 'United States' },
            { value: 'canada', label: 'Canada' },
            { value: 'united_kingdom', label: 'United Kingdom' },
            { value: 'germany', label: 'Germany' },
            { value: 'france', label: 'France' },
            { value: 'japan', label: 'Japan' },
            { value: 'australia', label: 'Australia' },
          ],
          sectors: [
            'Technology',
            'Healthcare',
            'Financial Services',
            'Consumer Discretionary',
            'Industrials',
            'Energy',
            'Telecommunications',
            'Real Estate',
          ],
        });
      } catch (_error) {
        // Filter options loading failed
      }
    };

    void loadFilterOptions();
  }, []);

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

      const response = await analyticsClient.getRankings({
        page: queryParams.page,
        per_page: queryParams.limit,
        country: queryParams.country,
        sector: queryParams.sector,
        sort_by: queryParams.sort_by,
        sort_order: 'desc',
      });

      if (response?.pagination && response.metadata) {
        const transformedData = {
          success: true,
          data: response.rankings.map((ranking, index) => ({
            rank: index + 1,
            symbol: ranking.symbol,
            latest_date: new Date().toISOString().split('T')[0],
            value: ranking.marketCap ?? 0,
            active_status: 'Active',
            quarterly_performance: [],
            next_quarter_estimate: undefined,
          })),
          pagination: {
            page: response.pagination.page,
            limit: response.pagination.per_page,
            total: response.pagination.total_items,
            totalPages: response.pagination.total_pages,
            hasNext: response.pagination.page < response.pagination.total_pages,
            hasPrev: response.pagination.page > 1,
          },
          metadata: {
            available_countries: response.metadata.available_countries ?? [],
            available_sectors: response.metadata.available_sectors ?? [],
            request_timestamp: response.metadata.request_timestamp ?? new Date().toISOString(),
            data_source: response.metadata.data_source ?? 'analytics-api',
          },
          processing_time_ms: response.metadata.query_time ?? 0,
        };
        setData(transformedData);
      }
    } catch (_error) {
      setError('Failed to load dashboard data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadData();
  }, [filters]);

  const updateFilters = (newFilters: Partial<AdvancedFilters>) => {
    setFilters((prev) => ({
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
    if (!data) {
      return;
    }

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
      const csvData = data.data.map((item) => ({
        symbol: item.symbol,
        rank: item.rank,
        value: item.value,
        status: item.active_status,
        latest_date: item.latest_date,
        quarters: item.quarterly_performance.length,
      }));

      const firstRow = csvData[0];
      const headers = firstRow ? Object.keys(firstRow).join(',') : '';
      const rows = csvData.map((row) => Object.values(row).join(','));
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

  return {
    data,
    setData,
    loading,
    error,
    showFilters,
    setShowFilters,
    filters,
    filterOptions,
    updateFilters,
    resetFilters,
    handleExport,
    loadData,
  };
}
