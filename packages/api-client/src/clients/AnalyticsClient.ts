import { BaseHttpClient } from '../base/BaseHttpClient';

import type {
  StockItem,
  StockRanking,
  StockFinancialData,
  PortfolioItem,
  WatchlistAddRequest,
  PriceAlert,
  PriceAlertCreateRequest,
  ApiResponse,
  PaginatedResponse,
} from '@epsx/types';

export class AnalyticsClient extends BaseHttpClient {
  // Stock rankings
  async getStockRankings(
    category?: string,
    limit?: number,
    page?: number
  ): Promise<ApiResponse<PaginatedResponse<StockRanking>>> {
    const params = new URLSearchParams();
    if (category) params.append('category', category);
    if (limit) params.append('limit', limit.toString());
    if (page) params.append('page', page.toString());
    
    return this.get<PaginatedResponse<StockRanking>>(`/api/analytics/rankings?${params}`);
  }

  async getStockDetails(symbol: string): Promise<ApiResponse<StockItem>> {
    return this.get<StockItem>(`/api/analytics/stocks/${symbol}`);
  }

  async getStockFinancials(symbol: string): Promise<ApiResponse<StockFinancialData>> {
    return this.get<StockFinancialData>(`/api/analytics/stocks/${symbol}/financials`);
  }

  async searchStocks(query: string, limit?: number): Promise<ApiResponse<StockItem[]>> {
    const params = new URLSearchParams({ query });
    if (limit) params.append('limit', limit.toString());
    
    return this.get<StockItem[]>(`/api/analytics/stocks/search?${params}`);
  }

  // Portfolio
  async getPortfolio(): Promise<ApiResponse<PortfolioItem[]>> {
    return this.get<PortfolioItem[]>('/api/analytics/portfolio');
  }

  async addToPortfolio(symbol: string, shares: number, avgPrice: number): Promise<ApiResponse<PortfolioItem>> {
    return this.post<PortfolioItem>('/api/analytics/portfolio', { symbol, shares, avgPrice });
  }

  async updatePortfolioItem(id: string, shares: number, avgPrice: number): Promise<ApiResponse<PortfolioItem>> {
    return this.put<PortfolioItem>(`/api/analytics/portfolio/${id}`, { shares, avgPrice });
  }

  async removeFromPortfolio(id: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/analytics/portfolio/${id}`);
  }

  // Watchlist
  async getWatchlist(): Promise<ApiResponse<StockItem[]>> {
    return this.get<StockItem[]>('/api/analytics/watchlist');
  }

  async addToWatchlist(data: WatchlistAddRequest): Promise<ApiResponse<void>> {
    return this.post<void>('/api/analytics/watchlist', data);
  }

  async removeFromWatchlist(symbol: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/analytics/watchlist/${symbol}`);
  }

  // Price alerts
  async getPriceAlerts(): Promise<ApiResponse<PriceAlert[]>> {
    return this.get<PriceAlert[]>('/api/analytics/alerts');
  }

  async createPriceAlert(data: PriceAlertCreateRequest): Promise<ApiResponse<PriceAlert>> {
    return this.post<PriceAlert>('/api/analytics/alerts', data);
  }

  async updatePriceAlert(id: string, data: Partial<PriceAlertCreateRequest>): Promise<ApiResponse<PriceAlert>> {
    return this.put<PriceAlert>(`/api/analytics/alerts/${id}`, data);
  }

  async deletePriceAlert(id: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/analytics/alerts/${id}`);
  }

  async togglePriceAlert(id: string, isActive: boolean): Promise<ApiResponse<PriceAlert>> {
    return this.patch<PriceAlert>(`/api/analytics/alerts/${id}`, { isActive });
  }

  // Market data
  async getMarketOverview(): Promise<ApiResponse<unknown>> {
    return this.get<unknown>('/api/analytics/market/overview');
  }

  async getMarketSectors(): Promise<ApiResponse<unknown[]>> {
    return this.get<unknown[]>('/api/analytics/market/sectors');
  }

  async getTrendingStocks(limit?: number): Promise<ApiResponse<StockItem[]>> {
    const params = limit ? `?limit=${limit}` : '';
    return this.get<StockItem[]>(`/api/analytics/market/trending${params}`);
  }

  // Backend analytics endpoints
  async getSystemMetrics(params?: {
    startDate?: string;
    endDate?: string;
    granularity?: string;
  }): Promise<ApiResponse<unknown>> {
    const queryParams = new URLSearchParams();
    if (params?.startDate) queryParams.append('start_date', params.startDate);
    if (params?.endDate) queryParams.append('end_date', params.endDate);
    if (params?.granularity) queryParams.append('granularity', params.granularity);
    
    return this.get<unknown>(`/api/v1/analytics/system/metrics?${queryParams}`);
  }

  async getAnalyticsData(params?: {
    startDate?: string;
    endDate?: string;
    granularity?: string;
  }): Promise<ApiResponse<unknown>> {
    const queryParams = new URLSearchParams();
    if (params?.startDate) queryParams.append('start_date', params.startDate);
    if (params?.endDate) queryParams.append('end_date', params.endDate);
    if (params?.granularity) queryParams.append('granularity', params.granularity);
    
    return this.get<unknown>(`/api/v1/analytics/data?${queryParams}`);
  }

  async getRealtimeMetrics(): Promise<ApiResponse<unknown>> {
    return this.get<unknown>('/api/v1/analytics/realtime');
  }

  async getRevenueAnalytics(params?: {
    startDate?: string;
    endDate?: string;
    granularity?: string;
  }): Promise<ApiResponse<unknown>> {
    const queryParams = new URLSearchParams();
    if (params?.startDate) queryParams.append('start_date', params.startDate);
    if (params?.endDate) queryParams.append('end_date', params.endDate);
    if (params?.granularity) queryParams.append('granularity', params.granularity);
    
    return this.get<unknown>(`/api/v1/analytics/revenue?${queryParams}`);
  }

  // EPS Analytics endpoints
  async getEPSRankings(params?: {
    page?: number;
    limit?: number;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: number;
    min_growth?: number;
  }): Promise<ApiResponse<{
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
  }>> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.country) queryParams.append('country', params.country);
    if (params?.sector) queryParams.append('sector', params.sector);
    if (params?.sort_by) queryParams.append('sort_by', params.sort_by);
    if (params?.min_eps) queryParams.append('min_eps', params.min_eps.toString());
    if (params?.min_growth) queryParams.append('min_growth', params.min_growth.toString());
    
    return this.get(`/api/v1/analytics/eps-rankings?${queryParams}`);
  }

  async getEPSCountries(): Promise<ApiResponse<{
    countries: string[];
    count: number;
  }>> {
    return this.get('/api/v1/analytics/eps-rankings/countries');
  }

  async getAllEPSCountries(): Promise<ApiResponse<{
    countries: string[];
    count: number;
  }>> {
    return this.get('/api/v1/analytics/eps-rankings/countries/all');
  }

  async getEPSSectorsByCountry(country?: string): Promise<ApiResponse<{
    sectors: string[];
    count: number;
    country?: string;
  }>> {
    const queryParams = country ? `?country=${encodeURIComponent(country)}` : '';
    return this.get(`/api/v1/analytics/eps-rankings/sectors${queryParams}`);
  }

  async getEPSHealth(): Promise<ApiResponse<{
    status: string;
    message: string;
    available_countries: number;
  }>> {
    return this.get('/api/v1/analytics/eps-rankings/health');
  }

  async syncEPSData(): Promise<ApiResponse<{
    success: boolean;
    message: string;
    stats: {
      total_fetched: number;
      total_processed: number;
      total_stored: number;
      total_errors: number;
      processing_duration_ms: number;
      countries_processed: string[];
    };
  }>> {
    return this.post('/api/v1/analytics/eps-rankings/sync', {});
  }

  // Live Analytics Rankings - Fresh data from backend (no cache)
  async getUnifiedAnalyticsRankings(params?: {
    page?: number;
    limit?: number;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: number;
    min_growth?: number;
  }): Promise<ApiResponse<CardDashboardResponse>> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.country) queryParams.append('country', params.country);
    if (params?.sector) queryParams.append('sector', params.sector);
    if (params?.sort_by) queryParams.append('sort_by', params.sort_by);
    if (params?.min_eps) queryParams.append('min_eps', params.min_eps.toString());
    if (params?.min_growth) queryParams.append('min_growth', params.min_growth.toString());
    
    // Add cache-busting timestamp to ensure fresh data
    queryParams.append('_t', Date.now().toString());
    
    return this.get(`/api/v1/analytics/rankings?${queryParams}`);
  }

  // Cache Management endpoints
  async getCacheStats(): Promise<ApiResponse<CacheStatsResponse>> {
    return this.get('/api/v1/analytics/cache/stats');
  }

  async refreshCache(): Promise<ApiResponse<CacheRefreshResponse>> {
    return this.post('/api/v1/analytics/cache/refresh', {});
  }

  async getCacheHealth(): Promise<ApiResponse<CacheHealthResponse>> {
    return this.get('/api/v1/analytics/cache/health');
  }
}

// Types for the unified analytics response
export interface UnifiedAnalyticsRankingsResponse {
  success: boolean;
  data: UnifiedRankingItem[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  metadata: UnifiedAnalyticsMetadata;
  message?: string;
  processing_time_ms: number;
}

export interface UnifiedRankingItem {
  symbol: string;
  company_name: string;
  ranking_position: number;
  current_price: number;
  current_price_date: string;
  quarterly_data: QuarterlyData[];
  market_data: MarketData;
  analytics: AnalyticsMetrics;
}

export interface QuarterlyData {
  quarter: string; // e.g., "Q3 '25"
  date: string;
  price: number;
  eps: number;
  eps_growth: number; // QoQ growth percentage
  price_growth: number; // QoQ price growth percentage
  volume?: number;
}

export interface MarketData {
  market_cap?: number;
  volume_24h?: number;
  country: string;
  sector: string;
  exchange: string;
}

export interface AnalyticsMetrics {
  qoq_growth: number;
  ranking_score: number;
  trend: string; // bullish, bearish, neutral, etc.
  volatility: number;
}

export interface UnifiedAnalyticsMetadata {
  available_countries: string[];
  available_sectors: string[];
  current_filters: UnifiedFilters;
  request_timestamp: string;
  data_source: string;
  enhanced_with_websocket: boolean;
}

export interface UnifiedFilters {
  country?: string;
  sector?: string;
  sort_by: string;
  min_eps?: number;
  min_growth?: number;
}

// Cache Management Types
export interface CacheStatsResponse {
  success: boolean;
  stats: CacheStats;
  message: string;
  timestamp: string;
}

export interface CacheStats {
  total_entries: number;
  active_entries: number;
  expired_entries: number;
  hit_ratio: number;
  miss_ratio: number;
  cache_size_mb: number;
}

export interface CacheRefreshResponse {
  success: boolean;
  refreshed_entries: number;
  duration_ms: number;
  message: string;
  timestamp: string;
}

export interface CacheHealthResponse {
  status: string;
  healthy: boolean;
  cache_stats: CacheStats;
  recommendations: string[];
  timestamp: string;
}

// Card Dashboard Types
export interface CardDashboardResponse {
  success: boolean;
  data: SymbolCardData[];
  pagination: CardDashboardPagination;
  metadata: CardDashboardMetadata;
  message?: string;
  processing_time_ms: number;
}

export interface SymbolCardData {
  symbol: string;
  rank: number;
  latest_date: string;
  value: number; // Current EPS value
  index: number; // Performance index score
  avg_growth: number; // Average quarterly growth
  eps_to_price?: string | null; // EPS to price correlation data (optional)
  quarterly_performance: QuarterlyPerformanceData[];
}

export interface QuarterlyPerformanceData {
  quarter: string;
  date: string;
  eps: number;
  eps_growth: number;
  price_growth: number;
  price: number;
}

export interface CardDashboardPagination {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface CardDashboardMetadata {
  request_timestamp: string;
  data_source: string;
  available_countries: string[];
  enhanced_with_websocket: boolean;
}