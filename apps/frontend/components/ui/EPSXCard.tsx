'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface EPSXCardProps {
  children: ReactNode;
  variant?: 'default' | 'analytics' | 'premium' | 'glass';
  size?: 'sm' | 'md' | 'lg';
  hover?: boolean;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  className?: string;
}

interface EPSXCardHeaderProps {
  children: ReactNode;
  className?: string;
}

interface EPSXCardContentProps {
  children: ReactNode;
  className?: string;
}

interface EPSXCardFooterProps {
  children: ReactNode;
  className?: string;
}

export function EPSXCard({
  children,
  variant = 'default',
  size = 'md',
  hover = false,
  padding = 'md',
  className
}: EPSXCardProps) {
  const baseClasses = 'rounded-lg border transition-all duration-200';
  
  const variants = {
    default: 'bg-white border-gray-200 shadow-sm',
    analytics: 'bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200 shadow-sm',
    premium: 'bg-gradient-to-br from-purple-50 to-pink-50 border-purple-200 shadow-sm',
    glass: 'bg-white/80 backdrop-blur-sm border-gray-200/50 shadow-lg'
  };

  const sizes = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg'
  };

  const paddings = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8'
  };

  const hoverClasses = hover ? 'hover:shadow-lg hover:-translate-y-1' : '';

  return (
    <div
      className={cn(
        baseClasses,
        variants[variant],
        sizes[size],
        paddings[padding],
        hoverClasses,
        className
      )}
    >
      {children}
    </div>
  );
}

export function EPSXCardHeader({ children, className }: EPSXCardHeaderProps) {
  return (
    <div className={cn('pb-4 border-b border-gray-200', className)}>
      {children}
    </div>
  );
}

export function EPSXCardContent({ children, className }: EPSXCardContentProps) {
  return (
    <div className={cn('py-4', className)}>
      {children}
    </div>
  );
}

export function EPSXCardFooter({ children, className }: EPSXCardFooterProps) {
  return (
    <div className={cn('pt-4 border-t border-gray-200', className)}>
      {children}
    </div>
  );
}