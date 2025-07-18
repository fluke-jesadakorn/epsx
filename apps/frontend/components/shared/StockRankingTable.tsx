'use client';

import React from 'react';
import FinancialDataTable from '@/components/home/FinancialDataTable';
import type { StockFinancialData } from '@/types/financialChartData';

interface StockRankingTableProps {
  data: StockFinancialData[];
  title?: string;
  subtitle?: string;
  showRank?: boolean;
  rankShift?: number; // For future difference - shift rank only
}

/**
 * Reusable Stock Ranking Table component
 * Uses the same FinancialDataTable component as /analytics page
 * Supports rank shifting for different zones without logic changes
 */
export default function StockRankingTable({
  data,
  title = "🍯 Sweet Performance Rankings 📊",
  subtitle = "Discover the most delicious data insights with our comprehensive analytics",
  showRank = true,
  rankShift = 0,
}: StockRankingTableProps): React.JSX.Element {
  // Apply rank shift if needed (for future use)
  const processedData = React.useMemo(() => {
    if (rankShift === 0) return data;
    
    // Future implementation: shift ranks without changing logic
    // This will allow different zones to show different ranking perspectives
    return data.map((item, index) => ({
      ...item,
      // Add rank-related properties if needed
      displayRank: index + 1 + rankShift,
    }));
  }, [data, rankShift]);

  return (
    <div className="w-full">
      {/* Custom header section that can override FinancialDataTable's default header */}
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
          </div>
        )}
      </div>
      
      <FinancialDataTable 
        data={processedData}
        className="min-h-screen"
      />
    </div>
  );
}
