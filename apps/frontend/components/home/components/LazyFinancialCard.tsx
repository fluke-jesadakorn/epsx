import React, { useState, useEffect } from 'react';
import { FinancialCard } from './FinancialCard';
import { SimpleCardSkeleton } from '../../common/SimpleCardSkeleton';
import type { StockFinancialData } from '@/types/financialChartData';
import { Card, CardContent } from '@/components/ui/card';

interface LazyFinancialCardProps {
  symbol: string;
  index: number;
  delay?: number;
}

/**
 * Lazy loading financial card that fetches data individually
 * Shows skeleton while loading, then renders the actual card
 */
export function LazyFinancialCard({
  symbol,
  index,
  delay = 0,
}: LazyFinancialCardProps): React.JSX.Element {
  const [data, setData] = useState<StockFinancialData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    const fetchCardData = async () => {
      try {
        // Reduce delay for faster loading
        if (delay > 0) {
          await new Promise(resolve => setTimeout(resolve, Math.min(delay, 300))); // Max 300ms delay
        }

        // Use the new individual API with server-side cache
        const response = await fetch(`/api/market-data/stocks/individual?symbol=${symbol}`);
        if (!response.ok) {
          throw new Error('Failed to fetch data');
        }

        const result = await response.json();
        
        // Check if we got an error response
        if (result.error) {
          throw new Error(result.error);
        }
        
        setData(result);
      } catch (err) {
        console.error(`Error fetching data for ${symbol}:`, err);
        setError(true);
      } finally {
        setLoading(false);
      }
    };

    fetchCardData();
  }, [symbol, delay]);

  if (loading) {
    return <SimpleCardSkeleton index={index} />;
  }

  if (error || !data) {
    return <FinancialCardError symbol={symbol} index={index} />;
  }

  return <FinancialCard data={data} index={index} />;
}

/**
 * Error component for failed card loads
 */
function FinancialCardError({ 
  symbol, 
  index 
}: { 
  symbol: string; 
  index: number; 
}): React.JSX.Element {
  return (
    <Card className="w-full transition-all duration-200 border-0 bg-gradient-to-br from-red-50/50 via-orange-50/50 to-yellow-50/50 dark:from-red-900/20 dark:via-orange-900/20 dark:to-yellow-900/20 rounded-3xl shadow-lg relative">
      {/* Rank Badge */}
      <div className="absolute top-4 left-4 z-30 w-12 h-12 flex items-center justify-center">
        <div className="w-10 h-10 bg-gradient-to-br from-red-200/70 via-orange-100/60 to-yellow-100/50 dark:from-red-900/40 dark:via-orange-900/30 dark:to-yellow-900/30 rounded-full flex items-center justify-center">
          <span className="text-sm font-bold text-slate-600 dark:text-slate-400">
            #{index + 1}
          </span>
        </div>
      </div>

      <CardContent className="p-6 pt-16 text-center">
        <div className="space-y-4">
          <div className="text-4xl opacity-30">⚠️</div>
          <div>
            <h3 className="text-lg font-semibold text-slate-700 dark:text-slate-300">
              {symbol}
            </h3>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
              Unable to load data
            </p>
          </div>
          <button 
            onClick={() => window.location.reload()}
            className="text-sm text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300 underline"
          >
            Try again
          </button>
        </div>
      </CardContent>
    </Card>
  );
}

export default LazyFinancialCard;
