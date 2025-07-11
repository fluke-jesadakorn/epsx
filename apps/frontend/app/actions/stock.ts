'use server';

import { rankStocksByEpsWithChart } from '@/utils/processStocks/rankingStocks';
import { transformFinancialData } from '@/utils/transformers/stockDataTransformer';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '../../../../types/marketCountries';

// Server-side cache to store data temporarily
let serverCache: {
  data: StockFinancialData[];
  timestamp: number;
  ttl: number;
} | null = null;

const CACHE_TTL = 300; // 5 minutes in seconds

export async function fetchStockFinancialData(
  skip = 0,
  limit = 10,
  country: typeof MarketCountry = MarketCountry,
  quarters = 4,
): Promise<StockFinancialData[]> {
  try {
    console.log('Fetching stock financial data with params:', {
      skip,
      limit,
      country,
      quarters,
    });

    // Check server-side cache first
    const now = Date.now();
    if (serverCache && now - serverCache.timestamp < serverCache.ttl * 1000) {
      console.log('Returning cached financial data');
      return serverCache.data;
    }

    // Fetch data using the new utility function
    const chartData = await rankStocksByEpsWithChart(
      skip,
      limit,
      country,
      quarters,
    );

    if (!chartData || Object.keys(chartData).length === 0) {
      console.log('No financial chart data retrieved.');
      return [];
    }

    // Transform the data to the new format
    const transformedData = transformFinancialData(chartData);

    // Cache the result
    serverCache = {
      data: transformedData,
      timestamp: now,
      ttl: CACHE_TTL,
    };

    console.log(transformedData);
    return transformedData;
  } catch (error) {
    console.error('Error in fetchStockFinancialData:', error);
    // Return cached data if available, even if expired, as fallback
    if (serverCache) {
      console.log('Returning expired cached data as fallback');
      return serverCache.data;
    }
    return [];
  }
}

// Keep legacy function for backward compatibility during transition
export async function fetchStockScreenerData(): Promise<StockFinancialData[]> {
  return fetchStockFinancialData();
}
