'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface MetroCardProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  children: ReactNode;
  className?: string;
  hover?: boolean;
  accent?: 'left' | 'top' | 'bottom' | 'right';
  glassmorphism?: boolean;
}

export function MetroCard({
  variant = 'pancake',
  children,
  className = '',
  hover = true,
  accent = 'left',
  glassmorphism = false
}: MetroCardProps) {
  const variants = {
    pancake: {
      bg: glassmorphism ? 'bg-white/10' : 'bg-white',
      border: 'border-orange-500',
      accent: 'bg-gradient-to-r from-orange-400 to-yellow-500',
      shadow: 'shadow-orange-200',
      text: glassmorphism ? 'text-white' : 'text-gray-800'
    },
    admin: {
      bg: glassmorphism ? 'bg-slate-800/20' : 'bg-slate-800',
      border: 'border-blue-500',
      accent: 'bg-gradient-to-r from-blue-600 to-indigo-700',
      shadow: 'shadow-blue-200',
      text: 'text-white'
    },
    analytics: {
      bg: glassmorphism ? 'bg-gray-800/10' : 'bg-white',
      border: 'border-gray-500',
      accent: 'bg-gradient-to-r from-slate-600 to-gray-700',
      shadow: 'shadow-gray-200',
      text: glassmorphism ? 'text-white' : 'text-gray-800'
    }
  };

  const style = variants[variant];

  const accentPositions = {
    left: 'left-0 top-0 bottom-0 w-1',
    right: 'right-0 top-0 bottom-0 w-1',
    top: 'top-0 left-0 right-0 h-1',
    bottom: 'bottom-0 left-0 right-0 h-1'
  };

  return (
    <div
      className={cn(
        'relative p-6 shadow-lg overflow-hidden',
        style.bg,
        style.text,
        style.shadow,
        glassmorphism && 'backdrop-blur-xl',
        hover && 'hover:shadow-xl transition-shadow duration-300',
        className
      )}
    >
      {/* Accent Bar */}
      <div className={cn('absolute z-10', accentPositions[accent], style.accent)} />

      {/* Content */}
      <div className="relative z-5">
        {children}
      </div>
    </div>
  );
}

// Specialized Metro Cards
export function PancakeCard({ glassmorphism = false, ...props }: Omit<MetroCardProps, 'variant'>) {
  return <MetroCard variant="pancake" glassmorphism={glassmorphism} {...props} />;
}

export function AdminCard({ glassmorphism = true, ...props }: Omit<MetroCardProps, 'variant'>) {
  return <MetroCard variant="admin" glassmorphism={glassmorphism} {...props} />;
}

export function AnalyticsCard({ glassmorphism = false, ...props }: Omit<MetroCardProps, 'variant'>) {
  return <MetroCard variant="analytics" glassmorphism={glassmorphism} {...props} />;
}

// Metro Stats Card
interface MetroStatsCardProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  icon?: string;
  value?: string | number;
  label?: string;
  trend?: 'up' | 'down' | 'neutral';
  percentage?: string;
}

export function MetroStatsCard({
  variant = 'pancake',
  icon,
  value,
  label,
  trend,
  percentage
}: MetroStatsCardProps) {
  const trendIcons = {
    up: '📈',
    down: '📉',
    neutral: '➡'
  };

  const trendColors = {
    up: 'text-green-500',
    down: 'text-red-500',
    neutral: 'text-gray-500'
  };

  return (
    <MetroCard variant={variant} className="min-w-48">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {icon && (
            <div className="text-2xl mb-2">
              {icon}
            </div>
          )}
          
          <div className="text-3xl font-bold mb-1">
            {value}
          </div>
          
          {label && (
            <div className="text-sm opacity-70 font-medium">
              {label}
            </div>
          )}
        </div>

        {trend && percentage && (
          <div className={cn('flex items-center space-x-1', trendColors[trend])}>
            <span>{trendIcons[trend]}</span>
            <span className="text-sm font-medium">{percentage}</span>
          </div>
        )}
      </div>
    </MetroCard>
  );
}

// Metro List Card
interface MetroListItem {
  icon?: string;
  title: string;
  subtitle?: string;
  action?: string;
}

interface MetroListCardProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  items: MetroListItem[];
  title?: string;
}

export function MetroListCard({ variant = 'pancake', items, title }: MetroListCardProps) {
  return (
    <MetroCard variant={variant}>
      {title && (
        <h3 className="font-bold text-lg mb-4 opacity-90">{title}</h3>
      )}
      
      <div className="space-y-3">
        {items.map((item, index) => (
          <div
            key={index}
            className="flex items-center justify-between py-2 border-b border-gray-200/20 last:border-b-0"
          >
            <div className="flex items-center space-x-3">
              {item.icon && (
                <span className="text-xl">{item.icon}</span>
              )}
              <div>
                <div className="font-medium">{item.title}</div>
                {item.subtitle && (
                  <div className="text-sm opacity-70">{item.subtitle}</div>
                )}
              </div>
            </div>
            
            {item.action && (
              <span className="text-sm opacity-70">{item.action}</span>
            )}
          </div>
        ))}
      </div>
    </MetroCard>
  );
}