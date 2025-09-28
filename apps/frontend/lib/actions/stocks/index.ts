'use server'

import type { StockFinancialData } from '@/types/financialChartData';

// Stock data server actions - stub implementations for build compatibility

interface BatchStocksResponse {
  success: boolean;
  data: { [symbol: string]: StockFinancialData | null };
  cached?: string[];
  fetched?: string[];
  errors?: string[];
}

interface PreloadStocksResponse {
  success: boolean;
  preloaded: string[];
}

interface CacheStatusResponse {
  success: boolean;
  symbols: string[];
}

/**
 * Batch fetch stock data - stub implementation
 * Returns empty data for build compatibility
 */
export async function getBatchStocks(symbols: string[]): Promise<BatchStocksResponse> {
  console.warn('getBatchStocks is not implemented - stock data disabled');
  return {
    success: true,
    data: Object.fromEntries(symbols.map(symbol => [symbol, null])),
    cached: [],
    fetched: [],
    errors: []
  };
}

/**
 * Get single stock data - stub implementation
 * Returns null for build compatibility
 */
export async function getStockData(symbol: string): Promise<StockFinancialData | null> {
  console.warn('getStockData is not implemented - stock data disabled');
  return null;
}

/**
 * Preload stocks into cache - stub implementation
 * Returns success for build compatibility
 */
export async function preloadStocks(symbols: string[]): Promise<PreloadStocksResponse> {
  console.warn('preloadStocks is not implemented - stock preloading disabled');
  return {
    success: true,
    preloaded: []
  };
}

/**
 * Check stock cache status - stub implementation
 * Returns empty array for build compatibility
 */
export async function checkStockCacheStatus(symbols: string[]): Promise<CacheStatusResponse> {
  console.warn('checkStockCacheStatus is not implemented - stock cache disabled');
  return {
    success: true,
    symbols: []
  };
}