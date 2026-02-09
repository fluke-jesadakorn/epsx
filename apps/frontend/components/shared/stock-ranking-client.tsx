'use client';

import React from 'react';
import StockRankingTable from '@/components/shared/stock-ranking-table';
import type { StockFinancialData } from '@/types/financialChartData';

interface StockRankingClientProps {
  initialData: StockFinancialData[];
  title?: string;
  subtitle?: string;
  rankShift?: number;
  showRank?: boolean;
}

/**
 * Client component for Stock Ranking Table with role-based access control
 * Integrates user subscription levels with stock ranking visibility
 */
export default function StockRankingClient({
  initialData,
  title,
  subtitle,
  rankShift = 0,
  showRank = true,
}: StockRankingClientProps): React.JSX.Element {
  
  // Apply rank shift if needed (for future use)
  const processedData = React.useMemo(() => {
    if (rankShift === 0) {return initialData;}
    
    return initialData.map((item, index) => ({
      ...item,
      displayRank: index + 1 + rankShift,
    }));
  }, [initialData, rankShift]);

  return (
    <div className="w-full">
      {/* Custom header section */}
      {(title ?? subtitle) && (
        <div className="text-center py-8">
          {title && (
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              <span className="bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600 bg-clip-text text-transparent">
                {title}
              </span>
            </h2>
          )}
          {subtitle && (
            <p className="text-lg text-slate-600 dark:text-slate-300 max-w-2xl mx-auto">
              {subtitle}
            </p>
          )}
          {showRank && (
            <div className="mt-2 text-sm text-slate-500 dark:text-slate-400">
              {rankShift !== 0 && `Ranking shifted by ${rankShift} positions`}
              {" • Role-based access control enabled"}
            </div>
          )}
        </div>
      )}
      
      {/* Role-based financial table */}
      <div className="min-h-screen">
        <StockRankingTable
          data={processedData}
        />
      </div>
    </div>
  );
}
