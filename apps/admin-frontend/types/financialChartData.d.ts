export interface QuarterData {
  price: number | null;
  date: string;
  eps: number;
  quarter: number;
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
