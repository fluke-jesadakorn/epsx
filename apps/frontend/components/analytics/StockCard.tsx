'use client';

import { memo } from 'react';
import type { StockCardProps } from '@/types/analytics';
import { getGrowthTrend, formatPercentage, EPSGrowthTrend } from '@/types/analytics';

const StockCard = memo<StockCardProps>(({ ranking, rank }) => {
  // Get the 2 most recent quarters from quarterly_data
  const quarters = ranking.quarterly_data?.slice(0, 2) || [];
  const latestQuarter = quarters[0];
  const previousQuarter = quarters[1];
  
  // Use quarterly data if available, otherwise fall back to overall metrics
  const primaryGrowth = latestQuarter?.eps_growth ?? ranking.qoq_growth ?? 0;
  const growthTrend = getGrowthTrend(primaryGrowth);
  const isPositiveGrowth = primaryGrowth >= 0;
  
  const trendColors = {
    [EPSGrowthTrend.Accelerating]: 'text-green-600 bg-green-50 border-green-200',
    [EPSGrowthTrend.Steady]: 'text-blue-600 bg-blue-50 border-blue-200',
    [EPSGrowthTrend.Decelerating]: 'text-orange-600 bg-orange-50 border-orange-200',
    [EPSGrowthTrend.Volatile]: 'text-red-600 bg-red-50 border-red-200',
    [EPSGrowthTrend.Unknown]: 'text-gray-600 bg-gray-50 border-gray-200',
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4 hover:shadow-md transition-all duration-200 touch-manipulation">
      {/* Header with rank and symbol */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-gradient-to-br from-orange-500 to-yellow-500 rounded-full flex items-center justify-center">
            <span className="text-white text-sm font-bold">#{rank}</span>
          </div>
          <div>
            <h3 className="font-semibold text-lg text-gray-900">{ranking.symbol}</h3>
            <p className="text-sm text-gray-600 truncate max-w-[150px]">{ranking.name}</p>
          </div>
        </div>
        
        {/* Growth indicator */}
        <div className={`px-2 py-1 rounded-full text-xs font-medium border ${trendColors[growthTrend]}`}>
          {growthTrend}
        </div>
      </div>

      {/* Quarter comparison - mobile optimized */}
      {quarters.length >= 2 ? (
        <div className="space-y-3 mb-3">
          {/* Quarter headers */}
          <div className="grid grid-cols-2 gap-3 text-xs font-medium text-gray-600">
            <div className="text-center">{latestQuarter.quarter}</div>
            <div className="text-center">{previousQuarter.quarter}</div>
          </div>
          
          {/* EPS Growth Comparison */}
          <div className="grid grid-cols-2 gap-3">
            <div className="bg-gray-50 rounded-lg p-3 text-center">
              <p className="text-xs text-gray-500 mb-1">EPS Growth</p>
              <p className={`font-semibold text-sm ${latestQuarter.eps_growth >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                {formatPercentage(latestQuarter.eps_growth)}
              </p>
              <p className="text-xs text-gray-600 mt-1">${latestQuarter.eps.toFixed(2)}</p>
            </div>
            <div className="bg-gray-50 rounded-lg p-3 text-center">
              <p className="text-xs text-gray-500 mb-1">EPS Growth</p>
              <p className={`font-semibold text-sm ${previousQuarter.eps_growth >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                {formatPercentage(previousQuarter.eps_growth)}
              </p>
              <p className="text-xs text-gray-600 mt-1">${previousQuarter.eps.toFixed(2)}</p>
            </div>
          </div>
          
          {/* Price Growth Comparison */}
          <div className="grid grid-cols-2 gap-3">
            <div className="bg-blue-50 rounded-lg p-3 text-center">
              <p className="text-xs text-gray-500 mb-1">Price Growth</p>
              <p className={`font-semibold text-sm ${latestQuarter.price_growth >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                {formatPercentage(latestQuarter.price_growth)}
              </p>
              <p className="text-xs text-gray-600 mt-1">${latestQuarter.price.toFixed(2)}</p>
            </div>
            <div className="bg-blue-50 rounded-lg p-3 text-center">
              <p className="text-xs text-gray-500 mb-1">Price Growth</p>
              <p className={`font-semibold text-sm ${previousQuarter.price_growth >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                {formatPercentage(previousQuarter.price_growth)}
              </p>
              <p className="text-xs text-gray-600 mt-1">${previousQuarter.price.toFixed(2)}</p>
            </div>
          </div>
        </div>
      ) : (
        /* Fallback to original layout if less than 2 quarters */
        <div className="grid grid-cols-2 gap-3 mb-3">
          <div className="bg-gray-50 rounded-lg p-3">
            <p className="text-xs text-gray-500 mb-1">Price</p>
            <p className="font-semibold text-gray-900">
              ${ranking.price_current?.toFixed(2) ?? 'N/A'}
            </p>
          </div>
          <div className="bg-gray-50 rounded-lg p-3">
            <p className="text-xs text-gray-500 mb-1">EPS Growth</p>
            <p className={`font-semibold ${isPositiveGrowth ? 'text-green-600' : 'text-red-600'}`}>
              {formatPercentage(ranking.qoq_growth)}
            </p>
          </div>
        </div>
      )}

      {/* Secondary metrics - stack on mobile */}
      <div className="space-y-2 text-sm">
        {quarters.length >= 2 ? (
          <>
            <div className="flex justify-between items-center">
              <span className="text-gray-500">Current Price:</span>
              <span className="font-medium">${ranking.price_current?.toFixed(2) ?? latestQuarter?.price.toFixed(2) ?? 'N/A'}</span>
            </div>
            
            <div className="flex justify-between items-center">
              <span className="text-gray-500">Status:</span>
              <span className={`font-medium px-2 py-1 rounded-full text-xs ${
                ranking.active_status === 'Active' 
                  ? 'bg-green-100 text-green-700' 
                  : 'bg-red-100 text-red-700'
              }`}>
                {ranking.active_status === 'Active' ? 'Active' : 'Non Active'}
              </span>
            </div>
          </>
        ) : (
          <>
            <div className="flex justify-between items-center">
              <span className="text-gray-500">Current EPS:</span>
              <span className="font-medium">${ranking.current_eps?.toFixed(2) ?? 'N/A'}</span>
            </div>
            
            <div className="flex justify-between items-center">
              <span className="text-gray-500">Status:</span>
              <span className={`font-medium px-2 py-1 rounded-full text-xs ${
                ranking.active_status === 'Active' 
                  ? 'bg-green-100 text-green-700' 
                  : 'bg-red-100 text-red-700'
              }`}>
                {ranking.active_status === 'Active' ? 'Active' : 'Non Active'}
              </span>
            </div>
          </>
        )}
        
        <div className="flex justify-between items-center">
          <span className="text-gray-500">Sector:</span>
          <span className="font-medium text-blue-600 truncate max-w-[120px]">{ranking.sector}</span>
        </div>
        
        <div className="flex justify-between items-center">
          <span className="text-gray-500">Country:</span>
          <span className="font-medium">{ranking.country}</span>
        </div>
      </div>

      {/* Action button - touch optimized */}
      <button className="w-full mt-4 bg-gradient-to-r from-orange-500 to-yellow-500 text-white py-2.5 rounded-lg font-medium text-sm hover:from-orange-600 hover:to-yellow-600 transition-all duration-200 min-h-[44px] touch-manipulation">
        View Details
      </button>
    </div>
  );
});

StockCard.displayName = 'StockCard';

export default StockCard;