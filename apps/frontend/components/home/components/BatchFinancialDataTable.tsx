'use client';

import React, { useState, useEffect } from 'react';
import { GRADIENTS, SPACING, ANIMATIONS } from '../constants/styles';
import { FinancialCard } from './FinancialCard';
import {
  FinancialDataLoading,
  FinancialDataHeader,
} from './LayoutComponents';
import type { StockFinancialData } from '@/types/financialChartData';
import { Card, CardContent } from '@/components/ui/card';

interface BatchFinancialDataTableProps {
  style?: React.CSSProperties;
  className?: string;
  maxCards?: number;
  useBatchLoading?: boolean;
}

/**
 * Batch loading version of FinancialDataTable
 * Shows skeletons immediately, then loads all data at once
 */
function BatchFinancialDataTable({
  style,
  className,
  maxCards = 10,
  useBatchLoading = true,
}: BatchFinancialDataTableProps): React.JSX.Element {
  const [data, setData] = useState<StockFinancialData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        // Use the existing stock action for faster data fetching
        const { fetchStockData } = await import('@/app/actions/stock');
        const result = await fetchStockData();
        // Limit the results based on maxCards
        const limitedResult = Array.isArray(result) ? result.slice(0, maxCards) : [];
        setData(limitedResult);
      } catch (err) {
        console.error('Error fetching batch data:', err);
        setError(true);
      } finally {
        setLoading(false);
      }
    };

    if (useBatchLoading) {
      // Add a small delay to prevent blocking the UI
      const timeoutId = setTimeout(fetchData, 100);
      return () => clearTimeout(timeoutId);
    }
  }, [maxCards, useBatchLoading]);

  // Show global loading only if no batch loading
  if (!useBatchLoading) {
    return <FinancialDataLoading />;
  }

  return (
    <div
      className={`
        w-full min-h-screen 
        bg-gradient-to-br ${GRADIENTS.background}
        transition-all duration-500 
        relative overflow-hidden
        ${className || ''}
      `}
      style={style}
    >
      {/* PancakeSwap-style floating decorative elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-20 left-10 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
        <div className="absolute top-40 right-20 w-12 h-12 bg-gradient-to-br from-amber-400/20 to-orange-400/20 rounded-full animate-bounce-gentle" />
        <div className="absolute bottom-40 left-20 w-8 h-8 bg-gradient-to-br from-yellow-400/20 to-amber-400/20 rounded-full animate-pulse" />
        <div className="absolute bottom-20 right-10 w-20 h-20 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float-reverse" />

        {/* Floating emojis */}
        <div className="absolute top-1/4 left-1/4 text-4xl opacity-10 animate-spin-slow">
          🥞
        </div>
        <div className="absolute top-3/4 right-1/4 text-3xl opacity-15 animate-bounce-gentle">
          📈
        </div>
        <div className="absolute bottom-1/4 left-1/3 text-2xl opacity-20 animate-float">
          💰
        </div>
        <div className="absolute top-1/2 right-10 text-3xl opacity-15 animate-pulse">
          ⚡
        </div>
        <div className="absolute bottom-1/3 right-1/3 text-2xl opacity-10 animate-bounce-gentle">
          🚀
        </div>
      </div>

      {/* Header Section */}
      <FinancialDataHeader />

      {/* Cards Container */}
      <div
        className={`relative ${SPACING.containerPadding} ${SPACING.verticalSpacing}`}
      >
        <div className="max-w-7xl mx-auto">
          {/* Section Title with PancakeSwap style */}
          <div className="text-center mb-12">
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              <span className="bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600 bg-clip-text text-transparent">
                🍯 Sweet Financial Rankings 📊
              </span>
            </h2>
            <p className="text-lg text-slate-600 dark:text-slate-300 max-w-2xl mx-auto">
              Discover the most delicious investment opportunities with our
              comprehensive analytics
            </p>
          </div>

          <div
            className={`grid grid-cols-1 lg:grid-cols-2 ${SPACING.sectionGap}`}
          >
            {loading ? (
              // Show skeleton cards while loading
              [...Array(maxCards)].map((_, index) => (
                <div
                  key={`skeleton-${index}`}
                  className={ANIMATIONS.fadeIn + ' relative'}
                  style={{
                    animationDelay: `${index * 20}ms`, // Much faster - 20ms instead of 50ms
                    animationDuration: '200ms', // Faster animation - 200ms instead of 300ms
                    animationFillMode: 'both',
                  }}
                >
                  <Card className="w-full transition-all duration-200 border-0 bg-gradient-to-br from-blue-50/30 via-purple-50/30 to-pink-50/30 dark:from-[#232946]/30 dark:via-[#1a1a2e]/30 dark:to-[#0f1021]/30 rounded-3xl shadow-lg relative">
                    <div className="absolute top-4 left-4 z-30 w-12 h-12 flex items-center justify-center">
                      <div className="w-10 h-10 bg-gradient-to-br from-orange-200/50 via-yellow-100/40 to-amber-100/30 dark:from-orange-900/30 dark:via-yellow-900/20 dark:to-amber-900/20 rounded-full flex items-center justify-center">
                        <span className="text-sm font-bold text-slate-600 dark:text-slate-400">
                          #{index + 1}
                        </span>
                      </div>
                    </div>
                    <CardContent className="p-6 pt-16">
                      <div className="space-y-4">
                        <div className="flex items-center justify-center py-4">
                          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500"></div>
                        </div>
                        <div className="text-center">
                          <div className="h-4 bg-gradient-to-r from-orange-300/20 to-yellow-300/20 rounded animate-pulse"></div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>
              ))
            ) : error ? (
              // Show error state
              <div className="col-span-full text-center py-12">
                <div className="text-4xl mb-4">⚠️</div>
                <h3 className="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">
                  Unable to load data
                </h3>
                <p className="text-sm text-slate-500 dark:text-slate-400 mb-4">
                  Please try refreshing the page
                </p>
                <button 
                  onClick={() => window.location.reload()}
                  className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
                >
                  Refresh
                </button>
              </div>
            ) : (
              // Show actual data
              data.map((item, index) => (
                <div
                  key={`${item.symbol}-${index}`}
                  className={ANIMATIONS.fadeIn + ' relative'}
                  style={{
                    animationDelay: `${index * 20}ms`, // Much faster - 20ms instead of 50ms
                    animationDuration: '200ms', // Faster animation
                    animationFillMode: 'both',
                  }}
                >
                  <FinancialCard data={item} index={index} />
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default BatchFinancialDataTable;
