// Shared stock data service

import { rankStocksByEpsWithChart } from '@/utils/processStocks/rankingStocks';
import { transformFinancialDataWithCurrentPrice } from '@/utils/processStocks/stockDataTransformer';
import type { StockFinancialData } from '@/types/financialChartData';
import { MarketCountry } from '../../../../types/marketCountries';
import { StockDataCache } from '@/utils/cache/stockDataCache';
import fetchScreenerStock from '@/utils/processStocks/fetchRankScreenedStock';

// Server-side cache to store data temporarily
let serverCache: {
  data: StockFinancialData[];
  timestamp: number;
  ttl: number;
} | null = null;

// Count cache for pagination
let countCache: {
  count: number;
  timestamp: number;
  ttl: number;
} | null = null;

const CACHE_TTL = 300; // 5 minutes in seconds

export async function getStockFinancialData(
  skip = 0,
  limit = 10,
  country: typeof MarketCountry = MarketCountry,
  quarters = 2,
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

    // Cache the result in server memory
    serverCache = {
      data: transformedData,
      timestamp: now,
      ttl: CACHE_TTL,
    };

    // Also cache individual symbols for per-card requests
    transformedData.forEach(stockData => {
      StockDataCache.set(stockData.symbol, stockData);
    });

    return transformedData;
  } catch (error) {
    // Return cached data if available, even if expired, as fallback
    if (serverCache) {
      return serverCache.data;
    }
    return [];
  }
}

/**
 * Get total count of stocks for pagination
 */
export async function getStockFinancialDataCount(
  country: typeof MarketCountry = MarketCountry,
  quarters = 2,
): Promise<number> {
  try {
    // Check count cache first
    const now = Date.now();
    if (countCache && now - countCache.timestamp < countCache.ttl * 1000) {
      return countCache.count;
    }

    // For simplicity, we'll fetch a large number to get the total count
    // In production, you might want to implement a dedicated count endpoint
    const stockData = await fetchScreenerStock(0, 1000, country);
    
    const count = stockData?.data?.length || 0;

    // Cache the count
    countCache = {
      count,
      timestamp: now,
      ttl: CACHE_TTL,
    };

    return count;
  } catch (error) {
    console.error('Error getting stock count:', error);
    // Return cached count if available, or fallback to 0
    return countCache?.count || 0;
  }
}
