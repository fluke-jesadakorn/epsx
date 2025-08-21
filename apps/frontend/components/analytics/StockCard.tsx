'use client';

import type {
  CardStockProps,
  StockCardProps,
  SymbolCardData,
} from '@/types/analytics';
import {
  EPSGrowthTrend,
  formatPercentage,
  getGrowthTrend,
} from '@/types/analytics';
import { memo } from 'react';

// Support both old and new props interfaces
type StockCardAllProps = CardStockProps | StockCardProps;

const StockCard = memo<StockCardAllProps>(props => {
  // Check if it's new card format or old ranking format
  const isCardFormat = 'cardData' in props;

  let cardData: SymbolCardData;
  let quarters: any[];
  let latestQuarter: any;
  let previousQuarter: any;
  let latestEPS: number;
  let previousEPS: number;
  let latestGrowth: number;
  let previousGrowth: number;

  if (isCardFormat) {
    // New card format - switch display order for chronological flow
    cardData = props.cardData;
    quarters = cardData.quarterly_performance || [];
    
    // Display order: older quarter (Q2) on left, newer quarter (Q3) on right
    latestQuarter = quarters[0]; // Latest fiscal quarter (right side)
    previousQuarter = quarters[1]; // Previous fiscal quarter (left side)
    latestEPS = latestQuarter?.eps || 0;
    previousEPS = previousQuarter?.eps || 0;
    latestGrowth = latestQuarter?.eps_growth || 0;
    previousGrowth = previousQuarter?.eps_growth || 0;
  } else {
    // Old ranking format - convert to card format
    const { ranking, rank } = props;
    quarters = ranking.quarterly_data?.slice(0, 2) || [];
    latestQuarter = quarters[0];
    previousQuarter = quarters[1];
    latestEPS = latestQuarter?.eps || ranking.current_eps || 0;
    previousEPS = previousQuarter?.eps || 0;
    latestGrowth = latestQuarter?.eps_growth || ranking.qoq_growth || 0;
    previousGrowth = previousQuarter?.eps_growth || 0;


    // Convert to card format
    cardData = {
      rank: rank,
      symbol: ranking.symbol,
      latest_date: new Date().toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      }),
      value: ranking.price_current || 0,
      active_status: ranking.active_status || 'Non Active',
      quarterly_performance: quarters.map(q => ({
        quarter: q.quarter || q.period || 'Q4',
        date:
          q.date ||
          new Date().toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
          }),
        price: q.price || ranking.price_current || 0,
        eps: q.eps || ranking.current_eps || 0,
        eps_growth: q.eps_growth || ranking.qoq_growth || 0,
        price_growth: q.price_growth || 0,
      })),
    };
  }

  // Determine primary growth trend from latest quarter
  const growthTrend = getGrowthTrend(latestGrowth);
  const isPositiveGrowth = latestGrowth >= 0;

  const trendColors = {
    [EPSGrowthTrend.Accelerating]:
      'text-green-600 bg-green-50 border-green-200',
    [EPSGrowthTrend.Steady]: 'text-blue-600 bg-blue-50 border-blue-200',
    [EPSGrowthTrend.Decelerating]:
      'text-orange-600 bg-orange-50 border-orange-200',
    [EPSGrowthTrend.Volatile]: 'text-red-600 bg-red-50 border-red-200',
    [EPSGrowthTrend.Unknown]: 'text-gray-600 bg-gray-50 border-gray-200',
  };

  return (
    <div className="touch-manipulation rounded-xl border border-gray-200 bg-white p-4 sm:p-6 transition-all duration-200 hover:shadow-lg hover:border-blue-200">
      {/* Mobile-Responsive Header */}
      <div className="mb-4 sm:mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
        <div className="flex items-center gap-3 sm:gap-4">
          {/* Responsive rank indicator */}
          <div className="flex h-10 w-10 sm:h-12 sm:w-12 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-blue-600 shadow-md">
            <span className="text-xs sm:text-sm font-bold text-white">
              #{cardData.rank}
            </span>
          </div>
          
          {/* Symbol and date with mobile optimization */}
          <div className="flex-1">
            <h3 className="text-lg sm:text-xl font-semibold text-gray-900 mb-1">
              {cardData.symbol}
            </h3>
            <p className="text-xs sm:text-sm text-gray-500">
              {cardData.latest_date}
            </p>
          </div>
        </div>

        {/* Mobile-optimized status indicator */}
        <div
          className={`self-start sm:self-auto rounded-full px-3 sm:px-4 py-1.5 sm:py-2 font-medium text-xs sm:text-sm shadow-sm ${
            cardData.active_status === 'Active'
              ? 'bg-green-100 text-green-700 border border-green-200'
              : 'bg-red-100 text-red-700 border border-red-200'
          }`}
        >
          {cardData.active_status}
        </div>
      </div>

      {/* Quarter Comparison - Chronological Layout */}
      {quarters.length >= 2 ? (
        <div className="mb-6 space-y-4">
          {/* Quarter Headers - Previous (Left) to Latest (Right) */}
          <div className="grid grid-cols-2 gap-4 sm:gap-6 text-center">
            <div className="text-sm font-semibold text-gray-700">
              {previousQuarter?.quarter || 'Previous'}
            </div>
            <div className="text-sm font-semibold text-gray-700">
              {latestQuarter?.quarter || 'Latest'}
            </div>
          </div>

          {/* EPS Growth Row - Mobile Optimized */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
            <div className="text-center p-4 rounded-lg bg-gray-50 sm:bg-transparent sm:p-0">
              <p className="mb-2 text-xs font-medium text-gray-600 uppercase tracking-wide">
                EPS Growth
              </p>
              <p
                className={`mb-1 text-2xl sm:text-3xl font-bold ${
                  previousGrowth >= 0 ? 'text-green-600' : 'text-red-600'
                }`}
              >
                {formatPercentage(previousGrowth)}
              </p>
              <div className="text-sm text-gray-600">
                <span className="font-medium">{previousEPS.toFixed(2)}</span>
                <span className="ml-1 text-gray-400">EPS</span>
              </div>
            </div>

            <div className="text-center p-4 rounded-lg bg-gray-50 sm:bg-transparent sm:p-0">
              <p className="mb-2 text-xs font-medium text-gray-600 uppercase tracking-wide">
                EPS Growth
              </p>
              <p
                className={`mb-1 text-2xl sm:text-3xl font-bold ${
                  latestGrowth >= 0 ? 'text-green-600' : 'text-red-600'
                }`}
              >
                {formatPercentage(latestGrowth)}
              </p>
              <div className="text-sm text-gray-600">
                <span className="font-medium">{latestEPS.toFixed(2)}</span>
                <span className="ml-1 text-gray-400">EPS</span>
              </div>
            </div>
          </div>

          {/* Price Growth Row - Mobile Optimized */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 pt-4 border-t border-gray-100">
            <div className="text-center p-3 rounded-lg bg-blue-50 sm:bg-transparent sm:p-0">
              <p className="mb-2 text-xs font-medium text-gray-600 uppercase tracking-wide">
                Price Growth
              </p>
              <p
                className={`mb-1 text-lg sm:text-xl font-bold ${
                  (previousQuarter?.price_growth || 0) >= 0 ? 'text-green-600' : 'text-red-600'
                }`}
              >
                {formatPercentage(previousQuarter?.price_growth || 0)}
              </p>
              <div className="text-sm text-gray-600">
                <span className="font-medium">${(previousQuarter?.price || 0).toFixed(2)}</span>
              </div>
            </div>

            <div className="text-center p-3 rounded-lg bg-blue-50 sm:bg-transparent sm:p-0">
              <p className="mb-2 text-xs font-medium text-gray-600 uppercase tracking-wide">
                Price Growth
              </p>
              {/* Show previous quarter's price growth if latest is 0.0, otherwise show latest */}
              {(() => {
                const latestPriceGrowth = latestQuarter?.price_growth || 0;
                const previousPriceGrowth = previousQuarter?.price_growth || 0;
                const displayGrowth = latestPriceGrowth === 0 ? previousPriceGrowth : latestPriceGrowth;
                
                return (
                  <p
                    className={`mb-1 text-lg sm:text-xl font-bold ${
                      displayGrowth >= 0 ? 'text-green-600' : 'text-red-600'
                    }`}
                  >
                    {formatPercentage(displayGrowth)}
                  </p>
                );
              })()}
              <div className="text-sm text-gray-600">
                <span className="font-medium">${(latestQuarter?.price || 0).toFixed(2)}</span>
              </div>
            </div>
          </div>
        </div>
      ) : (
        /* Fallback layout for single quarter data */
        <div className="mb-6 rounded-lg bg-gray-50 p-4">
          <div className="text-center">
            <p className="mb-2 text-xs text-gray-500">Current Price</p>
            <p className="text-2xl font-bold text-gray-900">
              ${cardData.value.toFixed(2)}
            </p>
            <p className="mt-1 text-xs text-gray-500">{cardData.latest_date}</p>
          </div>
        </div>
      )}

      {/* Simplified Footer */}
      <div className="space-y-3 pt-4 border-t border-gray-100">
        <div className="flex items-center justify-between">
          <span className="text-sm text-gray-600">Current Price:</span>
          <span className="text-lg font-bold text-gray-900">
            ${cardData.value.toFixed(2)}
          </span>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-sm text-gray-600">Status:</span>
          <span
            className={`rounded-full px-3 py-1 text-sm font-medium ${
              cardData.active_status === 'Active'
                ? 'bg-green-100 text-green-700'
                : 'bg-gray-100 text-gray-700'
            }`}
          >
            {cardData.active_status}
          </span>
        </div>
      </div>

      {/* Action button - enhanced accessibility */}
      <a
        href={`https://www.tradingview.com/symbols/${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
        target="_blank"
        rel="noopener noreferrer"
        className="mt-6 inline-flex items-center justify-center min-h-[48px] w-full touch-manipulation rounded-lg bg-gradient-to-r from-blue-500 to-blue-600 py-3 px-4 text-center text-sm font-medium text-white shadow-md transition-all duration-200 hover:from-blue-600 hover:to-blue-700 hover:shadow-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
      >
        📊 View EPS Analysis
      </a>
    </div>
  );
});

StockCard.displayName = 'StockCard';

export default StockCard;
