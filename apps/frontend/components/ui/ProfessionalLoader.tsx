'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ProfessionalLoaderProps {
  variant?: 'default' | 'analytics' | 'premium';
  size?: 'sm' | 'md' | 'lg';
  type?: 'spinner' | 'dots' | 'bars' | 'pulse';
  message?: string;
  className?: string;
}

export function ProfessionalLoader({ 
  variant = 'default', 
  size = 'md',
  type = 'spinner',
  message,
  className
}: ProfessionalLoaderProps) {
  const sizeClasses = {
    sm: 'w-6 h-6',
    md: 'w-8 h-8', 
    lg: 'w-12 h-12'
  };

  const variants = {
    default: {
      primary: 'border-blue-500',
      secondary: 'border-blue-200',
      accent: 'bg-blue-500',
      text: 'text-blue-600'
    },
    analytics: {
      primary: 'border-indigo-500',
      secondary: 'border-indigo-200',
      accent: 'bg-indigo-500',
      text: 'text-indigo-600'
    },
    premium: {
      primary: 'border-purple-500',
      secondary: 'border-purple-200',
      accent: 'bg-purple-500',
      text: 'text-purple-600'
    }
  };

  const style = variants[variant];

  const SpinnerLoader = () => (
    <div
      className={cn(
        sizeClasses[size],
        'border-2 border-t-transparent rounded-full animate-spin',
        style.primary
      )}
    />
  );

  const DotsLoader = () => (
    <div className="flex space-x-1">
      {Array.from({ length: 3 }).map((_, i) => (
        <div
          key={i}
          className={cn(
            'rounded-full animate-pulse',
            size === 'sm' ? 'w-1.5 h-1.5' : size === 'md' ? 'w-2 h-2' : 'w-3 h-3',
            style.accent
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
    <div className="flex space-x-1 items-end">
      {Array.from({ length: 4 }).map((_, i) => (
        <div
          key={i}
          className={cn(
            'animate-pulse',
            size === 'sm' ? 'w-1 h-4' : size === 'md' ? 'w-1.5 h-6' : 'w-2 h-8',
            style.accent
          )}
          style={{
            animationDelay: `${i * 0.15}s`,
            animationDuration: '0.8s'
          }}
        />
      ))}
    </div>
  );

  const PulseLoader = () => (
    <div className="relative">
      <div
        className={cn(
          sizeClasses[size],
          'rounded-full animate-ping absolute',
          style.accent,
          'opacity-75'
        )}
      />
      <div
        className={cn(
          sizeClasses[size],
          'rounded-full relative',
          style.accent
        )}
      />
    </div>
  );

  const renderLoader = () => {
    switch (type) {
      case 'dots':
        return <DotsLoader />;
      case 'bars':
        return <BarsLoader />;
      case 'pulse':
        return <PulseLoader />;
      default:
        return <SpinnerLoader />;
    }
  };

  return (
    <div className={cn('flex flex-col items-center justify-center space-y-4', className)}>
      {renderLoader()}
      
      {message && (
        <p className={cn('text-sm font-medium animate-pulse', style.text)}>
          {message}
        </p>
      )}
    </div>
  );
}

// Professional Progress Bar
interface ProfessionalProgressBarProps {
  progress?: number;
  variant?: 'default' | 'analytics' | 'premium';
  animated?: boolean;
  showPercentage?: boolean;
  className?: string;
}

export function ProfessionalProgressBar({ 
  progress = 0, 
  variant = 'default',
  animated = true,
  showPercentage = false,
  className
}: ProfessionalProgressBarProps) {
  const variants = {
    default: 'bg-gradient-to-r from-blue-500 to-blue-600',
    analytics: 'bg-gradient-to-r from-indigo-500 to-purple-600',
    premium: 'bg-gradient-to-r from-purple-500 to-pink-600'
  };

  const textVariants = {
    default: 'text-blue-600',
    analytics: 'text-indigo-600',
    premium: 'text-purple-600'
  };

  return (
    <div className={cn('w-full', className)}>
      <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
        <div
          className={cn(
            'h-full rounded-full transition-all duration-500 ease-out',
            variants[variant]
          )}
          style={{ width: `${Math.min(Math.max(progress, 0), 100)}%` }}
        />
        
        {animated && (
          <div
            className={cn(
              'h-full absolute top-0 opacity-30 rounded-full',
              variants[variant]
            )}
            style={{
              animation: 'shimmer 2s infinite linear',
              width: '20%',
              left: '-20%'
            }}
          />
        )}
      </div>
      
      {showPercentage && (
        <div className={cn('text-sm font-medium mt-2', textVariants[variant])}>
          {Math.round(progress)}%
        </div>
      )}
      
      <style jsx>{`
        @keyframes shimmer {
          0% { transform: translateX(0); }
          100% { transform: translateX(500%); }
        }
      `}</style>
    </div>
  );
}

// Professional Loading Skeleton
interface ProfessionalSkeletonProps {
  variant?: 'text' | 'card' | 'avatar' | 'button';
  lines?: number;
  className?: string;
}

export function ProfessionalSkeleton({
  variant = 'text',
  lines = 1,
  className
}: ProfessionalSkeletonProps) {
  const variants = {
    text: 'h-4 bg-gray-200 rounded',
    card: 'h-32 bg-gray-200 rounded-lg',
    avatar: 'w-10 h-10 bg-gray-200 rounded-full',
    button: 'h-10 bg-gray-200 rounded-md'
  };

  if (variant === 'text' && lines > 1) {
    return (
      <div className={cn('space-y-2', className)}>
        {Array.from({ length: lines }).map((_, i) => (
          <div
            key={i}
            className={cn(
              variants.text,
              'animate-pulse',
              i === lines - 1 ? 'w-3/4' : 'w-full'
            )}
          />
        ))}
      </div>
    );
  }

  return (
    <div className={cn(variants[variant], 'animate-pulse', className)} />
  );
}

// All-in-one Professional Loading Component
interface ProfessionalLoadingProps {
  type?: 'page' | 'section' | 'inline';
  variant?: 'default' | 'analytics' | 'premium';
  message?: string;
  children?: ReactNode;
  className?: string;
}

export function ProfessionalLoading({
  type = 'inline',
  variant = 'default',
  message = 'Loading...',
  children,
  className
}: ProfessionalLoadingProps) {
  if (type === 'page') {
    return (
      <div className={cn(
        'min-h-screen flex items-center justify-center bg-gray-50',
        className
      )}>
        <div className="text-center">
          <ProfessionalLoader variant={variant} size="lg" type="spinner" />
          <h2 className="mt-4 text-lg font-semibold text-gray-900">
            Loading EPSX Analytics
          </h2>
          <p className="mt-2 text-gray-600">{message}</p>
        </div>
      </div>
    );
  }

  if (type === 'section') {
    return (
      <div className={cn(
        'flex items-center justify-center py-12',
        className
      )}>
        <div className="text-center">
          <ProfessionalLoader variant={variant} size="md" type="dots" />
          <p className="mt-3 text-sm text-gray-600">{message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={cn('flex items-center justify-center py-4', className)}>
      <ProfessionalLoader variant={variant} size="sm" message={message} />
      {children}
    </div>
  );
}