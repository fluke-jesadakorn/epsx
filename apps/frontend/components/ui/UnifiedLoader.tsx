'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

type LoaderVariant = 'default' | 'pancake' | 'admin' | 'analytics' | 'premium' | 'white';
type LoaderSize = 'sm' | 'md' | 'lg';
type LoaderType = 'spinner' | 'dots' | 'bars' | 'pulse' | 'stack';

interface UnifiedLoaderProps {
  variant?: LoaderVariant;
  size?: LoaderSize;
  type?: LoaderType;
  message?: string;
  children?: ReactNode;
  className?: string;
}

export function UnifiedLoader({
  variant = 'default',
  size = 'md',
  type = 'spinner',
  message,
  children,
  className
}: UnifiedLoaderProps) {
  const variants = {
    default: {
      primary: 'border-blue-500',
      secondary: 'border-blue-200',
      accent: 'bg-blue-500',
      text: 'text-blue-600',
      icon: '⚡'
    },
    pancake: {
      primary: 'border-orange-500',
      secondary: 'border-orange-200', 
      accent: 'bg-gradient-to-r from-orange-400 to-yellow-500',
      text: 'text-orange-600',
      icon: '🥞'
    },
    admin: {
      primary: 'border-blue-600',
      secondary: 'border-blue-200',
      accent: 'bg-gradient-to-r from-blue-600 to-indigo-700',
      text: 'text-blue-600',
      icon: '⚡'
    },
    analytics: {
      primary: 'border-indigo-500',
      secondary: 'border-indigo-200',
      accent: 'bg-gradient-to-r from-indigo-500 to-purple-600',
      text: 'text-indigo-600',
      icon: '📊'
    },
    premium: {
      primary: 'border-purple-500',
      secondary: 'border-purple-200',
      accent: 'bg-gradient-to-r from-purple-500 to-pink-600',
      text: 'text-purple-600',
      icon: '💎'
    },
    white: {
      primary: 'border-white',
      secondary: 'border-white/50',
      accent: 'bg-white',
      text: 'text-white',
      icon: '⚡'
    }
  };

  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8'
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
    <div className="flex gap-1">
      {Array.from({ length: 3 }).map((_, i) => (
        <div
          key={i}
          className={cn(
            'rounded-full animate-pulse',
            size === 'sm' ? 'w-1 h-1' : size === 'md' ? 'w-1.5 h-1.5' : 'w-2 h-2',
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
    <div className="flex gap-1 items-end">
      {Array.from({ length: 4 }).map((_, i) => (
        <div
          key={i}
          className={cn(
            'animate-pulse',
            size === 'sm' ? 'w-1 h-3' : size === 'md' ? 'w-1.5 h-4' : 'w-2 h-6',
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
          'rounded-full animate-ping absolute opacity-75',
          style.accent
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

  const StackLoader = () => (
    <div className="relative">
      {/* Bottom Layer */}
      <div className={cn(sizeClasses[size], style.accent, 'rounded-full relative z-10')} />
      
      {/* Middle Layer */}
      <div className={cn(sizeClasses[size], style.secondary, 'rounded-full absolute top-0 left-0 z-20')} />

      {/* Top Layer with Icon */}
      <div className={cn(
        sizeClasses[size], 
        style.accent,
        'rounded-full absolute top-0 left-0 z-30 flex items-center justify-center text-white'
      )}>
        <span className={size === 'sm' ? 'text-xs' : size === 'md' ? 'text-sm' : 'text-base'}>
          {style.icon}
        </span>
      </div>
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
      case 'stack':
        return <StackLoader />;
      default:
        return <SpinnerLoader />;
    }
  };

  return (
    <div className={cn('flex flex-col items-center justify-center gap-3', className)}>
      {renderLoader()}
      
      {/* Progress Dots for stack type */}
      {type === 'stack' && (
        <div className="flex space-x-2">
          {Array.from({ length: 3 }).map((_, i) => (
            <div
              key={i}
              className={cn('w-2 h-2 rounded-none', style.accent)}
            />
          ))}
        </div>
      )}

      {message && (
        <p className={cn('text-sm font-medium animate-pulse', style.text)}>
          {message}
        </p>
      )}
      
      {children}
    </div>
  );
}

// Progress Bar
interface UnifiedProgressBarProps {
  progress?: number;
  variant?: LoaderVariant;
  animated?: boolean;
  showPercentage?: boolean;
  className?: string;
}

export function UnifiedProgressBar({ 
  progress = 0, 
  variant = 'default',
  animated = true,
  showPercentage = false,
  className
}: UnifiedProgressBarProps) {
  const variants = {
    default: 'bg-gradient-to-r from-blue-500 to-blue-600',
    pancake: 'bg-gradient-to-r from-orange-400 to-yellow-500',
    admin: 'bg-gradient-to-r from-blue-600 to-indigo-700',
    analytics: 'bg-gradient-to-r from-indigo-500 to-purple-600',
    premium: 'bg-gradient-to-r from-purple-500 to-pink-600',
    white: 'bg-white'
  };

  const textVariants = {
    default: 'text-blue-600',
    pancake: 'text-orange-600',
    admin: 'text-blue-600',
    analytics: 'text-indigo-600',
    premium: 'text-purple-600',
    white: 'text-white'
  };

  return (
    <div className={cn('w-full', className)}>
      <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden relative">
        <div
          className={cn(
            'h-full rounded-full',
            variants[variant]
          )}
          style={{ 
            width: `${Math.min(Math.max(progress, 0), 100)}%`,
            transition: 'width 0.5s ease-out'
          }}
        />
      </div>
      
      {showPercentage && (
        <div className={cn('text-sm font-medium mt-2', textVariants[variant])}>
          {Math.round(progress)}%
        </div>
      )}
    </div>
  );
}

// Skeleton
interface UnifiedSkeletonProps {
  variant?: 'text' | 'card' | 'avatar' | 'button';
  lines?: number;
  className?: string;
}

export function UnifiedSkeleton({
  variant = 'text',
  lines = 1,
  className
}: UnifiedSkeletonProps) {
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

// Full Loading States
interface UnifiedLoadingProps {
  type?: 'page' | 'section' | 'inline';
  variant?: LoaderVariant;
  message?: string;
  children?: ReactNode;
  className?: string;
}

export function UnifiedLoading({
  type = 'inline',
  variant = 'default',
  message = 'Loading...',
  children,
  className
}: UnifiedLoadingProps) {
  if (type === 'page') {
    return (
      <div className={cn(
        'min-h-screen flex items-center justify-center bg-gray-50',
        className
      )}>
        <div className="text-center">
          <UnifiedLoader variant={variant} size="lg" type="spinner" />
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
          <UnifiedLoader variant={variant} size="md" type="dots" />
          <p className="mt-3 text-sm text-gray-600">{message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={cn('flex items-center justify-center py-4', className)}>
      <UnifiedLoader variant={variant} size="sm" message={message} />
      {children}
    </div>
  );
}

// Legacy component aliases for backward compatibility
export const PancakeSwapLoader = ({ variant = 'pancake' as const, ...props }: Omit<UnifiedLoaderProps, 'variant'> & { variant?: 'pancake' | 'admin' | 'analytics' }) => 
  <UnifiedLoader variant={variant} type="stack" {...props} />;

export const EPSXLoader = UnifiedLoader;
export const ProfessionalLoader = UnifiedLoader;
export const MetroProgressBar = UnifiedProgressBar;
export const ProfessionalProgressBar = UnifiedProgressBar;
export const PancakeFlip = ({ variant = 'pancake' as const, size = 'md' as const }: { variant?: 'pancake' | 'admin' | 'analytics', size?: LoaderSize }) => 
  <UnifiedLoader variant={variant} size={size} type="stack" />;
export const ProfessionalSkeleton = UnifiedSkeleton;
export const ProfessionalLoading = UnifiedLoading;