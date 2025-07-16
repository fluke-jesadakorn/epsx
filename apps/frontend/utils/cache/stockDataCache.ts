import type { CacheData, StockFinancialData } from '@/types/financialChartData';

const DEFAULT_TTL = 300; // 5 minutes in seconds

/**
 * Server-side cache for individual stock data per card
 * Removed localStorage dependency for server-only caching
 */
export class StockDataCache {
  // Server-side memory cache for individual stock data
  private static cache = new Map<string, CacheData<StockFinancialData>>();

  /**
   * Get cached data for a specific stock symbol
   */
  static get(symbol: string): StockFinancialData | null {
    const cached = this.cache.get(symbol);
    if (!cached) return null;

    const now = Date.now();
    const isExpired = now - cached.timestamp > cached.ttl;

    if (isExpired) {
      this.cache.delete(symbol);
      return null;
    }

    return cached.data;
  }

  /**
   * Set cached data for a specific stock symbol
   */
  static set(symbol: string, data: StockFinancialData, ttl: number = DEFAULT_TTL): void {
    const cacheData: CacheData<StockFinancialData> = {
      data,
      timestamp: Date.now(),
      ttl: ttl * 1000, // Convert to milliseconds
    };

    this.cache.set(symbol, cacheData);
  }

  /**
   * Clear cache for a specific symbol
   */
  static clear(symbol?: string): void {
    if (symbol) {
      this.cache.delete(symbol);
    } else {
      this.cache.clear();
    }
  }

  /**
   * Check if cache for a specific symbol is expired
   */
  static isExpired(symbol: string): boolean {
    const cached = this.cache.get(symbol);
    if (!cached) return true;

    const now = Date.now();
    return now - cached.timestamp > cached.ttl;
  }

  /**
   * Get all cached symbols
   */
  static getCachedSymbols(): string[] {
    return Array.from(this.cache.keys());
  }

  /**
   * Get cache size
   */
  static size(): number {
    return this.cache.size;
  }

  /**
   * Clean up expired entries
   */
  static cleanup(): void {
    const now = Date.now();
    for (const [symbol, cached] of this.cache.entries()) {
      if (now - cached.timestamp > cached.ttl) {
        this.cache.delete(symbol);
      }
    }
  }
}
