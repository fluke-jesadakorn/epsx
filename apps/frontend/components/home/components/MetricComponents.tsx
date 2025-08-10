import React from 'react';
import { GRADIENTS, COLORS, SPACING, TYPOGRAPHY } from '../constants/styles';

interface MetricCardProps {
  title: string;
  value: string;
  type: 'price' | 'eps' | 'growth';
  className?: string;
}

/**
 * Reusable metric card component
 */
export function MetricCard({ title, value, type, className = '' }: MetricCardProps): React.JSX.Element {
  const gradientMap = {
    price: GRADIENTS.metric.price,
    eps: GRADIENTS.metric.eps,
    growth: GRADIENTS.metric.growth,
  };

  const colorMap = {
    price: COLORS.primary,
    eps: COLORS.secondary,
    growth: COLORS.positive,
  };

  const colors = colorMap[type];
  const gradient = gradientMap[type];

  return (
    <div className={`
      ${SPACING.gridGap} 
      rounded-xl 
      bg-gradient-to-br ${gradient} 
      border ${colors.border}
      min-w-0 w-full overflow-hidden
      ${className}
    `}>
      <div className={`${TYPOGRAPHY.caption} ${colors.text} font-semibold tracking-wide uppercase mb-2 truncate`}>
        {title}
      </div>
      <div className={`${TYPOGRAPHY.metric} ${colors.text} truncate`}>
        {value}
      </div>
    </div>
  );
}

interface QuarterRowProps {
  quarter: {
    quarter: number | string;
    date: string;
    price?: number | null;
    eps?: number;
    eps_growth?: number;
    price_growth?: number | null;
  };
  formatPrice: (price: number) => string;
  formatDate: (date: string) => string;
  className?: string;
}

/**
 * Quarter data row component
 */
export function QuarterRow({ 
  quarter, 
  formatPrice, 
  formatDate, 
  className = '' 
}: QuarterRowProps): React.JSX.Element {
  return (
    <div className={`
      group/row relative 
      flex flex-col sm:grid sm:grid-cols-5 md:grid-cols-5
      ${SPACING.itemGap} p-3 sm:p-3 rounded-lg 
      border ${COLORS.neutral.border} 
      bg-gradient-to-r from-white to-slate-50/50 
      dark:from-slate-800/30 dark:to-slate-700/30 
      hover:shadow-md hover:border-slate-300/50 
      dark:hover:border-slate-600/50 
      transition-all duration-200 
      hover:bg-gradient-to-r hover:from-slate-50 hover:to-white 
      dark:hover:from-slate-700/50 dark:hover:to-slate-800/50
      ${className}
    `}>
      
      {/* Mobile layout - shows all data stacked */}
      <div className="sm:hidden space-y-3">
        {/* Quarter and Date */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className={`${TYPOGRAPHY.body} text-slate-800 dark:text-white font-semibold`}>
              {typeof quarter.quarter === 'string' ? quarter.quarter : `Q${quarter.quarter}`}
            </span>
            <div className="w-1.5 h-1.5 rounded-full bg-gradient-to-r from-blue-400 to-purple-500" />
          </div>
          <span className={`${TYPOGRAPHY.caption} text-slate-500 dark:text-slate-400 font-medium`}>
            {formatDate(quarter.date)}
          </span>
        </div>
        
        {/* Metrics in 2x2 grid */}
        <div className="grid grid-cols-2 gap-3">
          {/* Price */}
          <div className="text-center p-2 rounded-lg bg-slate-50 dark:bg-slate-700/50">
            <div className={`${TYPOGRAPHY.caption} text-slate-500 dark:text-slate-400 mb-1`}>
              Price
            </div>
            <div className={`${TYPOGRAPHY.price} ${COLORS.primary.text} font-bold`}>
              {quarter?.price !== undefined && quarter?.price !== null
                ? formatPrice(quarter.price)
                : 'N/A'}
            </div>
          </div>
          
          {/* EPS */}
          <div className="text-center p-2 rounded-lg bg-slate-50 dark:bg-slate-700/50">
            <div className={`${TYPOGRAPHY.caption} text-slate-500 dark:text-slate-400 mb-1`}>
              EPS
            </div>
            <div className={`${TYPOGRAPHY.price} ${COLORS.secondary.text} font-bold`}>
              {quarter?.eps !== undefined ? quarter.eps.toFixed(4) : 'N/A'}
            </div>
          </div>
          
          {/* EPS Growth */}
          <div className="text-center p-2 rounded-lg bg-slate-50 dark:bg-slate-700/50">
            <div className={`${TYPOGRAPHY.caption} text-slate-500 dark:text-slate-400 mb-1`}>
              EPS %
            </div>
            <div className="flex items-center justify-center">
              {quarter?.eps_growth !== undefined ? (
                <div className="flex items-center gap-1">
                  <div className={`
                    w-3 h-3 rounded-full flex items-center justify-center 
                    ${quarter.eps_growth >= 0 ? COLORS.positive.bg : COLORS.negative.bg}
                  `}>
                    <span className={`
                      text-xs 
                      ${quarter.eps_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
                    `}>
                      {quarter.eps_growth >= 0 ? '▲' : '▼'}
                    </span>
                  </div>
                  <span className={`
                    font-bold text-sm 
                    ${quarter.eps_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
                  `}>
                    {quarter.eps_growth >= 0 ? '+' : ''}{Math.round(quarter.eps_growth)}%
                  </span>
                </div>
              ) : (
                <span className="text-slate-400 text-sm">-</span>
              )}
            </div>
          </div>
          
          {/* Price Growth */}
          <div className="text-center p-2 rounded-lg bg-slate-50 dark:bg-slate-700/50">
            <div className={`${TYPOGRAPHY.caption} text-slate-500 dark:text-slate-400 mb-1`}>
              Price %
            </div>
            <div className="flex items-center justify-center">
              {quarter?.price_growth !== undefined && quarter.price_growth !== null ? (
                <div className="flex items-center gap-1">
                  <div className={`
                    w-3 h-3 rounded-full flex items-center justify-center 
                    ${quarter.price_growth >= 0 ? COLORS.positive.bg : COLORS.negative.bg}
                  `}>
                    <span className={`
                      text-xs 
                      ${quarter.price_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
                    `}>
                      {quarter.price_growth >= 0 ? '▲' : '▼'}
                    </span>
                  </div>
                  <span className={`
                    font-bold text-sm 
                    ${quarter.price_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
                  `}>
                    {quarter.price_growth >= 0 ? '+' : ''}{Math.round(quarter.price_growth)}%
                  </span>
                </div>
              ) : (
                <span className="text-slate-400 text-sm">-</span>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Desktop layout - shows data in grid columns */}
      <div className="hidden sm:contents">
        {/* Quarter info */}
        <div className="flex flex-col items-start">
          <div className="flex items-center gap-2">
            <span className={`${TYPOGRAPHY.body} text-slate-800 dark:text-white`}>
              {typeof quarter.quarter === 'string' ? quarter.quarter : `Q${quarter.quarter}`}
            </span>
            <div className="w-1.5 h-1.5 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 group-hover/row:scale-125 transition-transform duration-200" />
          </div>
          <span className={`${TYPOGRAPHY.caption} text-slate-500 dark:text-slate-400 font-medium`}>
            {formatDate(quarter.date)}
          </span>
        </div>

        {/* Price */}
        <div className="flex flex-col items-end">
          <span className={`${TYPOGRAPHY.price} ${COLORS.primary.text}`}>
            {quarter?.price !== undefined && quarter?.price !== null
              ? formatPrice(quarter.price)
              : 'N/A'}
          </span>
        </div>

        {/* EPS */}
        <div className="flex flex-col items-end">
          <span className={`${TYPOGRAPHY.price} ${COLORS.secondary.text}`}>
            {quarter?.eps !== undefined ? quarter.eps.toFixed(4) : 'N/A'}
          </span>
        </div>

        {/* EPS Growth */}
        <div className="flex flex-col items-end">
          {quarter?.eps_growth !== undefined ? (
            <div className="flex items-center gap-1">
              <div className={`
                w-4 h-4 rounded-full flex items-center justify-center 
                ${quarter.eps_growth >= 0 ? COLORS.positive.bg : COLORS.negative.bg}
              `}>
                <span className={`
                  text-xs 
                  ${quarter.eps_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
                `}>
                  {quarter.eps_growth >= 0 ? '▲' : '▼'}
                </span>
              </div>
              <span className={`
                font-bold text-sm 
                ${quarter.eps_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
              `}>
                {quarter.eps_growth >= 0 ? '+' : ''}{quarter.eps_growth}%
              </span>
            </div>
          ) : (
            <span className="text-slate-400 text-sm">-</span>
          )}
        </div>

        {/* Price Growth */}
        <div className="flex flex-col items-end">
          {quarter?.price_growth !== undefined && quarter.price_growth !== null ? (
            <div className="flex items-center gap-1">
              <div className={`
                w-4 h-4 rounded-full flex items-center justify-center 
                ${quarter.price_growth >= 0 ? COLORS.positive.bg : COLORS.negative.bg}
              `}>
                <span className={`
                  text-xs 
                  ${quarter.price_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
                `}>
                  {quarter.price_growth >= 0 ? '▲' : '▼'}
                </span>
              </div>
              <span className={`
                font-bold text-sm 
                ${quarter.price_growth >= 0 ? COLORS.positive.text : COLORS.negative.text}
              `}>
                {quarter.price_growth >= 0 ? '+' : ''}{Math.round(quarter.price_growth)}%
              </span>
            </div>
          ) : (
            <span className="text-slate-400 text-sm">-</span>
          )}
        </div>
      </div>
    </div>
  );
}
