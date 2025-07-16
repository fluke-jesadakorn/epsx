import React from 'react';
import CachedFinancialDataTable from './CachedFinancialDataTable';
import { useStockPreloader } from '@/hooks/useStockData';

/**
 * Example component demonstrating the new caching system
 * This shows how to use the optimized components with preloading
 */
export function CachingSystemDemo(): React.JSX.Element {
  const { preload, checkCacheStatus } = useStockPreloader();

  // Example symbols for demonstration
  const demoSymbols = ['AAPL', 'GOOGL', 'MSFT', 'TSLA', 'AMZN', 'META', 'NVDA', 'NFLX', 'ADBE', 'CRM'];

  const handlePreload = async () => {
    console.log('Preloading symbols...');
    await preload(demoSymbols);
    console.log('Preload completed');
  };

  const handleCheckCache = async () => {
    const status = await checkCacheStatus(demoSymbols);
    console.log('Cache status:', status);
  };

  const handleClearCache = async () => {
    try {
      await fetch('/api/cache', { method: 'DELETE' });
      console.log('Cache cleared');
      window.location.reload();
    } catch (error) {
      console.error('Failed to clear cache:', error);
    }
  };

  const handleGetStats = async () => {
    try {
      const response = await fetch('/api/cache?action=stats');
      const stats = await response.json();
      console.log('Cache statistics:', stats);
    } catch (error) {
      console.error('Failed to get cache stats:', error);
    }
  };

  return (
    <div className="space-y-6">
      {/* Demo Controls */}
      <div className="bg-white dark:bg-slate-800 rounded-lg shadow-lg p-6">
        <h3 className="text-lg font-semibold mb-4">Cache Management Demo</h3>
        <div className="flex flex-wrap gap-3">
          <button
            onClick={handlePreload}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          >
            🚀 Preload Data
          </button>
          <button
            onClick={handleCheckCache}
            className="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors"
          >
            📊 Check Cache Status
          </button>
          <button
            onClick={handleGetStats}
            className="px-4 py-2 bg-purple-500 text-white rounded-lg hover:bg-purple-600 transition-colors"
          >
            📈 Get Statistics
          </button>
          <button
            onClick={handleClearCache}
            className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors"
          >
            🗑️ Clear Cache
          </button>
        </div>
        <p className="text-sm text-gray-600 dark:text-gray-400 mt-3">
          Check the browser console for detailed output from these actions.
        </p>
      </div>

      {/* Cached Financial Data Table */}
      <CachedFinancialDataTable
        symbols={demoSymbols}
        maxCards={10}
        enablePreloading={true}
      />
    </div>
  );
}

export default CachingSystemDemo;
