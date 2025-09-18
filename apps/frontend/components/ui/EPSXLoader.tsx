'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface EPSXLoaderProps {
  size?: 'sm' | 'md' | 'lg';
  variant?: 'spinner' | 'dots' | 'bars';
  color?: 'primary' | 'secondary' | 'white';
  text?: string;
  children?: ReactNode;
  className?: string;
}

export function EPSXLoader({
  size = 'md',
  variant = 'spinner',
  color = 'primary',
  text,
  children,
  className
}: EPSXLoaderProps) {
  const sizes = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8'
  };

  const colors = {
    primary: 'text-blue-500',
    secondary: 'text-gray-500',
    white: 'text-white'
  };

  const SpinnerLoader = () => (
    <div
      className={cn(
        'border-2 border-current border-t-transparent rounded-full animate-spin',
        sizes[size],
        colors[color]
      )}
    />
  );

  const DotsLoader = () => (
    <div className="flex gap-1">
      {[0, 1, 2].map((i) => (
        <div
          key={i}
          className={cn(
            'rounded-full animate-pulse',
            size === 'sm' ? 'w-1 h-1' : size === 'md' ? 'w-1.5 h-1.5' : 'w-2 h-2',
            colors[color] === 'text-white' ? 'bg-white' : 'bg-current'
          )}
          style={{
            animationDelay: `${i * 0.2}s`,
            animationDuration: '1s'
          }}
        />
      ))}
    </div>
  );

  const BarsLoader = () => (
    <div className="flex gap-1 items-end">
      {[0, 1, 2].map((i) => (
        <div
          key={i}
          className={cn(
            'animate-pulse',
            size === 'sm' ? 'w-1 h-3' : size === 'md' ? 'w-1.5 h-4' : 'w-2 h-6',
            colors[color] === 'text-white' ? 'bg-white' : 'bg-current'
          )}
          style={{
            animationDelay: `${i * 0.15}s`,
            animationDuration: '0.8s'
          }}
        />
      ))}
    </div>
  );

  const renderLoader = () => {
    switch (variant) {
      case 'dots':
        return <DotsLoader />;
      case 'bars':
        return <BarsLoader />;
      default:
        return <SpinnerLoader />;
    }
  };

  return (
    <div className={cn('flex flex-col items-center justify-center gap-3', className)}>
      {renderLoader()}
      {text && (
        <p className={cn('text-sm font-medium animate-pulse', colors[color])}>
          {text}
        </p>
      )}
      {children}
    </div>
  );
}