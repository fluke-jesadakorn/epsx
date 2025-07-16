import { StockDataCache } from './stockDataCache';
import type { StockFinancialData } from '@/types/financialChartData';

/**
 * Cache Manager for managing server-side stock data cache
 * Provides utilities for bulk operations and maintenance
 */
export class CacheManager {
  /**
   * Set multiple stock data entries at once
   */
  static setBulk(stockData: StockFinancialData[], ttl?: number): void {
    stockData.forEach(data => {
      StockDataCache.set(data.symbol, data, ttl);
    });
  }

  /**
   * Get multiple stock data entries by symbols
   */
  static getBulk(symbols: string[]): Record<string, StockFinancialData | null> {
    const result: Record<string, StockFinancialData | null> = {};
    
    symbols.forEach(symbol => {
      result[symbol] = StockDataCache.get(symbol);
    });

    return result;
  }

  /**
   * Get cache statistics
   */
  static getStats() {
    const cachedSymbols = StockDataCache.getCachedSymbols();
    let expiredCount = 0;

    cachedSymbols.forEach(symbol => {
      if (StockDataCache.isExpired(symbol)) {
        expiredCount++;
      }
    });

    return {
      totalEntries: StockDataCache.size(),
      cachedSymbols,
      expiredEntries: expiredCount,
      validEntries: cachedSymbols.length - expiredCount,
    };
  }

  /**
   * Auto cleanup expired entries
   * Should be called periodically
   */
  static scheduleCleanup(intervalMs: number = 5 * 60 * 1000): NodeJS.Timeout {
    return setInterval(() => {
      const beforeSize = StockDataCache.size();
      StockDataCache.cleanup();
      const afterSize = StockDataCache.size();
      
      if (beforeSize > afterSize) {
        console.log(`[CacheManager] Cleaned up ${beforeSize - afterSize} expired entries`);
      }
    }, intervalMs);
  }

  /**
   * Preload cache with symbols from a list
   * Useful for warming up cache with frequently accessed symbols
   */
  static async preloadSymbols(symbols: string[]): Promise<void> {
    const uncachedSymbols = symbols.filter(symbol => 
      !StockDataCache.get(symbol)
    );

    if (uncachedSymbols.length === 0) {
      return;
    }

    try {
      // Fetch data for uncached symbols
      // Note: This would need to be implemented based on your data source
      console.log(`[CacheManager] Preloading ${uncachedSymbols.length} symbols:`, uncachedSymbols);
      
      // For now, we'll just log. In a real implementation, you might:
      // 1. Call your data fetching service
      // 2. Transform the data
      // 3. Cache the results
      
    } catch (error) {
      console.error('[CacheManager] Error preloading symbols:', error);
    }
  }

  /**
   * Clear cache for specific symbols or all
   */
  static clearCache(symbols?: string[]): void {
    if (symbols) {
      symbols.forEach(symbol => StockDataCache.clear(symbol));
    } else {
      StockDataCache.clear();
    }
  }

  /**
   * Get cache hit rate for monitoring
   */
  static getCacheMetrics() {
    const stats = this.getStats();
    
    return {
      ...stats,
      hitRate: stats.totalEntries > 0 ? stats.validEntries / stats.totalEntries : 0,
      memoryUsage: `${stats.totalEntries} entries`,
    };
  }
}
