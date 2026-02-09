/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument */
'use client';

import React from 'react';
import { GRADIENTS, SPACING } from './constants/styles';
import { FinancialCard } from './components/financial-card';
import {
  FinancialDataLoading,
  FinancialDataHeader,
} from './components/layout-components';
import { EnhancedTouchWrapper } from '@/components/touch';
import { Heart, Share2, Bookmark, TrendingUp, ExternalLink } from 'lucide-react';
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

  // Enhanced touch interaction handlers
  const handleRefresh = async () => {
    // Simulate data refresh
    await new Promise(resolve => setTimeout(resolve, 1500));
  };

  const stockQuickActions = [
    {
      id: 'favorite',
      icon: <Heart className="h-5 w-5" />,
      label: 'Favorite',
      color: 'bg-red-500',
      action: () => {}
    },
    {
      id: 'share',
      icon: <Share2 className="h-5 w-5" />,
      label: 'Share',
      color: 'bg-blue-500',
      action: () => {}
    },
    {
      id: 'watchlist',
      icon: <Bookmark className="h-5 w-5" />,
      label: 'Watch',
      color: 'bg-green-500',
      action: () => {}
    },
    {
      id: 'analyze',
      icon: <TrendingUp className="h-5 w-5" />,
      label: 'Analyze',
      color: 'bg-purple-500',
      action: () => {}
    },
    {
      id: 'tradingview',
      icon: <ExternalLink className="h-5 w-5" />,
      label: 'TradingView',
      color: 'bg-orange-500',
      action: () => {}
    }
  ];

  // Show loading state when no data
  if (!safeData.length) {
    return <FinancialDataLoading />;
  }

  return (
    <EnhancedTouchWrapper
      enablePullToRefresh={true}
      onRefresh={handleRefresh}
      enableLongPress={true}
      enableQuickActions={true}
      quickActions={stockQuickActions}
      enablePinchZoom={true}
      className={`
        w-full min-h-screen 
        bg-gradient-to-br ${GRADIENTS.background}
        transition-all duration-500 
        relative overflow-hidden
        ${className ?? ''}
      `}
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

          {/* Mobile View Toggle Buttons */}
          <div className="flex flex-wrap gap-2 mb-6 md:hidden">
            <button className="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium">
              Card View
            </button>
            <button className="px-4 py-2 bg-muted text-muted-foreground rounded-lg text-sm font-medium hover:bg-primary hover:text-primary-foreground transition-colors">
              Table View
            </button>
          </div>

          {/* Mobile/Tablet: Horizontal Scrolling Cards */}
          <div className="block md:hidden">
            <div className="overflow-x-auto pb-4">
              <div className="flex gap-4 w-max">
                {safeData.map((item) => (
                  <div
                    key={item.symbol}
                    className="w-72 flex-shrink-0"
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
              {/* Scroll indicator */}
              <div className="flex justify-center mt-4">
                <div className="flex gap-1">
                  {Array.from({ length: Math.min(5, safeData.length) }).map((_, i) => (
                    <div key={`dot-${String(i)}`} className="w-2 h-2 bg-primary/30 rounded-full" />
                  ))}
                </div>
                <p className="text-xs text-muted-foreground ml-3 self-center">
                  Swipe to see more →
                </p>
              </div>
            </div>
          </div>

          {/* Desktop: Responsive Grid */}
          <div className="hidden md:block">
            <div
              className="
                grid 
                grid-cols-1 
                md:grid-cols-2 
                lg:grid-cols-2
                xl:grid-cols-3 
                2xl:grid-cols-4
                gap-4 lg:gap-6
                auto-rows-max
                w-full
                max-w-full
              "
            >
              {safeData.map((item) => (
                <div
                  key={`desktop-${item.symbol}`}
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

          {/* Load More Button - Mobile Optimized */}
          <div className="mt-8 text-center">
            <button className="px-6 py-3 bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-xl font-semibold shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300 w-full sm:w-auto">
              <span className="flex items-center justify-center gap-2">
                <span>📈 Load More Data</span>
                <span className="text-sm opacity-80">(+{Math.min(20, safeData.length)} more)</span>
              </span>
            </button>
            
            {/* Mobile hint */}
            <p className="text-xs text-muted-foreground mt-3 block sm:hidden">
              Tap to load more financial data
            </p>
          </div>
        </div>
      </div>
    </EnhancedTouchWrapper>
  );
}

export default FinancialDataTable;
