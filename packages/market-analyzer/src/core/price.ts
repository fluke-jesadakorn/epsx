import axios, { AxiosError } from 'axios';
import {
  TradingViewApiResponse,
  TradingViewPriceResponse,
  PriceRequestHeaders,
  PriceRequestData,
  CurrentPriceResponse,
} from '../types';
import {
  TRADINGVIEW_ENDPOINTS,
  DEFAULT_HEADERS,
  WS_CONFIG,
} from '../utils/constants';
import {
  delay,
  formatSymbol,
  dateToTimestamp,
  formatErrorMessage,
} from '../utils/helpers';

/**
 * Fetches current price and volume data from TradingView for a given symbol
 * @param symbol Stock symbol in EXCHANGE:SYMBOL format (e.g., 'NASDAQ:AAPL')
 */
export async function getPrice(symbol: string): Promise<CurrentPriceResponse> {
  try {
    const formattedSymbol = formatSymbol(symbol);
    const now = Math.floor(Date.now() / 1000);

    const data: PriceRequestData = {
      symbol: formattedSymbol,
      resolution: '1', // 1-minute resolution for current data
      from: now - 60, // Last minute
      to: now,
      countback: 1,
    };

    let retries = 0;
    while (true) {
      try {
        const response = await axios.post(
          TRADINGVIEW_ENDPOINTS.CHART_DATA,
          data,
          { headers: DEFAULT_HEADERS },
        );

        if (!response.data || response.data.s !== 'ok') {
          if (retries < WS_CONFIG.MAX_RETRIES - 1) {
            retries++;
            await delay(WS_CONFIG.RETRY_DELAY);
            continue;
          }
          return {
            symbol: formattedSymbol,
            price: 0,
            volume: 0,
            timestamp: now,
            status: 'error',
            error: response.data?.errmsg || 'Failed to fetch price data',
          };
        }

        const responseData: TradingViewApiResponse = response.data;
        const price =
          Array.isArray(responseData.c) && responseData.c.length > 0
            ? responseData.c[0]
            : 0;
        const volume =
          Array.isArray(responseData.v) && responseData.v.length > 0
            ? responseData.v[0]
            : 0;
        const timestamp =
          Array.isArray(responseData.t) && responseData.t.length > 0
            ? responseData.t[0]
            : now;

        return {
          symbol: formattedSymbol,
          price,
          volume,
          timestamp,
          status: 'success',
        };
      } catch (err) {
        if (err instanceof AxiosError && err.response?.status === 429) {
          // Rate limit hit
          if (retries < WS_CONFIG.MAX_RETRIES - 1) {
            retries++;
            await delay(WS_CONFIG.RETRY_DELAY * (retries + 1)); // Exponential backoff
            continue;
          }
        }
        throw err; // Re-throw other errors or if max retries reached
      }
    }
  } catch (error) {
    console.error('TradingView API error:', error);
    const errorMessage =
      error instanceof AxiosError
        ? `HTTP ${error.response?.status}: ${error.response?.statusText || error.message}`
        : formatErrorMessage(error);

    return {
      symbol,
      price: 0,
      volume: 0,
      timestamp: Math.floor(Date.now() / 1000),
      status: 'error',
      error: errorMessage,
    };
  }
}

export async function getTradingViewPrice(
  symbol: string,
  date: string,
): Promise<TradingViewPriceResponse> {
  try {
    const formattedSymbol = formatSymbol(symbol);
    const timestamp = dateToTimestamp(date);

    const data: PriceRequestData = {
      symbol: formattedSymbol,
      resolution: 'D', // Daily resolution
      from: timestamp,
      to: timestamp + 86400, // Add one day to get full day data
      countback: 1,
    };

    let retries = 0;
    while (true) {
      try {
        const response = await axios.post(
          TRADINGVIEW_ENDPOINTS.CHART_DATA,
          data,
          { headers: DEFAULT_HEADERS },
        );

        if (!response.data || response.data.s !== 'ok') {
          if (retries < WS_CONFIG.MAX_RETRIES - 1) {
            retries++;
            await delay(WS_CONFIG.RETRY_DELAY);
            continue;
          }
          return {
            symbol: formattedSymbol,
            price: 0,
            date,
            status: 'error',
            error: response.data?.errmsg || 'Failed to fetch price data',
          };
        }

        const prices: TradingViewApiResponse = response.data;
        const closePrice =
          Array.isArray(prices.c) && prices.c.length > 0 ? prices.c[0] : 0;

        return {
          symbol: formattedSymbol,
          price: closePrice,
          date,
          status: 'success',
        };
      } catch (err) {
        if (err instanceof AxiosError && err.response?.status === 429) {
          // Rate limit hit
          if (retries < WS_CONFIG.MAX_RETRIES - 1) {
            retries++;
            await delay(WS_CONFIG.RETRY_DELAY * (retries + 1)); // Exponential backoff
            continue;
          }
        }
        throw err; // Re-throw other errors or if max retries reached
      }
    }
  } catch (error) {
    console.error('TradingView API error:', error);
    const errorMessage =
      error instanceof AxiosError
        ? `HTTP ${error.response?.status}: ${error.response?.statusText || error.message}`
        : formatErrorMessage(error);

    return {
      symbol,
      price: 0,
      date,
      status: 'error',
      error: errorMessage,
    };
  }
}
