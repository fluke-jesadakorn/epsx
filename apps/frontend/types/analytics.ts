// Analytics dashboard types matching backend EPSRanking structures
export interface EPSRanking {
  symbol: string;
  name: string;
  country: string;
  sector: string;
  exchange: string;
  current_eps: number | null;
  qoq_growth: number | null;
  price_current: number | null;
  market_cap: number | null;
  volume: number | null;
  pe_ratio?: number | null;
  dividend_yield?: number | null;
  price_change?: number | null;
  price_change_pct?: number | null;
  relative_volume?: number | null;
  ranking_position: number | null;
  active_status: string;
  quarterly_data?: QuarterlyEPSData[];
}

export interface QuarterlyEPSData {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
  volume?: number;
}

export interface EPSRankingsResponse {
  data: EPSRanking[];
  pagination: EPSPagination;
}

export interface EPSPagination {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

// Filter types for analytics dashboard
export interface AnalyticsFilters {
  country?: string;
  sector?: string;
  sort_by: 'qoq_growth' | 'eps_growth' | 'current_eps' | 'eps' | 'market_cap' | 'volume' | 'price' | 'close' | 'pe_ratio' | 'dividend_yield' | 'change' | 'relative_volume' | 'name' | 'symbol' | 'ranking_position';
  min_eps?: number;
  max_eps?: number;
  min_growth?: number;
  max_growth?: number;
  min_market_cap?: number;
  max_market_cap?: number;
  min_volume?: number;
  max_volume?: number;
  min_price?: number;
  max_price?: number;
  min_pe_ratio?: number;
  max_pe_ratio?: number;
  min_dividend_yield?: number;
  max_dividend_yield?: number;
  exchange?: string;
  stock_type?: 'common' | 'preferred' | 'reit' | 'etf' | 'fund' | 'dr' | 'warrant' | 'unit' | 'all';
  page: number;
  limit: number;
}

// Filter options from backend
export interface FilterOptions {
  countries: string[];
  sectors: string[];
  exchanges?: string[];
  stock_types?: string[];
}

// Component props types
export interface StockCardProps {
  ranking: EPSRanking;
  rank: number;
}

export interface FilterPanelProps {
  filters: AnalyticsFilters;
  options: FilterOptions;
  onFiltersChange: (filters: Partial<AnalyticsFilters>) => void;
  isLoading?: boolean;
}

export interface PaginationProps {
  pagination: EPSPagination;
  onPageChange: (page: number) => void;
  isLoading?: boolean;
}

// Growth trend enum matching backend
export enum EPSGrowthTrend {
  Accelerating = 'Accelerating',
  Steady = 'Steady', 
  Decelerating = 'Decelerating',
  Volatile = 'Volatile',
  Unknown = 'Unknown'
}

// Utility functions for data processing
export const getGrowthTrend = (growth: number | null): EPSGrowthTrend => {
  if (growth === null) return EPSGrowthTrend.Unknown;
  if (growth > 50) return EPSGrowthTrend.Accelerating;
  if (growth >= 10) return EPSGrowthTrend.Steady;
  if (growth >= 0) return EPSGrowthTrend.Steady;
  if (growth >= -10) return EPSGrowthTrend.Decelerating;
  return EPSGrowthTrend.Volatile;
};

export const formatMarketCap = (marketCap: number | null): string => {
  if (!marketCap) return 'N/A';
  if (marketCap >= 1e12) return `$${(marketCap / 1e12).toFixed(1)}T`;
  if (marketCap >= 1e9) return `$${(marketCap / 1e9).toFixed(1)}B`;
  if (marketCap >= 1e6) return `$${(marketCap / 1e6).toFixed(1)}M`;
  return `$${marketCap.toLocaleString()}`;
};

export const formatPercentage = (value: number | null): string => {
  if (value === null) return 'N/A';
  return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

// Card Dashboard Types - matching backend CardDashboardResponse
export interface CardDashboardResponse {
  success: boolean;
  data: SymbolCardData[];
  pagination: EPSPagination;
  metadata: CardDashboardMetadata;
  message?: string;
  processing_time_ms: number;
}

export interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number; // Current price
  active_status: string; // "Active" or "Non Active"
  quarterly_performance: QuarterlyPerformanceData[];
}

export interface QuarterlyPerformanceData {
  quarter: string; // "Q1", "Q0", etc.
  date: string; // "Aug 8, 2025"
  price: number;
  eps: number;
  eps_growth: number; // EPS % growth
  price_growth: number; // Price % growth
}

export interface CardDashboardMetadata {
  available_countries: string[];
  available_sectors: string[];
  request_timestamp: string;
  data_source: string;
}

// Updated StockCardProps for new card format
export interface CardStockProps {
  cardData: SymbolCardData;
}