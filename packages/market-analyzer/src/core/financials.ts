import axios, { AxiosError } from 'axios';
import { 
  FinancialData,
  FinancialRequestData
} from '../types';
import { 
  TRADINGVIEW_ENDPOINTS, 
  DEFAULT_HEADERS,
  WS_CONFIG 
} from '../utils/constants';
import { 
  delay, 
  formatSymbol, 
  formatErrorMessage
} from '../utils/helpers';

/**
 * Fetches financial data from TradingView for a given symbol
 * @param symbol Stock symbol in EXCHANGE:SYMBOL format (e.g., 'NASDAQ:AAPL')
 */
export async function getFinancials(symbol: string): Promise<FinancialData> {
  try {
    const formattedSymbol = formatSymbol(symbol);
    const data: FinancialRequestData = {
      symbol: formattedSymbol,
      adjustment: 'splits',
      session: 'extended',
      type: 'stock'
    };

    let retries = 0;
    while (true) {
      try {
        const response = await axios.post(
          TRADINGVIEW_ENDPOINTS.CHART_DATA,
          data,
          { headers: DEFAULT_HEADERS }
        );

        if (!response.data || response.data.s === 'error') {
          if (retries < WS_CONFIG.MAX_RETRIES - 1) {
            retries++;
            await delay(WS_CONFIG.RETRY_DELAY);
            continue;
          }
          return {
            symbol: formattedSymbol,
            data: {},
            status: 'error',
            error: response.data?.error || 'Failed to fetch financial data'
          };
        }

        return {
          symbol: formattedSymbol,
          data: response.data,
          status: 'success'
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
    const errorMessage = error instanceof AxiosError
      ? `HTTP ${error.response?.status}: ${error.response?.statusText || error.message}`
      : formatErrorMessage(error);
      
    return {
      symbol,
      data: {},
      status: 'error',
      error: errorMessage
    };
  }
}
