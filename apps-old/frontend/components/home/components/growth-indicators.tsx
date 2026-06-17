import React from 'react';
import { COLORS } from '../constants/styles';

interface GrowthIndicatorProps {
  value: number | undefined | null;
  size?: 'sm' | 'md' | 'lg';
  showIcon?: boolean;
  className?: string;
}

/**
 * Reusable growth indicator component
 */
export function GrowthIndicator({
  value,
  size = 'md',
  showIcon = true,
  className = '',
}: GrowthIndicatorProps): React.JSX.Element {
  const isPositive = (value ?? 0) >= 0;
  const colors = isPositive ? COLORS.positive : COLORS.negative;

  const sizeClasses = {
    sm: 'w-4 h-4 text-xs',
    md: 'w-5 h-5 text-sm',
    lg: 'w-7 h-7 text-sm',
  };

  const iconClasses = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-sm',
  };

  if (value === undefined || value === null) {
    return <span className={`${COLORS.neutral.text} ${className}`}>-</span>;
  }

  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {showIcon && (
        <div
          className={`${sizeClasses[size]} rounded-full flex items-center justify-center ${colors.bg}`}
        >
          <span className={`${iconClasses[size]} ${colors.text}`}>
            {isPositive ? '▲' : '▼'}
          </span>
        </div>
      )}
      <span className={`font-bold ${colors.text}`}>
        {isPositive ? '+' : ''}
        {value}%
      </span>
    </div>
  );
}

interface TrendIconProps {
  direction: 'up' | 'down';
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

/**
 * Simple trend icon component
 */
export function TrendIcon({
  direction,
  size = 'md',
  className = '',
}: TrendIconProps): React.JSX.Element {
  const isUp = direction === 'up';
  const colors = isUp ? COLORS.positive : COLORS.negative;

  const sizeClasses = {
    sm: 'w-5 h-5',
    md: 'w-7 h-7',
    lg: 'w-8 h-8',
  };

  return (
    <div
      className={`
      ${sizeClasses[size]} 
      rounded-full 
      flex items-center justify-center 
      ${colors.bg} 
      ${colors.text}
      transition-all duration-300 group-hover:scale-110
      ${className}
    `}
    >
      <span className="text-sm font-bold">{isUp ? '↗' : '↘'}</span>
    </div>
  );
}

interface AnimatedBadgeProps {
  children: React.ReactNode;
  rank: number;
  isHovered?: boolean;
  className?: string;
}

/**
 * Animated rank badge component
 */
export function AnimatedBadge({
  children,
  rank,
  isHovered = false,
  className = '',
}: AnimatedBadgeProps): React.JSX.Element {
  return (
    <div
      className={`
        ${isHovered ? 'scale-110 shadow-xl' : ''}
        ${className}
      `}
      aria-label={`Rank ${rank}`}
    >
      <span className="drop-shadow-lg">{children}</span>
    </div>
  );
}
