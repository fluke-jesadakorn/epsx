import type { CacheData, StockFinancialData } from '@/types/financialChartData';

const CACHE_KEY = 'stock_financial_data';
const DEFAULT_TTL = 300; // 5 minutes in seconds

export class StockDataCache {
  private static getCache(): CacheData<StockFinancialData[]> | null {
    try {
      const cached = localStorage.getItem(CACHE_KEY);
      if (!cached) return null;

      return JSON.parse(cached) as CacheData<StockFinancialData[]>;
    } catch (error) {
      console.error('Error reading cache:', error);
      return null;
    }
  }

  private static setCache(
    data: StockFinancialData[],
    ttl: number = DEFAULT_TTL,
  ): void {
    try {
      const cacheData: CacheData<StockFinancialData[]> = {
        data,
        timestamp: Date.now(),
        ttl: ttl * 1000, // Convert to milliseconds
      };

      localStorage.setItem(CACHE_KEY, JSON.stringify(cacheData));
    } catch (error) {
      console.error('Error setting cache:', error);
    }
  }

  static get(): StockFinancialData[] | null {
    const cached = this.getCache();
    if (!cached) return null;

    const now = Date.now();
    const isExpired = now - cached.timestamp > cached.ttl;

    if (isExpired) {
      this.clear();
      return null;
    }

    return cached.data;
  }

  static set(data: StockFinancialData[], ttl?: number): void {
    this.setCache(data, ttl);
  }

  static clear(): void {
    try {
      localStorage.removeItem(CACHE_KEY);
    } catch (error) {
      console.error('Error clearing cache:', error);
    }
  }

  static isExpired(): boolean {
    const cached = this.getCache();
    if (!cached) return true;

    const now = Date.now();
    return now - cached.timestamp > cached.ttl;
  }
}
