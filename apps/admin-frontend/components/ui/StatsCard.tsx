'use client';

import { ReactElement } from 'react';
import { LucideIcon } from 'lucide-react';

export interface StatsCardProps {
  title: string;
  value: string | number;
  description?: string;
  icon: LucideIcon;
  variant?: 'default' | 'enhanced' | 'simple' | 'inline';
  gradient?: string;
  textColor?: string;
  color?: string;
  className?: string;
  onClick?: () => void;
}

export function StatsCard({
  title,
  value,
  description,
  icon: Icon,
  variant = 'default',
  gradient,
  textColor,
  color,
  className = '',
  onClick
}: StatsCardProps) {
  const baseClasses = 'relative overflow-hidden transition-all duration-200';
  const clickableClasses = onClick ? 'cursor-pointer hover:shadow-lg' : '';

  if (variant === 'enhanced') {
    // AdminDashboard style - elaborate with gradients and animations
    return (
      <div
        className={`pancake-card pancake-card-hover p-6 ${baseClasses} group ${clickableClasses} ${className}`}
        onClick={onClick}
      >
        <div className="absolute top-0 right-0 w-20 h-20 -mr-8 -mt-8 rounded-full bg-gradient-to-br from-white/5 to-white/10 group-hover:scale-110 transition-transform duration-500"></div>
        <div className="relative">
          <div className="flex items-center justify-between mb-4">
            <div className={`p-3 rounded-xl bg-gradient-to-r ${gradient || 'from-blue-500 to-purple-500'} text-white shadow-lg`}>
              <Icon className="h-6 w-6" />
            </div>
            <div className={`text-right ${textColor || 'text-blue-500'}`}>
              <div className="text-2xl font-bold leading-none">
                {value}
              </div>
            </div>
          </div>
          <div>
            <h3 className="font-semibold text-foreground mb-1">
              {title}
            </h3>
            {description && (
              <p className="text-sm text-muted-foreground">
                {description}
              </p>
            )}
          </div>
        </div>
      </div>
    );
  }

  if (variant === 'simple') {
    // IAMDashboard style - simple with icon on the right
    return (
      <div
        className={`pancake-card p-6 ${baseClasses} ${clickableClasses} ${className}`}
        onClick={onClick}
      >
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-muted-foreground">{title}</p>
            <p className="text-2xl font-bold mt-1">{value}</p>
            {description && (
              <p className="text-sm text-muted-foreground mt-1">{description}</p>
            )}
          </div>
          <div className={`p-3 rounded-full bg-gradient-to-r ${color || gradient || 'from-blue-500 to-blue-600'}`}>
            <Icon className="h-6 w-6 text-white" />
          </div>
        </div>
      </div>
    );
  }

  if (variant === 'inline') {
    // ModuleAnalyticsDashboard style - compact with inline icon
    return (
      <div
        className={`bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 ${baseClasses} ${clickableClasses} ${className}`}
        onClick={onClick}
      >
        <div className="flex items-center">
          <Icon className={`w-5 h-5 text-${color || 'blue'}-600`} />
          <span className="ml-2 text-sm font-medium text-gray-600 dark:text-gray-300">{title}</span>
        </div>
        <div className="mt-2">
          <span className="text-2xl font-bold text-gray-900 dark:text-white">{value}</span>
        </div>
        {description && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">{description}</p>
        )}
      </div>
    );
  }

  // Default variant - balanced design
  return (
    <div
      className={`bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-6 shadow-sm ${baseClasses} hover:shadow-md ${clickableClasses} ${className}`}
      onClick={onClick}
    >
      <div className="flex items-center justify-between mb-3">
        <div className={`p-2 rounded-lg ${gradient ? `bg-gradient-to-r ${gradient}` : `bg-${color || 'blue'}-100`}`}>
          <Icon className={`h-5 w-5 ${gradient ? 'text-white' : `text-${color || 'blue'}-600`}`} />
        </div>
        {textColor && (
          <div className={`text-right ${textColor}`}>
            <div className="text-xl font-bold">
              {value}
            </div>
          </div>
        )}
      </div>
      <div>
        <h3 className="text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">
          {title}
        </h3>
        {!textColor && (
          <div className="text-2xl font-bold text-gray-900 dark:text-white">
            {value}
          </div>
        )}
        {description && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {description}
          </p>
        )}
      </div>
    </div>
  );
}

// Convenience wrapper for multiple stats cards
export interface StatsGridProps {
  stats: StatsCardProps[];
  variant?: StatsCardProps['variant'];
  columns?: {
    default?: number;
    sm?: number;
    md?: number;
    lg?: number;
    xl?: number;
  };
  className?: string;
}

export function StatsGrid({ 
  stats, 
  variant = 'default', 
  columns = { default: 1, md: 2, lg: 4 },
  className = ''
}: StatsGridProps) {
  const gridClasses = [
    `grid gap-4`,
    columns.default && `grid-cols-${columns.default}`,
    columns.sm && `sm:grid-cols-${columns.sm}`,
    columns.md && `md:grid-cols-${columns.md}`,
    columns.lg && `lg:grid-cols-${columns.lg}`,
    columns.xl && `xl:grid-cols-${columns.xl}`,
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={gridClasses}>
      {stats.map((stat, index) => (
        <StatsCard
          key={index}
          {...stat}
          variant={stat.variant || variant}
        />
      ))}
    </div>
  );
}