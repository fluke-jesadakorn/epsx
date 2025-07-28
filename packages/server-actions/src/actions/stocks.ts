'use server';

import { serverGet, serverPost } from '../core/request';

// Stock Types
export interface StockFinancialData {
  symbol: string;
  price: number;
  change: number;
  changePercent: number;
  volume?: number;
  marketCap?: number;
  pe?: number;
  eps?: number;
  dividend?: number;
  dividendYield?: number;
  beta?: number;
  high52Week?: number;
  low52Week?: number;
  avgVolume?: number;
  lastUpdate?: string;
}

export interface BatchStockResponse {
  success: boolean;
  data: Record<string, StockFinancialData>;
  cached: string[];
  fetched: string[];
  errors: string[];
}

// Stock Actions
export async function getBatchStocks(symbols: string[]): Promise<BatchStockResponse> {
  try {
    const response = await serverPost('/api/v1/stocks/batch', { symbols });
    return response;
  } catch (error) {
    console.error('Error fetching batch stocks:', error);
    throw error;
  }
}

export async function getStockData(symbol: string): Promise<StockFinancialData | null> {
  try {
    const response = await serverGet(`/api/v1/stocks/${symbol}`);
    return response;
  } catch (error) {
    console.error('Error fetching stock data:', error);
    return null;
  }
}

export async function preloadStocks(symbols: string[]) {
  try {
    return await serverPost('/api/v1/stocks/preload', { symbols });
  } catch (error) {
    console.error('Error preloading stocks:', error);
    throw error;
  }
}

export async function checkStockCacheStatus(symbols: string[]) {
  try {
    return await serverPost('/api/v1/stocks/cache-status', { symbols });
  } catch (error) {
    console.error('Error checking stock cache status:', error);
    throw error;
  }
}

export async function getStockRankings(params?: {
  tier?: string;
  page?: number;
  limit?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}) {
  try {
    return await serverGet('/api/v1/stocks/rankings', params);
  } catch (error) {
    console.error('Error fetching stock rankings:', error);
    throw error;
  }
}

export async function getUserStockAccess() {
  try {
    return await serverGet('/api/v1/user/stock-access');
  } catch (error) {
    console.error('Error fetching user stock access:', error);
    return { allowed: false, tier: 'BRONZE' };
  }
}

export async function getWatchlist() {
  try {
    return await serverGet('/api/v1/user/watchlist');
  } catch (error) {
    console.error('Error fetching watchlist:', error);
    return [];
  }
}

export async function addToWatchlist(symbol: string) {
  try {
    return await serverPost('/api/v1/user/watchlist', { symbol });
  } catch (error) {
    console.error('Error adding to watchlist:', error);
    throw error;
  }
}

export async function removeFromWatchlist(symbol: string) {
  try {
    return await serverPost('/api/v1/user/watchlist/remove', { symbol });
  } catch (error) {
    console.error('Error removing from watchlist:', error);
    throw error;
  }
}