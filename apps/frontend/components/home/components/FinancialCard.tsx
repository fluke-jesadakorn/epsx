import React from 'react';
import { Card, CardContent } from '@/components/ui/card';
import {
  GRADIENTS,
  COLORS,
  SPACING,
  TYPOGRAPHY,
  ANIMATIONS,
} from '../constants/styles';
import { GrowthIndicator, TrendIcon, AnimatedBadge } from './GrowthIndicators';
import { MetricCard, QuarterRow } from './MetricComponents';
import { useFinancialData, getValidQuarters } from '../hooks/useFinancialData';
import type { StockFinancialData } from '@/types/financialChartData';
import {
  formatPrice,
  formatDate,
} from '@/utils/transformers/stockDataTransformer';

interface FinancialCardProps {
  data: StockFinancialData;
  index: number;
}

/**
 * Individual financial data card component
 */
export function FinancialCard({
  data,
  index,
}: FinancialCardProps): React.JSX.Element {
  const [isPressed, setIsPressed] = React.useState(false);
  const [isHovered, setIsHovered] = React.useState(false);

  const { latestQuarter, avgGrowth, displayPrice, hasGrowthData } =
    useFinancialData(data);

  const validQuarters = getValidQuarters(data.quarters);

  const handleInteractionStart = () => setIsPressed(true);
  const handleInteractionEnd = () => setIsPressed(false);
  const handleMouseEnter = () => setIsHovered(true);
  const handleMouseLeave = () => {
    setIsPressed(false);
    setIsHovered(false);
  };

  return (
    <Card
      className={`
        group w-full transition-all duration-300 ease-out 
        ${ANIMATIONS.scalePancake} 
        hover:shadow-2xl hover:shadow-orange-500/20 dark:hover:shadow-yellow-900/30 
        border-0 
        bg-gradient-to-br ${GRADIENTS.card}
        rounded-2xl shadow-lg 
        ${GRADIENTS.cardHover}
        overflow-hidden backdrop-blur-sm cursor-pointer 
        focus:outline-none focus:ring-4 focus:ring-orange-500/20 
        ${isPressed ? `${ANIMATIONS.scalePress} opacity-90` : ''}
        relative
      `}
      tabIndex={0}
      role="button"
      aria-label={`Financial data for ${data.symbol}`}
      onTouchStart={handleInteractionStart}
      onTouchEnd={handleInteractionEnd}
      onMouseDown={handleInteractionStart}
      onMouseUp={handleInteractionEnd}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onFocus={() => setIsHovered(true)}
      onBlur={() => setIsHovered(false)}
    >
      {/* Animated Background Gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-orange-500/5 via-yellow-500/5 to-amber-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />

      {/* PancakeSwap-style decorative elements */}
      <div className="absolute top-2 right-2 text-2xl opacity-20 group-hover:opacity-40 transition-opacity duration-300 animate-bounce-gentle">
        🥞
      </div>
      <div className="absolute bottom-2 left-2 text-lg opacity-15 group-hover:opacity-30 transition-opacity duration-300 animate-pulse">
        💰
      </div>

      {/* Rank Badge */}
      <div className="absolute top-4 left-4 z-30 w-12 h-12 flex items-center justify-center drop-shadow-lg ring-2 ring-orange-400/60 backdrop-blur-md bg-white/60 dark:bg-slate-900/60 rounded-full transition-all duration-300 hover:scale-105 hover:ring-4 hover:ring-orange-500/80">
        {/* Decorative background */}
        <div className="absolute inset-0 rounded-full bg-gradient-to-br from-orange-200/70 via-yellow-100/60 to-amber-100/50 dark:from-orange-900/40 dark:via-yellow-900/30 dark:to-amber-900/30 blur-[2px] z-[-1]" />
        <AnimatedBadge rank={index + 1} isHovered={isHovered}>
          <span className="flex items-center gap-1">
        #{index + 1}
        {index === 0 && <span className="text-xs">🏆</span>}
        {index === 1 && <span className="text-xs">🥈</span>}
        {index === 2 && <span className="text-xs">🥉</span>}
          </span>
        </AnimatedBadge>
      </div>

      {/* Performance Indicator with PancakeSwap glow */}
      {avgGrowth && avgGrowth > 0 && (
        <div className="absolute top-0 right-0 w-20 h-20 bg-gradient-to-br from-orange-400/20 via-yellow-400/30 to-amber-400/20 rounded-full blur-xl opacity-50 group-hover:opacity-75 transition-opacity duration-300" />
      )}

      {/* Animated Border */}
      <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-orange-500/20 via-yellow-500/20 to-amber-500/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300 blur-sm" />

      {/* Content */}
      <CardContent className={`relative ${SPACING.cardPadding} z-10`}>
        {/* Header */}
        <div className="flex items-start justify-between mb-5">
          <div className="flex items-center gap-3 ml-12">
            <div
              className={`${TYPOGRAPHY.title} text-slate-800 dark:text-white tracking-tight flex items-center gap-2`}
            >
              <span className="text-2xl">📈</span>
              {data.symbol}
              <span className="text-sm opacity-60">🚀</span>
            </div>
            {hasGrowthData && (
              <TrendIcon
                direction={
                  data.quarters[0].eps_growth! > data.quarters[1].eps_growth!
                    ? 'up'
                    : 'down'
                }
              />
            )}
          </div>
          <div className="text-right">
            <div
              className={`${TYPOGRAPHY.caption} ${COLORS.neutral.text} font-medium uppercase tracking-wider flex items-center gap-1`}
            >
              <span className="text-xs">📅</span>
              Latest Date
            </div>
            <div
              className={`${TYPOGRAPHY.body} text-slate-700 dark:text-slate-300`}
            >
              {latestQuarter?.date ? formatDate(latestQuarter.date) : 'N/A'}
            </div>
          </div>
        </div>

        {/* Key Metrics */}
        <div className={`grid grid-cols-3 ${SPACING.gridGap} mb-6`}>
          <MetricCard
            title="Latest Price"
            value={displayPrice !== null ? formatPrice(displayPrice) : 'N/A'}
            type="price"
            className="p-4"
          />
          <MetricCard
            title="Latest EPS"
            value={
              latestQuarter?.eps !== undefined
                ? latestQuarter.eps.toFixed(4)
                : 'N/A'
            }
            type="eps"
            className="p-4"
          />
          <div className="p-4 rounded-xl bg-gradient-to-br from-emerald-50 to-emerald-100/50 dark:from-emerald-900/20 dark:to-emerald-800/20 border border-emerald-200/50 dark:border-emerald-700/30">
            <div
              className={`${TYPOGRAPHY.caption} text-emerald-600 dark:text-emerald-400 font-semibold tracking-wide uppercase mb-2`}
            >
              Avg Growth
            </div>
            <div className="flex items-center gap-2">
              <GrowthIndicator value={avgGrowth} size="md" />
            </div>
          </div>
        </div>

        {/* TradingView Link */}
        <div className="mb-6">
          <a
            href={`https://www.tradingview.com/chart?symbol=${data.symbol}`}
            target="_blank"
            rel="noopener noreferrer"
            className={`
              flex items-center justify-center gap-2 w-full p-3 rounded-xl 
              bg-gradient-to-r ${GRADIENTS.button} 
              text-white font-semibold shadow-md 
              ${GRADIENTS.buttonHover}
              transition-all duration-200 
              focus:outline-none focus:ring-2 focus:ring-orange-400/60 focus:ring-offset-2 
              dark:focus:ring-orange-700/60 hover:shadow-lg ${ANIMATIONS.scalePancake}
              group
            `}
          >
            <span className="text-lg group-hover:animate-bounce">📊</span>
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
            <span className="text-lg group-hover:animate-pulse">🚀</span>
          </a>
        </div>

        {/* Quarterly Performance */}
        <div className="space-y-3">
          <div className="flex items-center gap-3 pb-2 border-b border-slate-200/70 dark:border-slate-700/70">
            <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-orange-500 to-yellow-600 flex items-center justify-center">
              <span className="text-white text-xs font-bold">📊</span>
            </div>
            <h3
              className={`${TYPOGRAPHY.body} text-slate-800 dark:text-white flex items-center gap-2`}
            >
              <span className="text-sm">🏆</span>
              Quarterly Performance
              <span className="text-sm">📈</span>
            </h3>
            <div className="ml-auto">
              <span
                className={`${TYPOGRAPHY.caption} bg-gradient-to-r from-orange-500 to-yellow-600 text-white px-2 py-1 rounded-full font-medium shadow-sm flex items-center gap-1`}
              >
                <span className="text-xs">⏰</span>
                {data.quarters.length - 1}Q
              </span>
            </div>
          </div>

          {/* Table Header */}
          <div className="hidden sm:grid grid-cols-5 gap-3 px-3 py-2 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
            {['Quarter', 'Price', 'EPS', 'EPS %', 'Price %'].map((header) => (
              <div
                key={header}
                className={`${TYPOGRAPHY.caption} font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wide ${header === 'Quarter' ? '' : 'text-right'}`}
              >
                {header}
              </div>
            ))}
          </div>

          {/* Quarter Rows */}
          <div className="space-y-2">
            {validQuarters.map((quarter, idx) => (
              <QuarterRow
                key={idx}
                quarter={quarter}
                formatPrice={formatPrice}
                formatDate={formatDate}
              />
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
