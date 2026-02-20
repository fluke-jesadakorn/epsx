'use client';

import { usePathname, useRouter } from 'next/navigation';
import { cn } from '@/design-system';
import * as LucideIcons from 'lucide-react';
import React from 'react';

// Type helper for Lucide icon names
type LucideIconName = keyof typeof LucideIcons;

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
  primary: 'from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]',
  success: 'from-[#31d0aa] to-[#1fc7d4]',
  info: 'from-[#1fc7d4] to-[#7645d9]',
  purple: 'from-[#7645d9] to-[#ed4b9e]',
  warning: 'from-[#ffb237] to-[#ffb237]',
  indigo: 'from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]',
  default: 'from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]',
};

const iconColorClasses: Record<GradientPreset, string> = {
  primary: 'text-[#1fc7d4]',
  success: 'text-[#31d0aa]',
  info: 'text-[#1fc7d4]',
  purple: 'text-[#7645d9]',
  warning: 'text-[#ffb237]',
  indigo: 'text-[#7645d9]',
  default: 'text-[#1fc7d4]',
};

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  /** Lucide icon name (as string) - e.g., 'ShieldCheck', 'Users', etc. */
  icon?: LucideIconName;
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

// eslint-disable-next-line complexity
export function PageHeader({
  title,
  subtitle,
  icon: iconName,
  emoji,
  gradient = 'default',
  centered = false,
  actions,
  className,
}: PageHeaderProps) {
  // Dynamically get the icon component from the icon name
  const Icon = iconName ? (LucideIcons[iconName] as React.ComponentType<{ className?: string }>) : null;

  return (
    <div className={cn(
      'mb-6 sm:mb-8',
      centered && 'text-center',
      className
    )}>
      <div className={cn(
        'flex items-start gap-4',
        centered ? 'justify-center' : 'justify-between',
        !centered && actions !== undefined && 'flex-col sm:flex-row sm:items-center'
      )}>
        <div className={cn(centered && 'flex flex-col items-center')}>
          <h1 className={cn(
            'flex items-center gap-3 text-3xl sm:text-4xl lg:text-5xl font-bold',
            centered && 'justify-center'
          )}>
            {Icon !== null && (
              <span className={iconColorClasses[gradient]}>
                <Icon className="w-8 h-8 sm:w-10 sm:h-10 lg:w-12 lg:h-12" />
              </span>
            )}
            {emoji !== undefined && (
              <span className="text-3xl sm:text-4xl">{emoji}</span>
            )}
            <span className={cn(
              'bg-gradient-to-r bg-clip-text text-transparent',
              gradientClasses[gradient]
            )}>
              {title}
            </span>
          </h1>
          {subtitle !== undefined && (
            <p className={cn(
              'text-sm sm:text-base lg:text-lg text-muted-foreground mt-2',
              centered ? 'max-w-2xl' : ''
            )}>
              {subtitle}
            </p>
          )}
        </div>
        {actions !== undefined && (
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
  /** Lucide icon name */
  icon?: LucideIconName;
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
    // Default to a premium purple if no gradient specified, matching the design
    return 'from-[#7645d9] to-[#7645d9]';
  };

  return (
    <div className={cn(
      'bg-white dark:bg-slate-900 backdrop-blur-2xl p-1.5 rounded-[32px] border border-gray-200 dark:border-slate-700 shadow-xl max-w-2xl mx-auto',
      className
    )}>
      <div className="relative flex gap-1 overflow-x-auto no-scrollbar justify-center">
        {tabs.map((tab) => {
          const Icon = tab.icon ? (LucideIcons[tab.icon] as React.ComponentType<{ className?: string }>) : null;
          const isActive = activeTab === tab.id;

          return (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={cn(
                'flex items-center justify-center gap-2 px-8 py-3 rounded-[28px] font-bold text-sm sm:text-base transition-all duration-300 active:scale-95 flex-1 min-w-[120px]',
                isActive
                  ? cn(
                    'text-white shadow-lg shadow-purple-500/20 bg-gradient-to-r',
                    getTabGradient(tab)
                  )
                  : 'text-muted-foreground hover:text-foreground hover:bg-gray-100 dark:hover:bg-white/5'
              )}
            >
              {Icon !== null && <Icon className="w-4 h-4 sm:w-5 sm:h-5" />}
              {tab.prefix !== undefined && Icon === null && <span>{tab.prefix}</span>}
              <span>{tab.label}</span>
            </button>
          );
        })}
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

// eslint-disable-next-line complexity
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
              <div className="w-10 h-10 sm:w-12 sm:h-12 bg-primary rounded-xl" />
              <div className="h-10 sm:h-12 bg-gradient-to-r from-primary to-secondary rounded-2xl w-48 sm:w-64" />
            </div>
            <div className="h-4 sm:h-5 bg-muted rounded-full w-48 sm:w-72 mt-3" />
          </div>
        )}

        {/* Tabs skeleton */}
        {showTabs && (
          <div className="rounded-2xl bg-white dark:bg-white/[0.04] backdrop-blur-xl border border-gray-200 dark:border-slate-700 p-2 animate-pulse">
            <div className={cn(
              'grid gap-2',
              tabCount === 2 && 'grid-cols-2',
              tabCount === 3 && 'grid-cols-3',
              tabCount === 4 && 'grid-cols-4'
            )}>
              {Array.from({ length: tabCount }).map((_, i) => (
                // eslint-disable-next-line react/no-array-index-key
                <div key={i} className="h-12 bg-white dark:bg-white/[0.04] rounded-xl" />
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
                // eslint-disable-next-line react/no-array-index-key
                key={i}
                className="bg-white dark:bg-white/[0.04] backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 border border-gray-200 dark:border-slate-700"
              >
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-orange-500 rounded-xl" />
                  <div className="w-12 h-4 bg-gray-50 dark:bg-white/[0.06] rounded-full" />
                </div>
                <div className="space-y-2">
                  <div className="h-8 bg-gradient-to-r from-purple-500 to-orange-500 rounded-lg w-20" />
                  <div className="h-4 bg-gray-50 dark:bg-white/[0.06] rounded-full w-24" />
                  <div className="h-3 bg-gray-50 dark:bg-white/[0.06] rounded-full w-16" />
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Content skeleton */}
        {rows > 0 && (
          <div className="bg-white dark:bg-white/[0.04] backdrop-blur-xl rounded-2xl sm:rounded-3xl border border-gray-200 dark:border-slate-700 overflow-hidden animate-pulse">
            <div className="p-4 sm:p-6 lg:p-8 space-y-4">
              {Array.from({ length: rows }).map((_, i) => (
                <div
                  // eslint-disable-next-line react/no-array-index-key
                  key={i}
                  className="flex items-center gap-4 p-3 sm:p-4 bg-white dark:bg-white/[0.04] rounded-xl sm:rounded-2xl"
                >
                  <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-orange-500 rounded-xl shrink-0" />
                  <div className="flex-1 space-y-2">
                    <div className="h-4 bg-gray-50 dark:bg-white/[0.06] rounded-lg w-1/3" />
                    <div className="h-3 bg-gray-50 dark:bg-white/[0.06] rounded-lg w-1/2" />
                  </div>
                  <div className="h-8 w-20 bg-gradient-to-r from-purple-500 to-orange-500 rounded-full shrink-0" />
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
  icon?: LucideIconName;
  action?: React.ReactNode;
  className?: string;
}

export function PageEmpty({
  title = 'No data found',
  message = 'There are no items to display',
  emoji = '📭',
  icon: iconName,
  action,
  className,
}: PageEmptyProps) {
  // Dynamically get the icon component from the icon name
  const Icon = iconName ? (LucideIcons[iconName] as React.ComponentType<{ className?: string }>) : null;

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
      {action !== undefined && <div className="mt-6">{action}</div>}
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
      <div className="w-16 h-16 bg-red-500/10 rounded-2xl flex items-center justify-center mb-4">
        <span className="text-3xl">⚠️</span>
      </div>
      <h3 className="text-lg sm:text-xl font-semibold text-foreground mb-2">{title}</h3>
      <p className="text-sm sm:text-base text-muted-foreground max-w-md mb-6">{message}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="px-6 py-3 bg-gradient-to-r from-purple-500 to-orange-500 text-white rounded-xl font-semibold shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30 hover-lift transition-all"
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
  const _router = useRouter();
  const pathname = usePathname();

  return (
    <div className={cn(
      'flex flex-col items-center justify-center py-16 sm:py-24 text-center',
      className
    )}>
      <div className="w-24 h-24 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[32px] flex items-center justify-center mb-8 border border-gray-200 dark:border-slate-700 shadow-xl">
        <span className="text-5xl">🔐</span>
      </div>
      <h3 className="text-3xl sm:text-4xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent mb-4 tracking-tight">{title}</h3>
      <p className="text-lg sm:text-xl text-muted-foreground max-w-md mb-10 font-medium">{message}</p>
      {onAuth ? (
        <button
          onClick={onAuth}
          className="px-10 py-5 bg-[#1fc7d4] text-white rounded-3xl font-bold text-xl shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 hover:scale-[1.02] active:scale-95 transition-all"
        >
          Connect Wallet
        </button>
      ) : (
        <a
          href={`/auth?return_url=${encodeURIComponent(pathname)}`}
          className="px-10 py-5 bg-[#1fc7d4] text-white rounded-3xl font-bold text-xl shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 hover:scale-[1.02] active:scale-95 transition-all"
        >
          Connect Wallet
        </a>
      )}
    </div>
  );
}

export default PageLayout;
