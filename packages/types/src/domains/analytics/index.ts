export interface StockItem {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  volume: number;
  marketCap: number;
  eps?: number;
  peRatio?: number;
  sector?: string;
  industry?: string;
  lastUpdated: Date;
}

export interface StockRanking {
  rank: number;
  symbol: string;
  name: string;
  score: number;
  metrics: Record<string, number>;
  change: number;
  changePercent: number;
  category: string;
}

export interface StockFinancialData {
  symbol: string;
  revenue: number;
  earnings: number;
  eps: number;
  pe: number;
  pb: number;
  roe: number;
  debtToEquity: number;
  currentRatio: number;
  quickRatio: number;
  profitMargin: number;
  operatingMargin: number;
  asOfDate: Date;
}

export interface PortfolioItem {
  id: string;
  userId: string;
  symbol: string;
  shares: number;
  avgPrice: number;
  currentPrice: number;
  totalValue: number;
  gainLoss: number;
  gainLossPercent: number;
  addedAt: Date;
  updatedAt: Date;
}

export interface WatchlistAddRequest {
  symbol: string;
  notes?: string;
}

export interface PriceAlert {
  id: string;
  userId: string;
  symbol: string;
  targetPrice: number;
  condition: 'above' | 'below';
  isActive: boolean;
  isTriggered: boolean;
  createdAt: Date;
  triggeredAt?: Date;
}

export interface PriceAlertCreateRequest {
  symbol: string;
  targetPrice: number;
  condition: 'above' | 'below';
}

export enum StockRankingType {
  EPS_GROWTH = 'eps_growth',
  MARKET_CAP = 'market_cap',
  VOLUME = 'volume',
  PRICE_CHANGE = 'price_change',
  TECHNICAL_INDICATORS = 'technical_indicators',
  AI_INSIGHTS = 'ai_insights',
  PATTERN_RECOGNITION = 'pattern_recognition',
  CUSTOM_METRICS = 'custom_metrics'
}