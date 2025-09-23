import { createApiClient, isApiError } from '@/lib/api-client';
import { MarketCountry } from '../../types/market';
import { getBackendUrl } from '../../../../shared/utils/url-resolver';

const BACKEND_URL = getBackendUrl('server');
const apiClient = createApiClient(BACKEND_URL);

export async function getStockFinancialData(
  page: number = 1,
  limit: number = 10,
  country?: typeof MarketCountry,
  quarters: number = 2
) {
  try {
    const response = await apiClient.getStocks({
      page,
      limit,
      country: country?.toString(),
      quarters,
    });

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch stock financial data');
    }

    return response.data;
  } catch (error) {
    console.error('Stock data fetch error:', error);
    throw error;
  }
}

export async function getStockFinancialDataPaginated(
  page: number = 1,
  limit: number = 10,
  country?: typeof MarketCountry,
  quarters: number = 2
) {
  try {
    const response = await apiClient.getPaginatedStocks({
      page,
      limit,
      country: country?.toString(),
      quarters,
    });

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch paginated stock data');
    }

    return response.data;
  } catch (error) {
    console.error('Paginated stock data fetch error:', error);
    throw error;
  }
}


export async function getIndividualStockData(symbol: string) {
  try {
    const response = await apiClient.serverGetIndividualStock(symbol);

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch individual stock data');
    }

    return response.data;
  } catch (error) {
    console.error('Individual stock data fetch error:', error);
    throw error;
  }
}

export async function getBatchStockData(symbols: string[]) {
  try {
    const response = await apiClient.serverBatchStocks(symbols);

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch batch stock data');
    }

    return response.data;
  } catch (error) {
    console.error('Batch stock data fetch error:', error);
    throw error;
  }
}

export async function getStocksCount(country?: string, quarters?: number) {
  try {
    const response = await apiClient.getStockCount({
      country,
      quarters,
    });

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch stocks count');
    }

    return response.data;
  } catch (error) {
    console.error('Stocks count fetch error:', error);
    throw error;
  }
}

export async function getPremiumRankings() {
  try {
    const response = await apiClient.serverGetPremiumRankings();

    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to fetch premium rankings');
    }

    return response.data;
  } catch (error) {
    console.error('Premium rankings fetch error:', error);
    throw error;
  }
}

