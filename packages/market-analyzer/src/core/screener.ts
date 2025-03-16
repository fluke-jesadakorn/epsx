import axios, { AxiosError, RawAxiosRequestHeaders } from 'axios';
import { 
  ScreenerRequestHeaders,
  ScreenerRequestBody,
  ScreenerResponse,
  ScreenerOptions,
  ScreenerFilter
} from '../types';
import { 
  TRADINGVIEW_ENDPOINTS, 
  DEFAULT_HEADERS,
  SCREENER_DEFAULTS 
} from '../utils/constants';
import { formatErrorMessage } from '../utils/helpers';

/**
 * Get stock data from TradingView screener
 * @param options Screening options including filters, columns, range, sort, and markets
 */
export async function screenStocks(options: ScreenerOptions = {}): Promise<ScreenerResponse> {
  try {
    const headers: RawAxiosRequestHeaders = {
      ...DEFAULT_HEADERS,
      accept: 'application/json',
      'accept-language': 'en-US,en;q=0.9',
      'content-type': 'application/json',
      priority: 'u=1, i',
      'sec-ch-ua': '"Chromium";v="120", "Not:A-Brand";v="24", "Google Chrome";v="120"',
      'sec-ch-ua-mobile': '?0',
      'sec-ch-ua-platform': '"macOS"',
      'sec-fetch-dest': 'empty',
      'sec-fetch-mode': 'cors',
      'sec-fetch-site': 'same-site',
      'Referrer-Policy': 'origin-when-cross-origin'
    };

    // Default filters for stocks
    const defaultFilters: ScreenerFilter[] = [
      { left: 'is_primary', operation: 'equal', right: true },
      { left: 'type', operation: 'equal', right: 'stock' }
    ];

    const body: ScreenerRequestBody = {
      columns: options.columns || [
        'name',
        'description',
        'close',
        'change',
        'volume',
        'market_cap_basic',
        'price_earnings_ttm',
        'sector',
        'market',
        'exchange'
      ],
      filter: [...defaultFilters, ...(options.filters || [])],
      ignore_unknown_fields: false,
      options: {
        lang: SCREENER_DEFAULTS.LANGUAGE
      },
      price_conversion: {
        to_currency: SCREENER_DEFAULTS.CURRENCY
      },
      range: options.range || SCREENER_DEFAULTS.RANGE,
      sort: options.sort
        ? { sortBy: options.sort.by, sortOrder: options.sort.order }
        : { sortBy: SCREENER_DEFAULTS.SORT.BY, sortOrder: SCREENER_DEFAULTS.SORT.ORDER },
      symbols: {},
      markets: options.markets || []
    };

    const response = await axios.post(
      TRADINGVIEW_ENDPOINTS.SCREENER,
      body,
      { headers }
    );

    if (!response.data || !Array.isArray(response.data.data)) {
      throw new Error('Invalid response from TradingView screener');
    }

    return {
      totalCount: response.data.totalCount || response.data.data.length,
      data: response.data.data.map((item: Record<string, any>) => ({
        name: item.name || '',
        description: item.description || '',
        close: item.close || 0,
        volume: item.volume || 0,
        market_cap_basic: item.market_cap_basic || 0,
        price_earnings_ttm: item.price_earnings_ttm || 0,
        sector: item.sector || '',
        market: item.market || '',
        exchange: item.exchange || '',
        ...item
      }))
    };
  } catch (error) {
    console.error('TradingView Screener error:', error);
    const errorMessage = error instanceof AxiosError
      ? `HTTP ${error.response?.status}: ${error.response?.statusText || error.message}`
      : formatErrorMessage(error);
    
    throw new Error(`Screener error: ${errorMessage}`);
  }
}

/**
 * Get stocks with specific EPS growth criteria
 */
export async function getGrowthStocks(): Promise<ScreenerResponse> {
  const filters: ScreenerFilter[] = [
    { left: 'is_primary', operation: 'equal', right: true },
    { left: 'last_annual_eps', operation: 'greater', right: 0 },
    { left: 'earnings_per_share_forecast_next_fq', operation: 'greater', right: 0 },
    { left: 'earnings_per_share_diluted_qoq_growth_fq', operation: 'greater', right: 0 }
  ];

  const columns = [
    'logoid',
    'name',
    'close',
    'change',
    'change_abs',
    'Recommend.All',
    'volume',
    'Value.Traded',
    'market_cap_basic',
    'price_earnings_ttm',
    'earnings_per_share_basic_ttm',
    'number_of_employees',
    'sector',
    'description'
  ];

  return screenStocks({
    filters,
    columns,
    sort: {
      by: 'volume',
      order: 'desc'
    }
  });
}
