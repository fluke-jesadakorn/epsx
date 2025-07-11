'use client';

import React from 'react';

import { Card, CardContent } from '@/components/ui/card';

import type { StockFinancialData } from '@/types/financialChartData';
import {
  getLatestQuarterData,
  calculateAverageEpsGrowth,
  formatPrice,
  formatEpsGrowth,
  formatDate,
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

function FinancialDataCard({
  data,
  index,
}: FinancialDataCardProps): React.JSX.Element {
  const [isPressed, setIsPressed] = React.useState(false);
  const [isHovered, setIsHovered] = React.useState(false);
  const latestQuarter = getLatestQuarterData(data);
  const avgGrowth = calculateAverageEpsGrowth(data);

  return (
    <Card
      className={`group w-full transition-all duration-300 ease-out hover:scale-[1.02] hover:shadow-2xl hover:shadow-purple-500/20 dark:hover:shadow-purple-900/30 border-0 bg-gradient-to-br from-white via-blue-50/30 to-purple-50/50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900 rounded-2xl shadow-lg hover:bg-gradient-to-br hover:from-white hover:via-blue-50 hover:to-purple-100 dark:hover:from-slate-800 dark:hover:via-slate-700 dark:hover:to-slate-800 relative overflow-hidden backdrop-blur-sm cursor-pointer focus:outline-none focus:ring-4 focus:ring-purple-500/20 ${
        isPressed ? 'scale-[0.98] opacity-90' : ''
      }`}
      tabIndex={0}
      role="button"
      aria-label={`Financial data for ${data.symbol}`}
      onTouchStart={() => setIsPressed(true)}
      onTouchEnd={() => setIsPressed(false)}
      onMouseDown={() => setIsPressed(true)}
      onMouseUp={() => setIsPressed(false)}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => {
        setIsPressed(false);
        setIsHovered(false);
      }}
      onFocus={() => setIsHovered(true)}
      onBlur={() => setIsHovered(false)}
    >
      {/* Animated Background Gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-blue-500/5 via-purple-500/5 to-pink-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />

      {/* Rank Number Badge - Better positioned */}
      <div
        className={`absolute top-4 left-4 w-10 h-10 rounded-full bg-gradient-to-br from-purple-600 to-blue-500 flex items-center justify-center text-white text-lg font-extrabold shadow-lg border-2 border-white dark:border-slate-900 z-20 transition-all duration-300 ${isHovered ? 'scale-110 shadow-xl' : ''}`}
        aria-label={`Rank ${index + 1}`}
      >
        <span className="drop-shadow-lg">{index + 1}</span>
      </div>

      {/* Performance indicator glow */}
      {avgGrowth && avgGrowth > 0 && (
        <div className="absolute top-0 right-0 w-20 h-20 bg-green-400/20 rounded-full blur-xl opacity-50 group-hover:opacity-75 transition-opacity duration-300" />
      )}

      {/* Subtle animated border */}
      <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-blue-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300 blur-sm" />

      {/* Header Section with improved layout */}
      <CardContent className="relative p-5 pt-6 z-10">
        <div className="flex items-start justify-between mb-5">
          <div className="flex items-center gap-3 ml-12">
            {' '}
            {/* Add left margin to avoid badge overlap */}
            <div className="text-2xl font-black text-slate-800 dark:text-white tracking-tight">
              {data.symbol}
            </div>
            {data.quarters.length >= 2 &&
              data.quarters[0].eps_growth !== undefined &&
              data.quarters[1].eps_growth !== undefined && (
                <div className="flex items-center gap-1">
                  <div
                    className={`w-7 h-7 rounded-full flex items-center justify-center ${
                      data.quarters[0].eps_growth > data.quarters[1].eps_growth
                        ? 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400'
                        : 'bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400'
                    } transition-all duration-300 group-hover:scale-110`}
                  >
                    <span className="text-sm font-bold">
                      {data.quarters[0].eps_growth > data.quarters[1].eps_growth
                        ? '↗'
                        : '↘'}
                    </span>
                  </div>
                </div>
              )}
          </div>
          <div className="text-right">
            <div className="text-xs text-slate-500 dark:text-slate-400 font-medium uppercase tracking-wider">
              Latest Date
            </div>
            <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
              {latestQuarter?.date ? formatDate(latestQuarter.date) : 'N/A'}
            </div>
          </div>
        </div>

        {/* Key Metrics Grid - Improved Balance */}
        <div className="grid grid-cols-3 gap-4 mb-6">
          {/* Latest Price */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-blue-50 to-blue-100/50 dark:from-blue-900/20 dark:to-blue-800/20 border border-blue-200/50 dark:border-blue-700/30">
            <div className="text-xs text-blue-600 dark:text-blue-400 font-semibold tracking-wide uppercase mb-2">
              Latest Price
            </div>
            <div className="font-bold text-lg text-blue-700 dark:text-blue-300">
              {(() => {
                const priceToShow =
                  data.currentPrice !== undefined && data.currentPrice !== null
                    ? data.currentPrice
                    : latestQuarter?.price;
                return priceToShow !== undefined && priceToShow !== null
                  ? formatPrice(priceToShow)
                  : 'N/A';
              })()}
            </div>
          </div>

          {/* Latest EPS */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-purple-50 to-purple-100/50 dark:from-purple-900/20 dark:to-purple-800/20 border border-purple-200/50 dark:border-purple-700/30">
            <div className="text-xs text-purple-600 dark:text-purple-400 font-semibold tracking-wide uppercase mb-2">
              Latest EPS
            </div>
            <div className="font-bold text-lg text-purple-700 dark:text-purple-300">
              {latestQuarter?.eps !== undefined
                ? latestQuarter.eps.toFixed(4)
                : 'N/A'}
            </div>
          </div>

          {/* Avg Growth */}
          <div className="p-4 rounded-xl bg-gradient-to-br from-emerald-50 to-emerald-100/50 dark:from-emerald-900/20 dark:to-emerald-800/20 border border-emerald-200/50 dark:border-emerald-700/30">
            <div className="text-xs text-emerald-600 dark:text-emerald-400 font-semibold tracking-wide uppercase mb-2">
              Avg Growth
            </div>
            <div
              className={`font-bold text-lg flex items-center gap-2 ${(avgGrowth || 0) >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}`}
            >
              <div
                className={`w-5 h-5 rounded-full flex items-center justify-center ${(avgGrowth || 0) >= 0 ? 'bg-emerald-100 dark:bg-emerald-900/50' : 'bg-red-100 dark:bg-red-900/50'}`}
              >
                {(avgGrowth || 0) >= 0 ? (
                  <span className="text-xs">▲</span>
                ) : (
                  <span className="text-xs">▼</span>
                )}
              </div>
              {formatEpsGrowth(avgGrowth)}
            </div>
          </div>
        </div>

        <div className="mb-6">
          <a
            href={`https://www.tradingview.com/chart?symbol=${data.symbol}`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-center gap-2 w-full p-3 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 text-white font-semibold shadow-md hover:from-purple-600 hover:to-blue-500 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-blue-400/60 focus:ring-offset-2 dark:focus:ring-blue-700/60 hover:shadow-lg hover:scale-[1.02]"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
              className="w-4 h-4"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6A2.25 2.25 0 005.25 5.25v13.5A2.25 2.25 0 007.5 21h9a2.25 2.25 0 002.25-2.25V12.75M12 15.75l9-9m0 0h-5.25m5.25 0V9"
              />
            </svg>
            <span>View Chart on TradingView</span>
          </a>
        </div>

        {/* Quarterly Performance Section - Improved Balance */}
        <div className="space-y-3">
          <div className="flex items-center gap-3 pb-2 border-b border-slate-200/70 dark:border-slate-700/70">
            <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center">
              <span className="text-white text-xs font-bold">📊</span>
            </div>
            <h3 className="text-base font-bold text-slate-800 dark:text-white">
              Quarterly Performance
            </h3>
            <div className="ml-auto">
              <span className="text-xs bg-gradient-to-r from-indigo-500 to-purple-600 text-white px-2 py-1 rounded-full font-medium shadow-sm">
                {data.quarters.length - 1}Q
              </span>
            </div>
          </div>

          {/* Compact Table header */}
          <div className="hidden sm:grid grid-cols-5 gap-3 px-3 py-2 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
            <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wide">
              Quarter
            </div>
            <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wide text-right">
              Price
            </div>
            <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wide text-right">
              EPS
            </div>
            <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wide text-right">
              EPS %
            </div>
            <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wide text-right">
              Price %
            </div>
          </div>

          <div className="space-y-2">
            {data.quarters
              .filter((quarter, idx) => {
                if (
                  idx === 0 &&
                  (quarter.eps_growth === undefined ||
                    quarter.eps_growth === null) &&
                  (quarter.price_growth === undefined ||
                    quarter.price_growth === null)
                ) {
                  return false;
                }
                return true;
              })
              .map((quarter, idx) => (
                <div
                  key={idx}
                  className="group/row relative flex flex-wrap sm:grid sm:grid-cols-5 items-center gap-3 p-3 rounded-lg border border-slate-200/50 dark:border-slate-700/50 bg-gradient-to-r from-white to-slate-50/50 dark:from-slate-800/30 dark:to-slate-700/30 hover:shadow-md hover:border-slate-300/50 dark:hover:border-slate-600/50 transition-all duration-200 hover:bg-gradient-to-r hover:from-slate-50 hover:to-white dark:hover:from-slate-700/50 dark:hover:to-slate-800/50"
                >
                  {/* Quarter info - More compact */}
                  <div className="flex flex-col items-start min-w-[70px]">
                    <div className="flex items-center gap-2">
                      <span className="text-base font-black text-slate-800 dark:text-white">
                        Q{quarter.quarter}
                      </span>
                      <div className="w-1.5 h-1.5 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 group-hover/row:scale-125 transition-transform duration-200" />
                    </div>
                    <span className="text-xs text-slate-500 dark:text-slate-400 font-medium">
                      {quarter?.date ? formatDate(quarter.date) : 'N/A'}
                    </span>
                  </div>

                  {/* Price - More compact */}
                  <div className="flex flex-col items-end sm:items-end min-w-[80px]">
                    <span className="text-sm font-bold text-blue-600 dark:text-blue-400">
                      {quarter?.price !== undefined && quarter?.price !== null
                        ? formatPrice(quarter.price)
                        : 'N/A'}
                    </span>
                    <span className="text-xs text-slate-400 sm:hidden">
                      Price
                    </span>
                  </div>

                  {/* EPS - More compact */}
                  <div className="flex flex-col items-end sm:items-end min-w-[60px]">
                    <span className="text-sm font-bold text-purple-600 dark:text-purple-400">
                      {quarter?.eps !== undefined
                        ? quarter.eps.toFixed(2)
                        : 'N/A'}
                    </span>
                    <span className="text-xs text-slate-400 sm:hidden">
                      EPS
                    </span>
                  </div>

                  {/* EPS Growth - More compact */}
                  <div className="flex flex-col items-end sm:items-end min-w-[70px]">
                    {quarter?.eps_growth !== undefined ? (
                      <div className="flex items-center gap-1">
                        <div
                          className={`w-4 h-4 rounded-full flex items-center justify-center ${quarter.eps_growth >= 0 ? 'bg-emerald-100 dark:bg-emerald-900/50' : 'bg-red-100 dark:bg-red-900/50'}`}
                        >
                          <span
                            className={`text-xs ${quarter.eps_growth >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}`}
                          >
                            {quarter.eps_growth >= 0 ? '▲' : '▼'}
                          </span>
                        </div>
                        <span
                          className={`font-bold text-sm ${quarter.eps_growth >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}`}
                        >
                          {quarter.eps_growth >= 0 ? '+' : ''}
                          {quarter.eps_growth}%
                        </span>
                      </div>
                    ) : (
                      <span className="text-slate-400 text-sm">-</span>
                    )}
                    <span className="text-xs text-slate-400 sm:hidden">
                      EPS %
                    </span>
                  </div>

                  {/* Price Growth - More compact */}
                  <div className="flex flex-col items-end sm:items-end min-w-[70px]">
                    {quarter?.price_growth !== undefined &&
                    quarter.price_growth !== null ? (
                      <div className="flex items-center gap-1">
                        <div
                          className={`w-4 h-4 rounded-full flex items-center justify-center ${quarter.price_growth >= 0 ? 'bg-emerald-100 dark:bg-emerald-900/50' : 'bg-red-100 dark:bg-red-900/50'}`}
                        >
                          <span
                            className={`text-xs ${quarter.price_growth >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}`}
                          >
                            {quarter.price_growth >= 0 ? '▲' : '▼'}
                          </span>
                        </div>
                        <span
                          className={`font-bold text-sm ${quarter.price_growth >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}`}
                        >
                          {quarter.price_growth >= 0 ? '+' : ''}
                          {quarter.price_growth}%
                        </span>
                      </div>
                    ) : (
                      <span className="text-slate-400 text-sm">-</span>
                    )}
                    <span className="text-xs text-slate-400 sm:hidden">
                      Price %
                    </span>
                  </div>
                </div>
              ))}
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

  // Add loading state for better UX
  if (!safeData.length) {
    return (
      <div
        className={`w-full min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-50 via-blue-50/30 to-purple-50/50 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950 ${className || ''}`}
        style={style}
      >
        <div className="text-center space-y-4">
          <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center animate-pulse">
            <span className="text-white text-2xl">📊</span>
          </div>
          <h3 className="text-xl font-semibold text-slate-600 dark:text-slate-400">
            Loading Data...
          </h3>
          <p className="text-slate-500 dark:text-slate-500">
            Please wait while we fetch the latest rankings
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`w-full min-h-screen bg-gradient-to-br from-slate-50 via-blue-50/30 to-purple-50/50 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950 transition-all duration-500 ${className || ''}`}
      style={style}
    >
      {/* Enhanced Header Section */}
      <div className="relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-r from-blue-600/5 via-purple-600/5 to-pink-600/5 dark:from-blue-500/10 dark:via-purple-500/10 dark:to-pink-500/10" />

        {/* Floating background elements */}
        <div className="absolute top-10 left-10 w-20 h-20 bg-blue-400/10 rounded-full blur-xl animate-pulse"></div>
        <div
          className="absolute top-20 right-20 w-32 h-32 bg-purple-400/10 rounded-full blur-xl animate-pulse"
          style={{ animationDelay: '1s' }}
        ></div>
        <div
          className="absolute bottom-10 left-1/3 w-24 h-24 bg-pink-400/10 rounded-full blur-xl animate-pulse"
          style={{ animationDelay: '2s' }}
        ></div>

        <div className="relative px-6 sm:px-12 pt-8 pb-12">
          <div className="max-w-7xl mx-auto">
            <div className="text-center space-y-6">
              <h1 className="text-4xl sm:text-6xl lg:text-7xl font-black bg-gradient-to-r from-blue-600 via-purple-600 to-pink-600 bg-clip-text text-transparent leading-tight animate-in slide-in-from-top-4 fade-in duration-1000">
                Rankings
              </h1>

              <p
                className="text-lg sm:text-xl text-slate-600 dark:text-slate-400 max-w-3xl mx-auto leading-relaxed animate-in slide-in-from-bottom-4 fade-in duration-1000"
                style={{ animationDelay: '200ms' }}
              >
                Discover top-performing stocks with comprehensive quarterly
                analysis, real-time data, and intelligent growth metrics
              </p>

              <div
                className="flex flex-wrap items-center justify-center gap-6 sm:gap-8 mt-8 animate-in slide-in-from-bottom-4 fade-in duration-1000"
                style={{ animationDelay: '400ms' }}
              >
                <div className="flex items-center gap-3 px-4 py-2 rounded-full bg-emerald-50 dark:bg-emerald-900/20 hover:bg-emerald-100 dark:hover:bg-emerald-900/30 transition-colors duration-200">
                  <div className="w-4 h-4 rounded-full bg-emerald-400 animate-pulse"></div>
                  <span className="text-sm font-semibold text-emerald-700 dark:text-emerald-300">
                    Growth Trending
                  </span>
                </div>
                <div className="flex items-center gap-3 px-4 py-2 rounded-full bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors duration-200">
                  <div
                    className="w-4 h-4 rounded-full bg-blue-400 animate-pulse"
                    style={{ animationDelay: '0.5s' }}
                  ></div>
                  <span className="text-sm font-semibold text-blue-700 dark:text-blue-300">
                    Live Data
                  </span>
                </div>
                <div className="flex items-center gap-3 px-4 py-2 rounded-full bg-purple-50 dark:bg-purple-900/20 hover:bg-purple-100 dark:hover:bg-purple-900/30 transition-colors duration-200">
                  <div
                    className="w-4 h-4 rounded-full bg-purple-400 animate-pulse"
                    style={{ animationDelay: '1s' }}
                  ></div>
                  <span className="text-sm font-semibold text-purple-700 dark:text-purple-300">
                    Multi-Quarter Analysis
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Cards Container with enhanced spacing and staggered animation */}
      <div className="relative px-6 sm:px-12 py-12">
        <div className="max-w-7xl mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {safeData.map((item, index) => (
              <div
                key={`${item.symbol}-${index}`}
                className="animate-in slide-in-from-bottom-4 fade-in"
                style={{
                  animationDelay: `${index * 150}ms`,
                  animationDuration: '600ms',
                  animationFillMode: 'both',
                }}
              >
                <FinancialDataCard data={item} index={index} />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default FinancialDataTable;
