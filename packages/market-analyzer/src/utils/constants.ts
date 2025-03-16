// API endpoints
export const TRADINGVIEW_ENDPOINTS = {
  CHART_DATA: 'https://www.tradingview.com/chart/data/',
  SCREENER: 'https://scanner.tradingview.com/global/scan',
  WEBSOCKET: 'wss://data.tradingview.com/socket.io/websocket'
} as const;

// Default request headers
export const DEFAULT_HEADERS = {
  'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
  Accept: 'application/json',
  'Accept-Language': 'en-US,en;q=0.9',
  Origin: 'https://www.tradingview.com',
  Referer: 'https://www.tradingview.com'
} as const;

// WebSocket configurations
export const WS_CONFIG = {
  RETRY_DELAY: 2000,         // 2 seconds initial delay
  MAX_RETRIES: 3,            // Maximum number of retry attempts
  READY_TIMEOUT: 30000,      // 30 seconds connection timeout
  SEND_DELAY: 100,          // 100ms delay between messages
  DEFAULT_INTERVAL: '1D'     // Default chart interval
} as const;

// Screener default settings
export const SCREENER_DEFAULTS = {
  LANGUAGE: 'en',
  CURRENCY: 'usd',
  RANGE: [0, 150] as [number, number],
  SORT: {
    BY: 'volume',
    ORDER: 'desc' as const
  }
} as const;

// Common error messages
export const ERROR_MESSAGES = {
  WS_NOT_READY: 'WebSocket not ready. Call waitForReady() first.',
  WS_CONNECTION_TIMEOUT: 'WebSocket connection timeout - failed to initialize within 30 seconds',
  MAX_RETRIES_REACHED: 'Max reconnection attempts reached',
  API_ERROR: 'TradingView API error:'
} as const;
