// Shared stock data service

import { rankStocksByEpsWithChart } from '@/utils/processStocks/rankingStocks';
import { transformFinancialDataWithCurrentPrice } from '@/utils/transformers/stockDataTransformer';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '../../../../types/marketCountries';

// Server-side cache to store data temporarily
let serverCache: {
  data: StockFinancialData[];
  timestamp: number;
  ttl: number;
} | null = null;

const CACHE_TTL = 300; // 5 minutes in seconds

export async function getStockFinancialData(
  skip = 0,
  limit = 10,
  country: typeof MarketCountry = MarketCountry,
  quarters = 4,
): Promise<StockFinancialData[]> {
  try {
    // Check server-side cache first
    const now = Date.now();
    if (serverCache && now - serverCache.timestamp < serverCache.ttl * 1000) {
      return serverCache.data;
    }

    // Fetch data using the utility function
    const chartData = await rankStocksByEpsWithChart(
      skip,
      limit,
      country,
      quarters,
    );

    if (!chartData || Object.keys(chartData).length === 0) {
      return [];
    }

    // Transform the data to the new format with current prices
    const transformedData = transformFinancialDataWithCurrentPrice(chartData);

    // Cache the result
    serverCache = {
      data: transformedData,
      timestamp: now,
      ttl: CACHE_TTL,
    };

    return transformedData;
  } catch (error) {
    // Return cached data if available, even if expired, as fallback
    if (serverCache) {
      return serverCache.data;
    }
    return [];
  }
}
