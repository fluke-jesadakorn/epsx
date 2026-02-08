'use client';

import React from 'react';
import FinancialDataTable from '@/components/home/financial-data-table';
import BatchFinancialDataTable from '@/components/home/components/Batchfinancial-data-table';
import type { StockFinancialData } from '@/types/financialChartData';

interface LazyStockRankingTableProps {
  data?: StockFinancialData[];
  title?: string;
  subtitle?: string;
  showRank?: boolean;
  rankShift?: number;
  maxCards?: number;
  useLazyLoading?: boolean;
}

/**
 * Performance Ranking Table with lazy loading support
 * Can work with pre-loaded data or fetch symbols and load cards progressively
 */
export default function LazyStockRankingTable({
  data = [],
  title = "🍯 Sweet Performance Rankings 📊",
  subtitle = "Discover the most delicious data insights with our comprehensive analytics",
  showRank = true,
  rankShift = 0,
  maxCards = 10,
  useLazyLoading = false,
}: LazyStockRankingTableProps): React.JSX.Element {
  // Apply rank shift if needed (for future use)
  const processedData = React.useMemo(() => {
    if (rankShift === 0) {return data;}
    
    return data.map((item, index) => ({
      ...item,
      displayRank: index + 1 + rankShift,
    }));
  }, [data, rankShift]);

  // Use batch loading when lazy loading is enabled and no data provided
  const shouldUseBatchLoading = useLazyLoading && data.length === 0;

  return (
    <div className="w-full">
      {/* Custom header section */}
      <div className="text-center py-8">
        <h2 className="text-3xl sm:text-4xl font-bold mb-4">
          <span className="bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600 bg-clip-text text-transparent">
            {title}
          </span>
        </h2>
        <p className="text-lg text-slate-600 dark:text-slate-300 max-w-2xl mx-auto">
          {subtitle}
        </p>
        {showRank && (
          <div className="mt-2 text-sm text-slate-500 dark:text-slate-400">
            {rankShift !== 0 && `Ranking shifted by ${rankShift} positions`}
            {useLazyLoading && " • Progressive loading enabled"}
          </div>
        )}
      </div>
      
      {/* Render appropriate table component */}
      {shouldUseBatchLoading ? (
        <BatchFinancialDataTable 
          maxCards={maxCards}
          useBatchLoading
          className="min-h-screen"
        />
      ) : (
        <FinancialDataTable 
          data={processedData}
          className="min-h-screen"
        />
      )}
    </div>
  );
}
