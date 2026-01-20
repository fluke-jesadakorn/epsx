'use client';

import { cn } from '@/design-system';
import { LucideIcon } from 'lucide-react';
import React from 'react';

/**
 * Unified Page Layout Component
 * Provides consistent padding, container width, and spacing across all admin pages
 */

interface PageLayoutProps {
  children: React.ReactNode;
  className?: string;
  /** Maximum width - defaults to 7xl */
  maxWidth?: 'full' | '7xl' | '6xl' | '5xl' | '4xl';
}

const maxWidthClasses = {
  full: '',
  '7xl': 'max-w-7xl mx-auto',
  '6xl': 'max-w-6xl mx-auto',
  '5xl': 'max-w-5xl mx-auto',
  '4xl': 'max-w-4xl mx-auto',
};

export function PageLayout({ children, className, maxWidth = '7xl' }: PageLayoutProps) {
  return (
    <div className={cn('p-3 sm:p-6 lg:p-8', className)}>
      <div className={cn(maxWidthClasses[maxWidth], 'space-y-6')}>
        {children}
      </div>
    </div>
  );
}

/**
 * Unified Page Header Component
 * Consistent header styling with gradient text, icons, and subtitles
 */

type GradientPreset =
  | 'primary'      // Yellow/Orange (PancakeSwap style)
  | 'success'      // Green/Teal
  | 'info'         // Blue/Purple
  | 'purple'       // Purple/Pink
  | 'warning'      // Orange/Red
  | 'indigo'       // Indigo/Purple/Pink
  | 'default';     // Primary to secondary

const gradientClasses: Record<GradientPreset, string> = {
  primary: 'from-primary via-secondary to-primary',
  success: 'from-emerald-500 via-teal-500 to-cyan-500',
  info: 'from-blue-500 via-indigo-500 to-purple-500',
  purple: 'from-purple-500 via-fuchsia-500 to-pink-500',
  warning: 'from-amber-500 via-orange-500 to-red-500',
  indigo: 'from-indigo-500 via-purple-500 to-pink-500',
  default: 'from-primary via-secondary to-primary',
};

const iconColorClasses: Record<GradientPreset, string> = {
  primary: 'text-primary',
  success: 'text-emerald-500',
  info: 'text-blue-500',
  purple: 'text-purple-500',
  warning: 'text-amber-500',
  indigo: 'text-indigo-500',
  default: 'text-primary',
};

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  /** Lucide icon component */
  icon?: LucideIcon;
  /** Emoji alternative to icon */
  emoji?: string;
  /** Gradient color preset */
  gradient?: GradientPreset;
  /** Whether to center the header */
  centered?: boolean;
  /** Additional content (e.g., action buttons) */
  actions?: React.ReactNode;
  className?: string;
}

export function PageHeader({
  title,
  subtitle,
  icon: Icon,
  emoji,
  gradient = 'default',
  centered = false,
  actions,
  className,
}: PageHeaderProps) {
  return (
    <div className={cn(
      'mb-6 sm:mb-8',
      centered && 'text-center',
      className
    )}>
      <div className={cn(
        'flex items-start gap-4',
        centered ? 'justify-center' : 'justify-between',
        !centered && actions && 'flex-col sm:flex-row sm:items-center'
      )}>
        <div className={cn(centered && 'flex flex-col items-center')}>
          <h1 className={cn(
            'flex items-center gap-3 text-3xl sm:text-4xl lg:text-5xl font-bold',
            centered && 'justify-center'
          )}>
            {Icon && (
              <span className={iconColorClasses[gradient]}>
                <Icon className="w-8 h-8 sm:w-10 sm:h-10 lg:w-12 lg:h-12" />
              </span>
            )}
            {emoji && (
              <span className="text-3xl sm:text-4xl">{emoji}</span>
            )}
            <span className={cn(
              'bg-gradient-to-r bg-clip-text text-transparent',
              gradientClasses[gradient]
            )}>
              {title}
            </span>
          </h1>
          {subtitle && (
            <p className={cn(
              'text-sm sm:text-base lg:text-lg text-muted-foreground mt-2',
              centered && 'max-w-2xl'
            )}>
              {subtitle}
            </p>
          )}
        </div>
        {actions && (
          <div className="flex items-center gap-2 shrink-0">
            {actions}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Unified Page Tabs Component
 * Consistent tab navigation with gradient backgrounds
 */

export interface TabItem {
  id: string;
  label: string;
  /** Emoji or icon prefix */
  prefix?: string;
  /** Custom gradient for active state */
  gradient?: GradientPreset;
}

interface PageTabsProps {
  tabs: TabItem[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
  className?: string;
}

export function PageTabs({ tabs, activeTab, onTabChange, className }: PageTabsProps) {
  const getTabGradient = (tab: TabItem): string => {
    if (tab.gradient) {
      return gradientClasses[tab.gradient];
    }
    // Alternate between info and purple for visual variety
    const index = tabs.findIndex(t => t.id === tab.id);
    return index % 2 === 0 ? gradientClasses.info : gradientClasses.purple;
  };

  return (
    <div className={cn(
      'relative overflow-hidden rounded-2xl bg-gradient-to-r from-primary/10 via-secondary/10 to-primary/10 p-0.5',
      className
    )}>
      <div className="relative bg-card rounded-2xl p-2">
        <div className={cn(
          'grid gap-2',
          tabs.length === 2 && 'grid-cols-2',
          tabs.length === 3 && 'grid-cols-1 sm:grid-cols-3',
          tabs.length === 4 && 'grid-cols-2 sm:grid-cols-4',
          tabs.length > 4 && 'grid-cols-2 sm:grid-cols-3 lg:grid-cols-4'
        )}>
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={cn(
                'px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all duration-200',
                activeTab === tab.id
                  ? cn('bg-gradient-to-r text-white shadow-lg', getTabGradient(tab))
                  : 'bg-card/80 text-foreground hover:bg-muted'
              )}
            >
              {tab.prefix && <span className="mr-1">{tab.prefix}</span>}
              {tab.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

/**
 * Unified Page Skeleton Component
 * Consistent loading state across all pages
 */

interface PageSkeletonProps {
  /** Number of stat cards to show */
  stats?: number;
  /** Number of content rows to show */
  rows?: number;
  /** Show header skeleton */
  showHeader?: boolean;
  /** Show tabs skeleton */
  showTabs?: boolean;
  /** Number of tabs */
  tabCount?: number;
  className?: string;
}

export function PageSkeleton({
  stats = 4,
  rows = 6,
  showHeader = true,
  showTabs = false,
  tabCount = 4,
  className,
}: PageSkeletonProps) {
  return (
    <div className={cn('p-3 sm:p-6 lg:p-8', className)}>
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header skeleton */}
        {showHeader && (
          <div className="mb-6 sm:mb-8 animate-pulse">
            <div className="flex items-center gap-3 mb-2">
              <div className="w-10 h-10 sm:w-12 sm:h-12 bg-primary/20 rounded-xl" />
              <div className="h-10 sm:h-12 bg-gradient-to-r from-primary/20 to-secondary/20 rounded-2xl w-48 sm:w-64" />
            </div>
            <div className="h-4 sm:h-5 bg-muted rounded-full w-48 sm:w-72 mt-3" />
          </div>
        )}

        {/* Tabs skeleton */}
        {showTabs && (
          <div className="rounded-2xl bg-muted/30 p-2 animate-pulse">
            <div className={cn(
              'grid gap-2',
              tabCount === 2 && 'grid-cols-2',
              tabCount === 3 && 'grid-cols-3',
              tabCount === 4 && 'grid-cols-4'
            )}>
              {Array.from({ length: tabCount }).map((_, i) => (
                <div key={i} className="h-12 bg-muted rounded-xl" />
              ))}
            </div>
          </div>
        )}

        {/* Stats grid skeleton */}
        {stats > 0 && (
          <div className={cn(
            'grid gap-4 sm:gap-6 animate-pulse',
            stats <= 2 && 'grid-cols-1 sm:grid-cols-2',
            stats === 3 && 'grid-cols-1 sm:grid-cols-3',
            stats >= 4 && 'grid-cols-2 lg:grid-cols-4'
          )}>
            {Array.from({ length: stats }).map((_, i) => (
              <div
                key={i}
                className="bg-card rounded-2xl sm:rounded-3xl p-4 sm:p-6 border-2 border-primary/10"
              >
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="w-10 h-10 bg-primary/10 rounded-xl" />
                  <div className="w-12 h-4 bg-muted rounded-full" />
                </div>
                <div className="space-y-2">
                  <div className="h-8 bg-primary/20 rounded-lg w-20" />
                  <div className="h-4 bg-muted rounded-full w-24" />
                  <div className="h-3 bg-muted/60 rounded-full w-16" />
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Content skeleton */}
        {rows > 0 && (
          <div className="bg-card rounded-2xl sm:rounded-3xl border border-border/30 overflow-hidden animate-pulse">
            <div className="p-4 sm:p-6 lg:p-8 space-y-4">
              {Array.from({ length: rows }).map((_, i) => (
                <div
                  key={i}
                  className="flex items-center gap-4 p-3 sm:p-4 bg-muted/30 rounded-xl sm:rounded-2xl"
                >
                  <div className="w-10 h-10 bg-primary/10 rounded-xl shrink-0" />
                  <div className="flex-1 space-y-2">
                    <div className="h-4 bg-muted rounded-lg w-1/3" />
                    <div className="h-3 bg-muted/60 rounded-lg w-1/2" />
                  </div>
                  <div className="h-8 w-20 bg-primary/10 rounded-full shrink-0" />
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Unified Empty State Component
 */

interface PageEmptyProps {
  title?: string;
  message?: string;
  emoji?: string;
  icon?: LucideIcon;
  action?: React.ReactNode;
  className?: string;
}

export function PageEmpty({
  title = 'No data found',
  message = 'There are no items to display',
  emoji = '📭',
  icon: Icon,
  action,
  className,
}: PageEmptyProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center py-12 sm:py-16 text-center',
      className
    )}>
      {Icon ? (
        <Icon className="w-16 h-16 text-muted-foreground/50 mb-4" />
      ) : (
        <div className="text-5xl sm:text-6xl mb-4">{emoji}</div>
      )}
      <h3 className="text-lg sm:text-xl font-semibold text-foreground mb-2">{title}</h3>
      <p className="text-sm sm:text-base text-muted-foreground max-w-md">{message}</p>
      {action && <div className="mt-6">{action}</div>}
    </div>
  );
}

/**
 * Unified Error State Component
 */

interface PageErrorProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
  className?: string;
}

export function PageError({
  title = 'Something went wrong',
  message = 'An error occurred while loading the data',
  onRetry,
  className,
}: PageErrorProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center py-12 sm:py-16 text-center',
      className
    )}>
      <div className="w-16 h-16 bg-destructive/10 rounded-2xl flex items-center justify-center mb-4">
        <span className="text-3xl">⚠️</span>
      </div>
      <h3 className="text-lg sm:text-xl font-semibold text-foreground mb-2">{title}</h3>
      <p className="text-sm sm:text-base text-muted-foreground max-w-md mb-6">{message}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="px-6 py-3 bg-primary text-primary-foreground rounded-xl font-semibold hover:opacity-90 transition-opacity"
        >
          Try Again
        </button>
      )}
    </div>
  );
}

/**
 * Unified Auth Required State
 */

interface PageAuthRequiredProps {
  title?: string;
  message?: string;
  onAuth?: () => void;
  className?: string;
}

export function PageAuthRequired({
  title = 'Authentication Required',
  message = 'Please connect your wallet to access this page.',
  onAuth,
  className,
}: PageAuthRequiredProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center py-16 sm:py-24 text-center',
      className
    )}>
      <div className="w-20 h-20 bg-primary/10 rounded-3xl flex items-center justify-center mb-6">
        <span className="text-4xl">🔐</span>
      </div>
      <h3 className="text-2xl sm:text-3xl font-bold text-foreground mb-3">{title}</h3>
      <p className="text-base sm:text-lg text-muted-foreground max-w-md mb-8">{message}</p>
      {onAuth ? (
        <button
          onClick={onAuth}
          className="px-8 py-4 bg-primary text-primary-foreground rounded-2xl font-bold text-lg hover:opacity-90 transition-opacity"
        >
          Connect Wallet
        </button>
      ) : (
        <a
          href="/auth"
          className="px-8 py-4 bg-primary text-primary-foreground rounded-2xl font-bold text-lg hover:opacity-90 transition-opacity"
        >
          Connect Wallet
        </a>
      )}
    </div>
  );
}

export default PageLayout;
