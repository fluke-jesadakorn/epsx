'use client';

import React from 'react';

import { Card, CardContent } from '@/components/ui/card';

import { Button } from '../ui/button';

import type { StockFinancialData } from '@/types/financialChartData';
import { 
  getLatestQuarterData, 
  calculateAverageEpsGrowth, 
  formatPrice, 
  formatEpsGrowth, 
  formatDate 
} from '@/utils/transformers/stockDataTransformer';

interface FinancialDataTableProps {
  style?: React.CSSProperties;
  className?: string;
  data: StockFinancialData[];
}

interface FinancialDataCardProps {
  data: StockFinancialData;
  index: number;
}

function FinancialDataCard({ data, index }: FinancialDataCardProps): React.JSX.Element {
  const [expanded, setExpanded] = React.useState(false);
  const [isPressed, setIsPressed] = React.useState(false);
  const latestQuarter = getLatestQuarterData(data);
  const avgGrowth = calculateAverageEpsGrowth(data);

  return (
    <Card
      className={`w-full transition-all duration-200 hover:shadow-2xl border-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-[#232946] dark:via-[#1a1a2e] dark:to-[#0f1021] rounded-3xl shadow-lg relative ${
        isPressed ? 'scale-[0.98] opacity-90' : ''
      }`}
      onTouchStart={() => setIsPressed(true)}
      onTouchEnd={() => setIsPressed(false)}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseLeave={() => setIsPressed(false)}
    >
      {/* Rank Number Badge */}
      <div className="absolute -top-3 -left-3 w-10 h-10 rounded-full bg-gradient-to-br from-yellow-400 via-pink-400 to-purple-500 dark:from-yellow-600 dark:via-pink-700 dark:to-purple-800 flex items-center justify-center text-white text-base font-extrabold shadow-xl border-4 border-white dark:border-[#232946]">
        {index + 1}
      </div>
      <CardContent className="p-6 pt-8">
        {/* Unified Data Fields (match table) */}
        <div className="grid grid-cols-2 gap-2 mb-3">
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Symbol</div>
            <div className="flex items-center gap-2">
              <div className="text-xl font-extrabold text-primary dark:text-white drop-shadow-sm tracking-wide">
                {data.symbol}
              </div>
              {data.quarters.length >= 2 && data.quarters[0].eps_growth !== undefined && data.quarters[1].eps_growth !== undefined && (
                <span className={`text-lg ${
                  data.quarters[0].eps_growth > data.quarters[1].eps_growth 
                    ? 'text-emerald-500' 
                    : 'text-rose-500'
                }`}>
                  {data.quarters[0].eps_growth > data.quarters[1].eps_growth ? '↗' : '↘'}
                </span>
              )}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Latest Date</div>
            <div className="text-sm font-medium">
              {latestQuarter?.date ? formatDate(latestQuarter.date) : 'N/A'}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Latest Price</div>
            <div className="font-bold text-lg text-blue-600 dark:text-blue-300 drop-shadow">
              {(() => {
                // Use current price if available, otherwise fall back to latest quarter price
                const priceToShow = data.currentPrice !== undefined && data.currentPrice !== null 
                  ? data.currentPrice 
                  : latestQuarter?.price;
                return priceToShow !== undefined && priceToShow !== null ? formatPrice(priceToShow) : 'N/A';
              })()}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Latest EPS</div>
            <div className="font-bold">
              {latestQuarter?.eps !== undefined ? latestQuarter.eps.toFixed(4) : 'N/A'}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">EPS Growth %</div>
            <div
              className={`font-bold ${
                (latestQuarter?.eps_growth || 0) >= 0
                  ? 'text-green-500'
                  : 'text-rose-400 dark:text-rose-300'
              }`}
            >
              {latestQuarter?.eps_growth !== undefined ? formatEpsGrowth(latestQuarter.eps_growth) : 'N/A'}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Avg Growth %</div>
            <div
              className={`font-bold ${(avgGrowth || 0) >= 0 ? 'text-green-500' : 'text-rose-500'}`}
            >
              {formatEpsGrowth(avgGrowth)}
            </div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground font-semibold">Quarters</div>
            <div className="font-bold">{data.quarters.length}</div>
          </div>
        </div>

        {/* Action Buttons Row */}
        <div className="flex justify-between items-center gap-2 mt-3">
          <Button
            size="sm"
            variant="ghost"
            className="w-full rounded-full bg-gradient-to-r from-blue-100 to-purple-100 dark:from-[#232946] dark:to-[#1a1a2e] text-primary dark:text-white font-bold shadow hover:scale-105 transition"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? 'Less' : 'More'}
          </Button>
          <Button
            asChild
            size="sm"
            variant="secondary"
            className="w-[100px] rounded-full bg-gradient-to-r from-yellow-300 to-pink-300 dark:from-yellow-700 dark:to-pink-700 text-white font-bold shadow hover:scale-105 transition"
          >
            <a
              href={`https://www.tradingview.com/chart?symbol=${data.symbol}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              Analytics
            </a>
          </Button>
        </div>

        {/* Expandable Content */}
        <div
          className={`mt-4 pt-4 border-t border-border/50 grid gap-3 text-xs sm:text-sm overflow-hidden transition-all duration-300 ${
            expanded ? 'opacity-100 max-h-[500px]' : 'opacity-0 max-h-0'
          }`}
        >
          <div className="grid grid-cols-2 gap-2">
            <div>
              <div className="text-muted-foreground font-semibold">Avg Growth</div>
              <div className="font-bold">{formatEpsGrowth(avgGrowth)}</div>
            </div>
            <div>
              <div className="text-muted-foreground font-semibold">
                Quarters
              </div>
              <div className="font-bold">{data.quarters.length}</div>
            </div>
          </div>

          <div className="space-y-3">
            <div className="text-muted-foreground font-semibold text-sm flex items-center gap-2">
              <span>📊 Quarterly Performance</span>
              <span className="text-xs bg-blue-100 dark:bg-blue-900/30 px-2 py-0.5 rounded-full">
                {data.quarters.length}Q
              </span>
            </div>
            
            {/* Modern Grid Header */}
            <div className="grid grid-cols-6 gap-1 text-[9px] font-bold text-muted-foreground/80 uppercase tracking-wide bg-gradient-to-r from-slate-100 to-slate-50 dark:from-slate-800/50 dark:to-slate-700/50 px-2 py-1.5 rounded-lg">
              <span className="text-center">Q</span>
              <span className="text-center">Date</span>
              <span className="text-center">Price</span>
              <span className="text-center">EPS</span>
              <span className="text-center">EPS %</span>
              <span className="text-center">Price %</span>
            </div>
            
            {/* Quarter Data with improved layout */}
            <div className="space-y-1 max-h-[200px] overflow-y-auto custom-scrollbar">
              {data.quarters.map((quarter, idx) => (
                <div key={idx} className="group hover:scale-[1.01] transition-all duration-150">
                  <div className="grid grid-cols-6 gap-1 text-xs bg-gradient-to-r from-white/70 to-slate-50/70 dark:from-slate-800/30 dark:to-slate-700/30 rounded-lg px-2 py-2 border border-slate-200/40 dark:border-slate-600/20 hover:shadow-sm hover:border-blue-300/40 dark:hover:border-blue-500/30 transition-all duration-150">
                    
                    {/* Quarter */}
                    <div className="flex items-center justify-center">
                      <span className="font-bold text-primary/80 bg-blue-100 dark:bg-blue-900/30 px-1.5 py-0.5 rounded-full text-[9px]">
                        Q{quarter.quarter}
                      </span>
                    </div>
                    
                    {/* Date */}
                    <div className="flex items-center justify-center">
                      <span className="font-medium text-slate-700 dark:text-slate-300 text-[9px] text-center">
                        {quarter?.date ? formatDate(quarter.date).slice(5) : 'N/A'}
                      </span>
                    </div>
                    
                    {/* Price */}
                    <div className="flex items-center justify-center">
                      <span className="font-bold text-blue-700 dark:text-blue-300 text-[9px] text-center">
                        {quarter?.price !== undefined && quarter?.price !== null ? 
                          (quarter.price < 1000 ? formatPrice(quarter.price) : `$${(quarter.price/1000).toFixed(1)}k`) 
                          : 'N/A'
                        }
                      </span>
                    </div>
                    
                    {/* EPS */}
                    <div className="flex items-center justify-center">
                      <span className="font-bold text-purple-700 dark:text-purple-300 text-[9px] text-center">
                        {quarter?.eps !== undefined ? quarter.eps.toFixed(2) : 'N/A'}
                      </span>
                    </div>
                    
                    {/* EPS Growth */}
                    <div className="flex items-center justify-center">
                      {quarter?.eps_growth !== undefined ? (
                        <div className={`px-1 py-0.5 rounded-full font-bold text-[8px] text-center min-w-[28px] ${
                          quarter.eps_growth >= 0 
                            ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300' 
                            : 'bg-rose-100 text-rose-700 dark:bg-rose-900/40 dark:text-rose-300'
                        }`}>
                          {quarter.eps_growth >= 0 ? '+' : ''}{quarter.eps_growth}%
                        </div>
                      ) : (
                        <span className="text-slate-400 text-[8px]">-</span>
                      )}
                    </div>
                    
                    {/* Price Growth */}
                    <div className="flex items-center justify-center">
                      {quarter?.price_growth !== undefined && quarter.price_growth !== null ? (
                        <div className={`px-1 py-0.5 rounded-full font-bold text-[8px] text-center min-w-[28px] ${
                          quarter.price_growth >= 0 
                            ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300' 
                            : 'bg-rose-100 text-rose-700 dark:bg-rose-900/40 dark:text-rose-300'
                        }`}>
                          {quarter.price_growth >= 0 ? '+' : ''}{quarter.price_growth}%
                        </div>
                      ) : (
                        <span className="text-slate-400 text-[8px]">-</span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
            
            {/* Quick Insights */}
            <div className="grid grid-cols-3 gap-2 mt-3 pt-2 border-t border-slate-200 dark:border-slate-600">
              <div className="text-center">
                <div className="text-[8px] text-muted-foreground font-medium">Best Growth</div>
                <div className="text-[10px] font-bold text-emerald-600 dark:text-emerald-400">
                  {Math.max(...data.quarters.filter(q => q.eps_growth !== undefined).map(q => q.eps_growth || 0), 0)}%
                </div>
              </div>
              <div className="text-center">
                <div className="text-[8px] text-muted-foreground font-medium">Trend</div>
                <div className="text-[10px] font-bold">
                  {data.quarters.length >= 2 && data.quarters[0].eps_growth !== undefined && data.quarters[1].eps_growth !== undefined ? (
                    <span className={data.quarters[0].eps_growth > data.quarters[1].eps_growth ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}>
                      {data.quarters[0].eps_growth > data.quarters[1].eps_growth ? '↗' : '↘'}
                    </span>
                  ) : '-'}
                </div>
              </div>
              <div className="text-center">
                <div className="text-[8px] text-muted-foreground font-medium">Quarters</div>
                <div className="text-[10px] font-bold text-blue-600 dark:text-blue-400">
                  {data.quarters.length}
                </div>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function FinancialDataTable({
  style,
  className,
  data,
}: FinancialDataTableProps): React.JSX.Element {
  // Ensure data is always an array to prevent runtime errors
  const safeData = Array.isArray(data) ? data : [];

  // Responsive card grid
  const renderCardView = () => (
    <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
      {safeData.map((item, index) => (
        <FinancialDataCard key={`${item.symbol}-${index}`} data={item} index={index} />
      ))}
    </div>
  );

  return (
    <div
      className={`w-full space-y-6 p-4 sm:p-8 ${className || ''}`}
      style={style}
    >
      {/* Title */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-8">
        <div>
          <h2 className="text-3xl font-extrabold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent drop-shadow">
            Financial Data Rankings
          </h2>
          <div className="w-24 h-1 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 rounded-full mt-2" />
        </div>
      </div>
      {/* Card content only */}
      <div className="-mx-4 sm:-mx-8 px-4 sm:px-8">
        {renderCardView()}
      </div>
    </div>
  );
}

export default FinancialDataTable;
