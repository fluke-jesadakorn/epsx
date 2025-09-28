import { useState, useEffect, useCallback } from 'react';
import type { StockFinancialData } from '@/types/financialChartData';
import { 
  getBatchStocks,
  getStockData as _getStockData,
  preloadStocks,
  checkStockCacheStatus
} from '@/lib/server-actions';

interface BatchStockData {
  [symbol: string]: StockFinancialData | null;
}

interface BatchFetchState {
  data: BatchStockData;
  loading: boolean;
  error: string | null;
  cached: string[];
  fetched: string[];
  errors: string[];
}

/**
 * Hook for fetching multiple stock symbols efficiently using server-side cache
 * Provides better performance than individual requests by batching requests
 */
export function useBatchStockData(symbols: string[]): BatchFetchState {
  const [state, setState] = useState<BatchFetchState>({
    data: {},
    loading: true,
    error: null,
    cached: [],
    fetched: [],
    errors: [],
  });

  const fetchBatch = useCallback(async (symbolsList: string[]) => {
    if (symbolsList.length === 0) {
      setState(prev => ({ ...prev, loading: false }));
      return;
    }

    try {
      setState(prev => ({ ...prev, loading: true, error: null }));

      const result = await getBatchStocks(symbolsList);
      
      if (!result.success) {
        const errorMessage = result.errors && result.errors.length > 0 ? result.errors.join(', ') : 'Failed to fetch batch data';
        throw new Error(errorMessage);
      }

      setState(prev => ({
        ...prev,
        data: { ...prev.data, ...result.data },
        loading: false,
        cached: result.cached || [],
        fetched: result.fetched || [],
        errors: result.errors || [],
      }));

    } catch (err) {
      console.error('Error in batch fetch:', err);
      setState(prev => ({
        ...prev,
        loading: false,
        error: err instanceof Error ? err.message : 'Unknown error occurred',
      }));
    }
  }, []);

  useEffect(() => {
    if (symbols.length > 0) {
      fetchBatch(symbols);
    }
  }, [symbols, fetchBatch]);

  return state;
}

/**
 * Hook for fetching a single stock symbol with caching
 * Falls back to the batch API for consistency
 */
export function useStockData(symbol: string): {
  data: StockFinancialData | null;
  loading: boolean;
  error: string | null;
  fromCache: boolean;
} {
  const batchResult = useBatchStockData([symbol]);
  
  return {
    data: batchResult.data[symbol] || null,
    loading: batchResult.loading,
    error: batchResult.error,
    fromCache: batchResult.cached.includes(symbol),
  };
}

/**
 * Hook for preloading stock symbols into cache
 * Useful for warming up the cache before users navigate to cards
 */
export function useStockPreloader() {
  const [preloading, setPreloading] = useState(false);

  const preload = useCallback(async (symbols: string[]) => {
    if (symbols.length === 0) return;

    setPreloading(true);
    try {
      const _response = await preloadStocks(symbols);
      // Preload completed
    } catch (error) {
      console.error('Preload error:', error);
    } finally {
      setPreloading(false);
    }
  }, []);

  const checkCacheStatus = useCallback(async (symbols: string[]) => {
    try {
      const response = await checkStockCacheStatus(symbols);
      return response.symbols || [];
    } catch (error) {
      console.error('Cache status check error:', error);
      return [];
    }
  }, []);

  return { preload, checkCacheStatus, preloading };
}
