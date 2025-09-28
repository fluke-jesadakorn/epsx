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

    // Use backend's TRACK/STOP status directly
    let activeStatus = ranking.active_status || 'STOP';

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
      active_status: activeStatus,
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

  // Convert backend TRACK/STOP to frontend Active/Inactive display
  let displayStatus = cardData.active_status;
  if (displayStatus === 'TRACK') displayStatus = 'Active';
  if (displayStatus === 'STOP') displayStatus = 'Inactive';

  const isActive = displayStatus === 'Active';
  const isInactive = displayStatus === 'Inactive';

  // Calculate days left for next action (mock for now)
  const daysLeft = Math.floor(Math.random() * 90) + 1; // 1-90 days
  const progressPercentage = Math.max(10, Math.min(90, (90 - daysLeft) / 90 * 100));

  return (
    <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden touch-manipulation p-6">
      {/* Header Section */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <div className={`w-12 h-12 rounded-full flex items-center justify-center ${
            isActive 
              ? 'bg-green-400 text-purple-800' 
              : isInactive
              ? 'bg-red-400 text-white'
              : 'bg-gray-400 text-purple-800'
          }`}>
            <span className="font-bold text-lg">{cardData.rank}</span>
          </div>
          <h3 className="font-bold text-3xl text-white">{cardData.symbol}</h3>
        </div>
        <button className={`px-4 py-2 rounded-full font-bold text-sm flex items-center gap-2 ${
          isActive 
            ? 'bg-green-400 text-purple-800 hover:bg-green-300' 
            : isInactive
            ? 'bg-red-400 text-white hover:bg-red-300'
            : 'bg-gray-400 text-purple-800 hover:bg-gray-300'
        }`}>
          <span className={`w-2 h-2 rounded-full ${
            isActive ? 'bg-purple-800' : 'bg-current'
          }`}></span>
          View 🔗
        </button>
      </div>

      {/* Status Button */}
      <div className="mb-6 flex justify-center">
        <button className={`px-8 py-3 rounded-full font-bold text-lg ${
          isActive 
            ? 'bg-green-400 text-purple-800 hover:bg-green-300' 
            : isInactive
            ? 'bg-red-400 text-white hover:bg-red-300'
            : 'bg-gray-400 text-purple-800 hover:bg-gray-300'
        }`}>
          ● {displayStatus.toUpperCase()}
        </button>
      </div>

      {/* Next Action Progress */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <span className="text-white font-medium">Next Action</span>
          <span className="text-white font-medium">{daysLeft}d left</span>
        </div>
        <div className="w-full bg-purple-500/50 rounded-full h-3">
          <div 
            className="bg-green-400 h-3 rounded-full"
            style={{ width: `${progressPercentage}%` }}
          ></div>
        </div>
      </div>

      {/* Growth and Price Section */}
      <div className="grid grid-cols-2 gap-4">
        {/* Growth */}
        <div className={`rounded-2xl p-4 text-center ${
          latestGrowth >= 0 
            ? 'bg-green-400 text-purple-800' 
            : 'bg-red-500 text-white'
        }`}>
          <div className="font-bold text-sm mb-1">Growth</div>
          <div className="font-bold text-xl">
            {latestGrowth >= 0 ? '+' : ''}{formatPercentage(latestGrowth)}
          </div>
        </div>
        
        {/* Price */}
        <div className="text-center flex flex-col justify-center">
          <div className="text-white/80 font-medium text-sm mb-1">Price</div>
          <div className="text-white font-bold text-xl">
            {new Intl.NumberFormat('en-US', {
              style: 'currency',
              currency: cardData.currency || 'USD',
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            }).format(cardData.value)}
          </div>
        </div>
      </div>
    </div>
  );
});

StockCard.displayName = 'StockCard';

export default StockCard;
