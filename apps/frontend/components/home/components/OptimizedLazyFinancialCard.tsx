import React from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { FinancialCard } from './FinancialCard';
import { SimpleCardSkeleton } from '../../common/SimpleCardSkeleton';
import { useStockData } from '@/hooks/useStockData';

interface OptimizedLazyFinancialCardProps {
  symbol: string;
  index: number;
  delay?: number;
}

/**
 * Optimized lazy loading financial card using server-side cache
 * Uses the new caching system for better performance
 */
export function OptimizedLazyFinancialCard({
  symbol,
  index,
  delay = 0,
}: OptimizedLazyFinancialCardProps): React.JSX.Element {
  const { data, loading, error, fromCache } = useStockData(symbol);

  // Apply delay only if not from cache
  const [showContent, setShowContent] = React.useState(fromCache);

  React.useEffect(() => {
    if (fromCache) {
      setShowContent(true);
    } else if (delay > 0) {
      const timer = setTimeout(() => setShowContent(true), Math.min(delay, 300));
      return () => clearTimeout(timer);
    } else {
      setShowContent(true);
    }
  }, [fromCache, delay]);

  if (!showContent || loading) {
    return <SimpleCardSkeleton index={index} />;
  }

  if (error || !data) {
    return <FinancialCardError symbol={symbol} index={index} error={error} />;
  }

  return (
    <div className="relative">
      {/* Cache indicator */}
      {fromCache && (
        <div className="absolute top-2 right-2 z-50 bg-green-500 text-white text-xs px-2 py-1 rounded-full opacity-75">
          ⚡ Cached
        </div>
      )}
      <FinancialCard data={data} index={index} />
    </div>
  );
}

/**
 * Enhanced error component with retry functionality
 */
function FinancialCardError({ 
  symbol, 
  index,
  error = null,
}: { 
  symbol: string; 
  index: number; 
  error?: string | null;
}): React.JSX.Element {
  const [retrying, setRetrying] = React.useState(false);

  const handleRetry = async () => {
    setRetrying(true);
    // Clear cache for this symbol and trigger reload
    try {
      await fetch(`/api/v1/system/cache?symbol=${symbol}`, { method: 'DELETE' });
      window.location.reload();
    } catch (err) {
      console.error('Retry failed:', err);
    } finally {
      setRetrying(false);
    }
  };

  return (
    <Card className="w-full h-64 flex items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-600">
      <CardContent className="text-center">
        <div className="text-gray-500 dark:text-gray-400 mb-4">
          <span className="text-2xl mb-2 block">⚠️</span>
          <h3 className="text-lg font-semibold mb-2">
            Failed to load {symbol} (#{index + 1})
          </h3>
          {error && (
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
              {error}
            </p>
          )}
          <button 
            onClick={handleRetry}
            disabled={retrying}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
          >
            {retrying ? '🔄 Retrying...' : '🔄 Retry'}
          </button>
        </div>
      </CardContent>
    </Card>
  );
}

export default OptimizedLazyFinancialCard;
