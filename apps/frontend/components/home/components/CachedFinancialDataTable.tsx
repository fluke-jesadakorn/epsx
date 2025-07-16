import React, { useState, useEffect } from 'react';
import { GRADIENTS, SPACING, ANIMATIONS } from '../constants/styles';
import { OptimizedLazyFinancialCard } from './OptimizedLazyFinancialCard';
import { FinancialCard } from './FinancialCard';
import { FinancialDataLoading, FinancialDataHeader } from './LayoutComponents';
import { useBatchStockData, useStockPreloader } from '@/hooks/useStockData';
import type { StockFinancialData } from '@/types/financialChartData';

interface CachedFinancialDataTableProps {
  style?: React.CSSProperties;
  className?: string;
  symbols?: string[];
  initialData?: StockFinancialData[];
  maxCards?: number;
  enablePreloading?: boolean;
}

/**
 * Advanced financial data table with server-side caching and preloading
 * Uses the new per-card caching system for optimal performance
 */
function CachedFinancialDataTable({
  style,
  className,
  symbols = [],
  initialData = [],
  maxCards = 10,
  enablePreloading = true,
}: CachedFinancialDataTableProps): React.JSX.Element {
  const [allSymbols, setAllSymbols] = useState<string[]>([]);
  const { preload, preloading } = useStockPreloader();

  // Determine symbols to use
  const cardSymbols = symbols.length > 0 
    ? symbols.slice(0, maxCards)
    : allSymbols.slice(0, maxCards);

  // Fetch batch data for all symbols
  const { data: batchData, cached, fetched } = useBatchStockData(cardSymbols);

  // Extract symbols from initial data if no symbols provided
  useEffect(() => {
    if (symbols.length === 0 && initialData.length > 0) {
      const extractedSymbols = initialData.map(item => item.symbol);
      setAllSymbols(extractedSymbols);
    } else if (symbols.length > 0) {
      setAllSymbols(symbols);
    }
  }, [symbols, initialData]);

  // Preload symbols for better performance
  useEffect(() => {
    if (enablePreloading && cardSymbols.length > 0 && !preloading) {
      // Preload symbols that aren't already cached
      const uncachedSymbols = cardSymbols.filter(symbol => !cached.includes(symbol));
      if (uncachedSymbols.length > 0) {
        preload(uncachedSymbols);
      }
    }
  }, [cardSymbols, cached, enablePreloading, preload, preloading]);

  // Show loading state when no symbols are available
  if (cardSymbols.length === 0) {
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
      {/* Background decorative elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-20 left-10 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
        <div className="absolute top-40 right-20 w-12 h-12 bg-gradient-to-br from-amber-400/20 to-orange-400/20 rounded-full animate-bounce-gentle" />
        <div className="absolute bottom-40 left-20 w-8 h-8 bg-gradient-to-br from-yellow-400/20 to-amber-400/20 rounded-full animate-pulse" />
        <div className="absolute bottom-20 right-10 w-20 h-20 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float-reverse" />
      </div>

      {/* Header Section */}
      <FinancialDataHeader />

      {/* Cache Status Indicator */}
      <div className="relative z-10 text-center mb-6">
        <div className="inline-flex items-center gap-4 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm rounded-full px-6 py-3 shadow-lg">
          <div className="flex items-center gap-2">
            <span className="text-green-500">⚡</span>
            <span className="text-sm font-medium">Cached: {cached.length}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-blue-500">🌐</span>
            <span className="text-sm font-medium">Fetched: {fetched.length}</span>
          </div>
          {preloading && (
            <div className="flex items-center gap-2">
              <span className="text-orange-500 animate-spin">⚙️</span>
              <span className="text-sm font-medium">Preloading...</span>
            </div>
          )}
        </div>
      </div>

      {/* Cards Container */}
      <div className={`relative ${SPACING.containerPadding} ${SPACING.verticalSpacing}`}>
        <div className="max-w-7xl mx-auto">
          {/* Section Title */}
          <div className="text-center mb-12">
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              <span className="bg-gradient-to-r from-orange-500 via-yellow-400 to-orange-600 bg-clip-text text-transparent">
                🍯 Cached Financial Rankings 📊
              </span>
            </h2>
            <p className="text-lg text-slate-600 dark:text-slate-300 max-w-2xl mx-auto">
              Lightning-fast data with intelligent server-side caching
            </p>
          </div>

          <div className={`grid grid-cols-1 lg:grid-cols-2 ${SPACING.sectionGap}`}>
            {cardSymbols.map((symbol, index) => {
              const stockData = batchData[symbol];
              
              return (
                <div
                  key={`${symbol}-${index}`}
                  className={ANIMATIONS.fadeIn + ' relative'}
                  style={{
                    animationDelay: `${index * 100}ms`,
                    animationDuration: '400ms',
                    animationFillMode: 'both',
                  }}
                >
                  {/* If we have data in batch results, use regular FinancialCard */}
                  {stockData ? (
                    <div className="relative">
                      {cached.includes(symbol) && (
                        <div className="absolute top-2 right-2 z-50 bg-green-500 text-white text-xs px-2 py-1 rounded-full opacity-75">
                          ⚡ Cached
                        </div>
                      )}
                      <FinancialCard data={stockData} index={index} />
                    </div>
                  ) : (
                    /* Otherwise use optimized lazy card */
                    <OptimizedLazyFinancialCard
                      symbol={symbol}
                      index={index}
                      delay={index * 50} // Reduced delay
                    />
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

export default CachedFinancialDataTable;
