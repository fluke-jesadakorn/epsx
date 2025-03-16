import axios, { AxiosError } from 'axios';

const RETRY_DELAY = 2000; // 2 seconds
const MAX_RETRIES = 3;

interface TradingViewApiResponse {
  s: 'ok' | 'error' | 'no_data';
  errmsg?: string;
  c?: number[]; // Close prices
  t?: number[]; // Timestamps
  v?: number[]; // Volumes
  h?: number[]; // High prices
  l?: number[]; // Low prices
  o?: number[]; // Open prices
}

interface TradingViewPriceResponse {
  symbol: string;
  price: number;
  date: string;
  status: 'success' | 'error';
  error?: string;
}

/**
 * Fetches stock price data from TradingView for a given symbol and date
 * @param symbol - Stock symbol (e.g., 'AAPL')
 * @param date - Target date in YYYY-MM-DD format
 */
async function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function getTradingViewPrice(
  symbol: string = 'AAPL',
  date: string = '2024-03-14',
): Promise<TradingViewPriceResponse> {
  try {
    // Convert symbol to TradingView format (e.g., 'NASDAQ:AAPL')
    const formattedSymbol = symbol.includes(':') ? symbol : `NASDAQ:${symbol}`;

    // TradingView requires specific headers to prevent blocking
    const headers = {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
      Accept: 'application/json',
      'Accept-Language': 'en-US,en;q=0.9',
      Origin: 'https://www.tradingview.com',
      Referer: 'https://www.tradingview.com',
    };

    // TradingView chart data endpoint
    const url = 'https://www.tradingview.com/chart/data/';

    // Convert date to Unix timestamp (midnight UTC)
    const targetDate = new Date(date);
    const timestamp = Math.floor(targetDate.getTime() / 1000);

    const data = {
      symbol: formattedSymbol,
      resolution: 'D', // Daily resolution
      from: timestamp,
      to: timestamp + 86400, // Add one day to get full day data
      countback: 1,
    };

    let retries = 0;
    while (true) {
      try {
        const response = await axios.post(url, data, { headers });

        if (!response.data || response.data.s !== 'ok') {
          if (retries < MAX_RETRIES - 1) {
            retries++;
            await delay(RETRY_DELAY);
            continue;
          }
          return {
            symbol,
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
          symbol,
          price: closePrice,
          date,
          status: 'success',
        };
      } catch (err) {
        if (err instanceof AxiosError && err.response?.status === 429) {
          // Rate limit hit
          if (retries < MAX_RETRIES - 1) {
            retries++;
            await delay(RETRY_DELAY * (retries + 1)); // Exponential backoff
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
        : error instanceof Error
          ? error.message
          : 'Unknown error occurred';
    return {
      symbol,
      price: 0,
      date,
      status: 'error',
      error: errorMessage,
    };
  }
}

// Example usage:
const priceData = await getTradingViewPrice('AAPL', '2024-03-14');
console.log(priceData.date);
