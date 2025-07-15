'use client';

import React from 'react';
import { GRADIENTS, SPACING, ANIMATIONS } from '../constants/styles';
import { LazyFinancialCard } from './LazyFinancialCard';
import { FinancialCard } from './FinancialCard';
import {
  FinancialDataLoading,
  FinancialDataHeader,
} from './LayoutComponents';
import type { StockFinancialData } from '@/types/financialChartData';

interface LazyFinancialDataTableProps {
  style?: React.CSSProperties;
  className?: string;
  symbols?: string[];
  initialData?: StockFinancialData[];
  maxCards?: number;
}

/**
 * Lazy loading version of FinancialDataTable
 * Loads cards progressively to prevent blocking the UI
 */
function LazyFinancialDataTable({
  style,
  className,
  symbols = [],
  initialData = [],
  maxCards = 10,
}: LazyFinancialDataTableProps): React.JSX.Element {
  // Use either provided symbols or extract from initial data
  const cardSymbols = symbols.length > 0 
    ? symbols.slice(0, maxCards)
    : initialData.slice(0, maxCards).map(item => item.symbol);

  // Show global loading only if no symbols are provided and no initial data
  if (cardSymbols.length === 0 && initialData.length === 0) {
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
            {cardSymbols.map((symbol, index) => (
              <div
                key={`${symbol}-${index}`}
                className={ANIMATIONS.fadeIn + ' relative'}
                style={{
                  animationDelay: `${index * 150}ms`,
                  animationDuration: '600ms',
                  animationFillMode: 'both',
                }}
              >
                {/* If we have initial data, use the regular FinancialCard */}
                {initialData.length > 0 && initialData[index] ? (
                  <FinancialCard data={initialData[index]} index={index} />
                ) : (
                  /* Otherwise use LazyFinancialCard */
                  <LazyFinancialCard
                    symbol={symbol}
                    index={index}
                    delay={index * 100} // Reduced stagger delay from 200ms to 100ms
                  />
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default LazyFinancialDataTable;
