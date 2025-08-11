'use server';

import { getStockFinancialData, getStockFinancialDataCount } from '@/lib/services/stock.service';
import { stockApiClient } from '@/lib/api/stockApiClient.client';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '../../../../types/marketCountries';

export interface PaginatedStockData {
  data: StockFinancialData[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

/**
 * Fetch paginated stock data using direct service calls (server-side)
 */
export async function fetchPaginatedStockData(
  page: number = 1,
  limit: number = 10,
  country: typeof MarketCountry = MarketCountry,
  quarters: number = 2
): Promise<PaginatedStockData> {
  try {
    const [data, totalCount] = await Promise.all([
      getStockFinancialData(page, limit, country, quarters),
      getStockFinancialDataCount(country, quarters)
    ]);

    return {
      data,
      pagination: {
        page,
        limit,
        total: totalCount,
        totalPages: Math.ceil(totalCount / limit),
        hasNext: page < Math.ceil(totalCount / limit),
        hasPrev: page > 1,
      }
    };
  } catch (error) {
    console.error('Error fetching paginated stock data:', error);
    return {
      data: [],
      pagination: {
        page: 1,
        limit,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false,
      }
    };
  }
}

/**
 * Fetch paginated stock data using API client (client-side)
 */
export async function fetchPaginatedStockDataFromAPI(
  page: number = 1,
  limit: number = 10,
  country?: string,
  quarters: number = 2
): Promise<PaginatedStockData> {
  try {
    const response = await stockApiClient.getPaginatedStocks({
      page,
      limit,
      country,
      quarters
    });

    return {
      data: response.data,
      pagination: {
        page: response.pagination.page,
        limit: response.pagination.limit,
        total: response.pagination.total,
        totalPages: response.pagination.totalPages,
        hasNext: response.pagination.hasNext,
        hasPrev: response.pagination.hasPrev,
      }
    };
  } catch (error) {
    console.error('Error fetching paginated stock data from API:', error);
    return {
      data: [],
      pagination: {
        page: 1,
        limit,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false,
      }
    };
  }
}

/**
 * Fetch stock count using API client
 */
export async function fetchStockCount(
  country?: string,
  quarters: number = 2
): Promise<number> {
  try {
    const response = await stockApiClient.getStockCount({
      country,
      quarters
    });
    return response.count;
  } catch (error) {
    console.error('Error fetching stock count:', error);
    return 0;
  }
}
