export interface QuarterData {
  price: number | null;
  date: string;
  eps: number;
  quarter: number;
  eps_growth?: number;
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
