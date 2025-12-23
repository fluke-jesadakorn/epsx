// Consolidated Analytics Domain Types

export interface EPSRanking {
    symbol: string;
    name: string;
    country: string;
    sector: string;
    exchange: string;
    current_eps: number | null;
    growth_factor: number | null;
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
    currency?: string;
    quarterly_data?: QuarterlyEPSData[];
}

export type AnalyticsRankingItem = EPSRanking;

export interface QuarterlyEPSData {
    quarter: string;
    date: string;
    price: number;
    eps: number;
    eps_growth: number;
    price_growth: number;
    volume?: number;
    year?: number;
    revenue?: number | null;
    revenue_growth?: number | null;
}

export interface AnalyticsFilters {
    country?: string;
    sector?: string;
    sort_by: string;
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
    stock_type?: string;
    page: number;
    limit: number;
}

export interface AnalyticsQueryParams {
    page?: number;
    limit?: number;
    offset?: number;
    sort_by?: string;
    sort_order?: 'asc' | 'desc';
    country?: string;
    sector?: string;
    min_eps?: number;
    max_eps?: number;
    min_growth?: number;
    max_growth?: number;
    min_market_cap?: number;
    max_market_cap?: number;
}

export interface FilterOptions {
    countries: CountryOption[];
    sectors: string[];
    exchanges?: string[];
    stock_types?: string[];
}

export interface CountryOption {
    value: string;
    label: string;
    flag?: string;
    market_count?: number;
}

export interface AnalyticsPagination {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
}

export interface AnalyticsRankingsResponse {
    data: EPSRanking[];
    pagination: AnalyticsPagination;
}

export interface SymbolCardData {
    rank: number;
    symbol: string;
    latest_date: string;
    value: number;
    active_status: string;
    currency?: string;
    quarterly_performance: QuarterlyPerformanceData[];
    next_quarter_estimate?: NextQuarterEstimate;
    next_earnings_date?: number;
    last_earnings_date?: number;
    next_earnings_date_formatted?: string;
    days_until_next_earnings?: number;
    progress_percentage?: number;
}

export interface QuarterlyPerformanceData {
    quarter: string;
    date: string;
    price: number;
    eps: number;
    eps_growth: number;
    price_growth: number;
    announcement_date?: string;
    announcement_timestamp?: number;
    is_estimated?: boolean;
}

export interface NextQuarterEstimate {
    quarter: string;
    estimated_eps: number;
    announcement_date: string;
    announcement_timestamp: number;
    days_until_announcement: number;
    estimated_price_target?: number;
    confidence: string;
}

export enum EPSGrowthTrend {
    Accelerating = 'Accelerating',
    Steady = 'Steady',
    Decelerating = 'Decelerating',
    Volatile = 'Volatile',
    Unknown = 'Unknown'
}

export interface PortfolioPosition {
    symbol: string;
    rank: number;
    actionStatus: string;
    actionPhase: {
        start: string;
        end: string;
    };
    daysRemaining: number;
    performance: number;
    quarters: QuarterlyPerformanceData[];
    nextAnnouncement: string;
    gradientClass: string;
    currency?: string;
}

// Component Props Types
export interface FilterPanelProps {
    filters: AnalyticsFilters;
    options: FilterOptions;
    onFiltersChange: (filters: Partial<AnalyticsFilters>) => void;
    isLoading?: boolean;
}

export interface PaginationProps {
    pagination: AnalyticsPagination;
    onPageChange: (page: number) => void;
    onLimitChange?: (limit: number) => void;
    isLoading?: boolean;
}

export interface StockCardProps {
    ranking: EPSRanking;
    rank: number;
}

export interface CardStockProps {
    cardData: SymbolCardData;
}

export interface PositionCardProps {
    position: PortfolioPosition;
    onActionChange: (symbol: string, action: string) => void;
}
