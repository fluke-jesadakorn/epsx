export interface TradingViewMessage {
  m: string;
  p: any[];
}

export interface SessionMessage {
  session_id: string;
  timestamp: number;
  timestampMs: number;
}

export type PriceUpdateCallback = (data: {
  symbol: string;
  price: number;
  volume: number;
  timestamp: number;
}) => void;

export interface SubscriptionOptions {
  symbol: string;
  interval?: string;
  onUpdate?: PriceUpdateCallback;
}

export interface SubscriptionOptionsInternal extends SubscriptionOptions {
  symbolId: string;
  seriesId: string;
  exchange?: string;
}

export interface QuoteData {
  lp?: number;      // Last price
  volume?: number;  // Volume
  ch?: number;      // Change
  chp?: number;     // Change percent
  bid?: number;     // Bid price
  ask?: number;     // Ask price
  [key: string]: number | undefined;  // Allow other number fields
}

export interface QuoteMessage {
  [key: string]: QuoteData;
}
