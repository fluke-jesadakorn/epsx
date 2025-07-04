// TypeScript type for TradingView stock screener API response

export interface TradingViewStockScreenerItem {
  s: string; // Symbol, e.g., "NASDAQ:MSFT"
  d: any[];  // Data array, order depends on columns requested
}

export interface TradingViewStockScreenerResponse {
  totalCount: number;
  data: TradingViewStockScreenerItem[];
}
