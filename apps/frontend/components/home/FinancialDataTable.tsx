'use client';

import React from 'react';
import { GRADIENTS, SPACING } from './constants/styles';
import { FinancialCard } from './components/FinancialCard';
import {
  FinancialDataLoading,
  FinancialDataHeader,
} from './components/LayoutComponents';
import type { StockFinancialData } from '@/types/financialChartData';

interface FinancialDataTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: StockFinancialData[];
}

/**
 * Main financial data table component - refactored for better maintainability
 */
function FinancialDataTable({
  style,
  className,
  data,
}: FinancialDataTableProps): React.JSX.Element {
  // Ensure data is always an array to prevent runtime errors
  const safeData = Array.isArray(data) ? data : [];

  // Show loading state when no data
  if (!safeData.length) {
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
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        <div className="absolute top-20 left-4 sm:left-10 w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
        <div className="absolute top-40 right-8 sm:right-20 w-8 sm:w-12 h-8 sm:h-12 bg-gradient-to-br from-amber-400/20 to-orange-400/20 rounded-full animate-bounce-gentle" />
        <div className="absolute bottom-40 left-8 sm:left-20 w-6 sm:w-8 h-6 sm:h-8 bg-gradient-to-br from-yellow-400/20 to-amber-400/20 rounded-full animate-pulse" />
        <div className="absolute bottom-20 right-4 sm:right-10 w-16 sm:w-20 h-16 sm:h-20 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float-reverse" />

        {/* Floating emojis - hide on very small screens */}
        <div className="hidden sm:block absolute top-1/4 left-1/4 text-3xl sm:text-4xl opacity-10 animate-spin-slow">
          🥞
        </div>
        <div className="absolute top-3/4 right-1/4 text-2xl sm:text-3xl opacity-15 animate-bounce-gentle">
          📈
        </div>
        <div className="absolute bottom-1/4 left-1/3 text-xl sm:text-2xl opacity-20 animate-float">
          💰
        </div>
        <div className="hidden md:block absolute top-1/2 right-10 text-2xl sm:text-3xl opacity-15 animate-pulse">
          ⚡
        </div>
        <div className="absolute bottom-1/3 right-1/3 text-xl sm:text-2xl opacity-10 animate-bounce-gentle">
          🚀
        </div>
      </div>

      {/* Header Section */}
      <FinancialDataHeader />

      {/* Cards Container */}
      <div
        className={`relative ${SPACING.mobileContainer} ${SPACING.verticalSpacing}`}
      >
        <div className="max-w-7xl mx-auto">
          {/* Section Title with PancakeSwap style */}
          <div className="text-center mb-8 sm:mb-10 md:mb-12">
            <h2 className="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold mb-3 sm:mb-4">
              <span className="bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600 bg-clip-text text-transparent">
                🍯 Sweet Financial Rankings 📊
              </span>
            </h2>
            <p className="text-base sm:text-lg md:text-xl text-slate-600 dark:text-slate-300 max-w-xl sm:max-w-2xl mx-auto px-4 sm:px-0">
              Discover the most delicious investment opportunities with our
              comprehensive analytics
            </p>
          </div>

          {/* Responsive Cards Grid */}
          <div
            className="
              grid 
              grid-cols-1 
              sm:grid-cols-1 
              md:grid-cols-2 
              xl:grid-cols-3 
              gap-4 sm:gap-6
              auto-rows-max
              w-full
              max-w-full
            "
          >
            {safeData.map((item, index) => (
              <div
                key={`${item.symbol}-${index}`}
                className="relative w-full max-w-full overflow-hidden"
                style={{
                  animationDelay: `${index * 150}ms`,
                  animationDuration: '600ms',
                  animationFillMode: 'both',
                }}
              >
                <FinancialCard data={item} index={index} />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default FinancialDataTable;
