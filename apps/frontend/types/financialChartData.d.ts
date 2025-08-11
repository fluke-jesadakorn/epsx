export interface QuarterData {
  price: number | null;
  date: string;
  eps: number;
  quarter: number | string;
  eps_growth?: number;
  price_growth?: number;
  last_eps_vs_current_price?: {
    lastEpsGrowth: number | null;
    currentPriceGrowth: number | null;
  };
}

export interface StockFinancialData {
  symbol: string;
  quarters: QuarterData[];
  currentPrice?: number | null;
  currentPriceDate?: string | null;
}

export interface CacheData<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

// Unified Analytics types - matching the backend API
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
  quarterly_data: UnifiedQuarterlyData[];
  market_data: UnifiedMarketData;
  analytics: UnifiedAnalyticsMetrics;
}

export interface UnifiedQuarterlyData {
  quarter: string; // e.g., "Q3 '25"
  date: string;
  price: number;
  eps: number;
  eps_growth: number; // QoQ growth percentage
  price_growth: number; // QoQ price growth percentage
  volume?: number;
}

export interface UnifiedMarketData {
  market_cap?: number;
  volume_24h?: number;
  country: string;
  sector: string;
  exchange: string;
}

export interface UnifiedAnalyticsMetrics {
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

// Cache Management Types (matching the API client)
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

// Card Dashboard Types - matching backend API
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
  eps_growth: number; // Quarter-over-quarter EPS growth
  price_growth: number; // Quarter-over-quarter price growth 
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
