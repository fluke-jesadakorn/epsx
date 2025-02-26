export interface StockScreenerResponse<T = any> {
  data: {
    data: T[];
    resultsCount?: number;
  };
}

export interface IStockDocument {
  _id: unknown;
  symbol: string;
  company_name: string;
  exchange: unknown; // Reference to Exchange
  __v?: number;
}

export interface IStockBatchItem {
  s: string;  // symbol
  n: string;  // name
}
