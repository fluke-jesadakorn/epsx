'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ProfessionalTileProps {
  size?: 'sm' | 'md' | 'lg' | 'wide';
  variant?: 'default' | 'analytics' | 'premium';
  children?: ReactNode;
  icon?: ReactNode;
  badge?: string | number;
  onClick?: () => void;
  className?: string;
  title?: string;
  subtitle?: string;
}

export function ProfessionalTile({
  size = 'md',
  variant = 'default',
  children,
  icon,
  badge,
  onClick,
  className,
  title,
  subtitle
}: ProfessionalTileProps) {
  const sizeClasses = {
    sm: 'w-20 h-20',
    md: 'w-32 h-32',
    lg: 'w-32 h-48',
    wide: 'w-64 h-32'
  };

  const variantStyles = {
    default: {
      bg: 'bg-gradient-to-br from-blue-500 to-blue-600',
      hover: 'hover:from-blue-600 hover:to-blue-700',
      accent: 'bg-blue-700',
      text: 'text-white'
    },
    analytics: {
      bg: 'bg-gradient-to-br from-indigo-500 to-purple-600',
      hover: 'hover:from-indigo-600 hover:to-purple-700',
      accent: 'bg-purple-700',
      text: 'text-white'
    },
    premium: {
      bg: 'bg-gradient-to-br from-purple-500 to-pink-600',
      hover: 'hover:from-purple-600 hover:to-pink-700',
      accent: 'bg-pink-700',
      text: 'text-white'
    }
  };

  const style = variantStyles[variant];

  return (
    <div
      className={cn(
        sizeClasses[size],
        style.bg,
        style.hover,
        style.text,
        'relative overflow-hidden cursor-pointer rounded-lg shadow-lg hover:shadow-xl transition-all duration-300 hover:-translate-y-1 group',
        onClick && 'cursor-pointer',
        className
      )}
      onClick={onClick}
    >
      {/* Badge */}
      {badge && (
        <div className={cn(
          'absolute -top-2 -right-2 text-white rounded-full w-6 h-6 flex items-center justify-center text-xs font-bold z-20',
          style.accent
        )}>
          {badge}
        </div>
      )}

      {/* Content */}
      <div className="relative z-10 h-full flex flex-col p-4">
        {/* Icon */}
        {icon && (
          <div className="text-2xl mb-2 flex-shrink-0">
            {icon}
          </div>
        )}

        {/* Title and Subtitle */}
        {(title || subtitle) && (
          <div className="mb-2 flex-shrink-0">
            {title && (
              <div className="font-semibold text-sm mb-1 leading-tight">
                {title}
              </div>
            )}
            {subtitle && (
              <div className="text-xs opacity-80 leading-tight">
                {subtitle}
              </div>
            )}
          </div>
        )}

        {/* Children Content */}
        <div className="flex-1 flex flex-col justify-center">
          {children}
        </div>

        {/* Bottom accent line */}
        <div className={cn(
          'absolute bottom-0 left-0 h-1 w-full transition-all duration-300 group-hover:h-2',
          style.accent
        )} />
      </div>

      {/* Hover effect overlay */}
      <div className="absolute inset-0 bg-white opacity-0 group-hover:opacity-10 transition-opacity duration-300" />
    </div>
  );
}

// Professional Dashboard Grid Component
interface ProfessionalDashboardProps {
  children: ReactNode;
  className?: string;
  columns?: 'auto' | '2' | '3' | '4' | '6';
}

export function ProfessionalDashboard({ 
  children, 
  className = '',
  columns = 'auto'
}: ProfessionalDashboardProps) {
  const columnClasses = {
    auto: 'grid-cols-[repeat(auto-fit,minmax(128px,1fr))]',
    '2': 'grid-cols-2',
    '3': 'grid-cols-3',
    '4': 'grid-cols-4',
    '6': 'grid-cols-6'
  };

  return (
    <div className={cn(
      'grid gap-4 p-4',
      columnClasses[columns],
      className
    )}>
      {children}
    </div>
  );
}

// Specialized tiles
export function AnalyticsTile({ ...props }: Omit<ProfessionalTileProps, 'variant'>) {
  return <ProfessionalTile variant="analytics" {...props} />;
}

export function PremiumTile({ ...props }: Omit<ProfessionalTileProps, 'variant'>) {
  return <ProfessionalTile variant="premium" {...props} />;
}

// Stats Tile
interface ProfessionalStatsTileProps extends Omit<ProfessionalTileProps, 'children'> {
  value: string | number;
  label: string;
  trend?: 'up' | 'down' | 'neutral';
  percentage?: string;
}

export function ProfessionalStatsTile({
  value,
  label,
  trend,
  percentage,
  ...props
}: ProfessionalStatsTileProps) {
  const trendIcons = {
    up: '↗',
    down: '↘',
    neutral: '→'
  };

  const trendColors = {
    up: 'text-green-300',
    down: 'text-red-300',
    neutral: 'text-gray-300'
  };

  return (
    <ProfessionalTile {...props}>
      <div className="text-center">
        <div className="text-2xl font-bold mb-1">
          {value}
        </div>
        <div className="text-xs opacity-80 mb-2">
          {label}
        </div>
        {trend && percentage && (
          <div className={cn(
            'text-xs flex items-center justify-center gap-1',
            trendColors[trend]
          )}>
            <span>{trendIcons[trend]}</span>
            <span>{percentage}</span>
          </div>
        )}
      </div>
    </ProfessionalTile>
  );
}

// Action Tile
interface ProfessionalActionTileProps extends Omit<ProfessionalTileProps, 'children'> {
  action: string;
  description?: string;
}

export function ProfessionalActionTile({
  action,
  description,
  ...props
}: ProfessionalActionTileProps) {
  return (
    <ProfessionalTile {...props}>
      <div className="text-center">
        <div className="font-semibold text-sm mb-1">
          {action}
        </div>
        {description && (
          <div className="text-xs opacity-80">
            {description}
          </div>
        )}
      </div>
    </ProfessionalTile>
  );
}