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
    latestGrowth = latestQuarter?.eps_growth || ranking.growth_factor || 0;
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
        eps_growth: q.eps_growth || ranking.growth_factor || 0,
        price_growth: q.price_growth || 0,
      })),
    };
  }

  return (
    <div className="w-full max-w-sm mx-auto bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden touch-manipulation">
      {/* Row 1: Header with symbol and status */}
      <div className="px-4 sm:px-6 py-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 rounded-full bg-blue-500 flex items-center justify-center flex-shrink-0">
            <span className="text-white font-bold text-sm">#{cardData.rank}</span>
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="font-bold text-xl text-gray-900 truncate">{cardData.symbol}</h3>
            <p className="text-sm text-gray-500 truncate">{cardData.latest_date}</p>
          </div>
        </div>
        <div className={`px-3 py-1 rounded-full text-sm font-medium self-start sm:self-auto ${
          cardData.active_status === 'Active' 
            ? 'bg-green-100 text-green-700' 
            : 'bg-gray-100 text-gray-700'
        }`}>
          {cardData.active_status}
        </div>
      </div>

      {quarters.length >= 2 ? (
        <>
          {/* Row 2: Quarter headers */}
          <div className="px-4 sm:px-6 py-2">
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center">
                <div className="text-sm font-medium text-gray-700">
                  {previousQuarter?.quarter || '2025-Q2'}
                </div>
              </div>
              <div className="text-center">
                <div className="text-sm font-medium text-gray-700">
                  {latestQuarter?.quarter || '2025-Q3'}
                </div>
              </div>
            </div>
          </div>

          {/* Row 3: EPS Growth label */}
          <div className="px-4 sm:px-6 py-1">
            <div className="text-xs font-medium text-gray-600 uppercase tracking-wide">
              EPS GROWTH
            </div>
          </div>

          {/* Row 4: EPS Growth percentages */}
          <div className="px-4 sm:px-6 py-2">
            <div className="grid grid-cols-2 gap-4 min-h-[48px] items-center">
              <div className="text-center">
                <div className={`text-xl sm:text-2xl font-bold ${
                  previousGrowth >= 0 ? 'text-green-600' : 'text-red-600'
                }`}>
                  {formatPercentage(previousGrowth)}
                </div>
              </div>
              <div className="text-center">
                <div className={`text-xl sm:text-2xl font-bold ${
                  latestGrowth >= 0 ? 'text-green-600' : 'text-red-600'
                }`}>
                  {formatPercentage(latestGrowth)}
                </div>
              </div>
            </div>
          </div>

          {/* Row 5: EPS values */}
          <div className="px-4 sm:px-6 py-2">
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center">
                <div className="text-sm text-gray-600">
                  <span className="font-medium">{previousEPS.toFixed(2)}</span>
                  <span className="ml-1 text-gray-400">EPS</span>
                </div>
              </div>
              <div className="text-center">
                <div className="text-sm text-gray-600">
                  <span className="font-medium">{latestEPS.toFixed(2)}</span>
                  <span className="ml-1 text-gray-400">EPS</span>
                </div>
              </div>
            </div>
          </div>

          {/* Row 6: Price Growth label */}
          <div className="px-4 sm:px-6 py-1 pt-4 border-t border-gray-100">
            <div className="text-xs font-medium text-gray-600 uppercase tracking-wide">
              PRICE GROWTH
            </div>
          </div>

          {/* Row 7: Price Growth percentages */}
          <div className="px-4 sm:px-6 py-2">
            <div className="grid grid-cols-2 gap-4 min-h-[44px] items-center">
              <div className="text-center">
                <div className={`text-lg sm:text-xl font-bold ${
                  (previousQuarter?.price_growth || 0) >= 0 ? 'text-green-600' : 'text-red-600'
                }`}>
                  {formatPercentage(previousQuarter?.price_growth || 0)}
                </div>
              </div>
              <div className="text-center">
                <div className={`text-lg sm:text-xl font-bold ${
                  (latestQuarter?.price_growth || 0) >= 0 ? 'text-green-600' : 'text-red-600'
                }`}>
                  {formatPercentage(latestQuarter?.price_growth || 0)}
                </div>
              </div>
            </div>
          </div>

          {/* Row 8: Price values */}
          <div className="px-4 sm:px-6 py-2">
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center">
                <div className="text-sm font-medium text-gray-600">
                  ${(previousQuarter?.price || 0).toFixed(2)}
                </div>
              </div>
              <div className="text-center">
                <div className="text-sm font-medium text-gray-600">
                  ${(latestQuarter?.price || 0).toFixed(2)}
                </div>
              </div>
            </div>
          </div>
        </>
      ) : (
        /* Fallback for single quarter */
        <div className="px-4 sm:px-6 py-4">
          <div className="text-center">
            <p className="text-sm text-gray-500 mb-2">Current Price</p>
            <p className="text-2xl font-bold text-gray-900">${cardData.value.toFixed(2)}</p>
          </div>
        </div>
      )}

      {/* Row 9: Current Price */}
      <div className="px-4 sm:px-6 py-4 border-t border-gray-100">
        <div className="flex items-center justify-between min-h-[44px]">
          <span className="text-sm text-gray-600">Current Price:</span>
          <span className="text-xl sm:text-2xl font-bold text-gray-900">
            ${cardData.value.toFixed(2)}
          </span>
        </div>
      </div>

      {/* Row 10: Status */}
      <div className="px-4 sm:px-6 py-3">
        <div className="flex items-center justify-between min-h-[44px]">
          <span className="text-sm text-gray-600">Status:</span>
          <span className={`px-3 py-2 rounded-full text-sm font-medium ${
            cardData.active_status === 'Active'
              ? 'bg-green-100 text-green-700'
              : 'bg-gray-100 text-gray-700'
          }`}>
            {cardData.active_status}
          </span>
        </div>
      </div>

      {/* Row 11: Action button */}
      <div className="px-4 sm:px-6 py-4">
        <a
          href={`https://www.tradingview.com/symbols/${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
          target="_blank"
          rel="noopener noreferrer"
          className="w-full min-h-[48px] flex items-center justify-center bg-blue-500 text-white rounded-lg font-medium text-sm py-3 px-4 transition-colors duration-200 hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 touch-manipulation"
        >
          📊 View EPS Analysis
        </a>
      </div>
    </div>
  );
});

StockCard.displayName = 'StockCard';

export default StockCard;
