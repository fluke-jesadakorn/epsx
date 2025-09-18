'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface WindowsPhoneTileProps {
  size?: 'small' | 'medium' | 'large' | 'wide';
  variant?: 'pancake' | 'admin' | 'analytics';
  children?: ReactNode;
  icon?: string;
  badge?: string | number;
  onClick?: () => void;
  animate?: boolean;
  delay?: number;
}

export function WindowsPhoneTile({
  size = 'medium',
  variant = 'pancake',
  children,
  icon,
  badge,
  onClick,
  animate = true,
  delay = 0
}: WindowsPhoneTileProps) {
  const sizeClasses = {
    small: 'w-16 h-16',
    medium: 'w-32 h-32',
    large: 'w-32 h-64',
    wide: 'w-64 h-32'
  };

  const variantStyles = {
    pancake: {
      bg: 'bg-gradient-to-br from-orange-400 via-yellow-500 to-orange-600',
      accent: 'bg-orange-600',
      text: 'text-white'
    },
    admin: {
      bg: 'bg-gradient-to-br from-blue-600 via-indigo-600 to-blue-800',
      accent: 'bg-blue-800',
      text: 'text-white'
    },
    analytics: {
      bg: 'bg-gradient-to-br from-slate-600 via-gray-700 to-slate-800',
      accent: 'bg-slate-800',
      text: 'text-white'
    }
  };

  const style = variantStyles[variant];

  return (
    <div
      className={cn(
        sizeClasses[size],
        style.bg,
        style.text,
        'relative overflow-hidden cursor-pointer shadow-lg hover:shadow-xl transition-shadow duration-300 group',
        onClick && 'cursor-pointer'
      )}
      onClick={onClick}
    >
      {/* Badge */}
      {badge && (
        <div className={cn(
          'absolute -top-2 -right-2 text-white rounded-full w-6 h-6 flex items-center justify-center text-xs font-bold z-10',
          style.accent
        )}>
          {badge}
        </div>
      )}

      {/* Content */}
      <div className="relative z-5 h-full flex flex-col p-3">
        {/* Icon */}
        {icon && (
          <div className="text-2xl mb-2">
            {icon}
          </div>
        )}

        {/* Children Content */}
        <div className="flex-1 flex flex-col justify-center">
          {children}
        </div>

        {/* Bottom accent line */}
        <div className={cn('absolute bottom-0 left-0 h-1 w-full', style.accent)} />
      </div>

      {/* Hover effect */}
      <div className="absolute inset-0 bg-white opacity-0 group-hover:opacity-10 transition-opacity duration-300" />
    </div>
  );
}

// Metro Dashboard Grid Component
interface MetroDashboardProps {
  children: ReactNode;
  className?: string;
}

export function MetroDashboard({ children, className = '' }: MetroDashboardProps) {
  return (
    <div className={cn('grid gap-4 p-4 grid-cols-[repeat(auto-fit,minmax(128px,1fr))]', className)}>
      {children}
    </div>
  );
}

// Pancake-themed tiles
export function PancakeTile({ size = 'medium', ...props }: Omit<WindowsPhoneTileProps, 'variant'>) {
  return <WindowsPhoneTile variant="pancake" size={size} {...props} />;
}

export function AdminTile({ size = 'medium', ...props }: Omit<WindowsPhoneTileProps, 'variant'>) {
  return <WindowsPhoneTile variant="admin" size={size} {...props} />;
}

export function AnalyticsTile({ size = 'medium', ...props }: Omit<WindowsPhoneTileProps, 'variant'>) {
  return <WindowsPhoneTile variant="analytics" size={size} {...props} />;
}