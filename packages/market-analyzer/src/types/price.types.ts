export interface TradingViewApiResponse {
  s: 'ok' | 'error' | 'no_data';
  errmsg?: string;
  c?: number[]; // Close prices
  t?: number[]; // Timestamps
  v?: number[]; // Volumes
  h?: number[]; // High prices
  l?: number[]; // Low prices
  o?: number[]; // Open prices
}

export interface TradingViewPriceResponse {
  symbol: string;
  price: number;
  date: string;
  status: 'success' | 'error';
  error?: string;
}

export interface PriceRequestHeaders {
  'User-Agent': string;
  Accept: string;
  'Accept-Language': string;
  Origin: string;
  Referer: string;
  [key: string]: string;
}

export interface PriceRequestData {
  symbol: string;
  resolution: string;
  from: number;
  to: number;
  countback: number;
}

export interface CurrentPriceResponse {
  symbol: string;
  price: number;
  volume: number;
  timestamp: number;
  status: 'success' | 'error';
  error?: string;
}
