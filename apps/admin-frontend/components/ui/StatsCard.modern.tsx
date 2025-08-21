/**
 * Modernized StatsCard Component
 * 
 * This is a demonstration of how the StatsCard component looks when
 * migrated to use the new design system. This replaces hardcoded colors
 * and legacy classes with type-safe design system variants.
 * 
 * Key improvements:
 * - Type-safe component variants using CVA
 * - Semantic color system instead of hardcoded values
 * - Consistent sizing and spacing from design tokens
 * - Better accessibility and theming support
 */

import { LucideIcon } from 'lucide-react';
import { adminCardVariants, adminBadgeVariants, type AdminCardVariants } from '@/design-system';
import { cn } from '@/design-system';

export interface ModernStatsCardProps {
  title: string;
  value: string | number;
  description?: string;
  icon: LucideIcon;
  variant?: AdminCardVariants['variant'];
  size?: 'sm' | 'default' | 'lg';
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  className?: string;
}

export function ModernStatsCard({
  title,
  value,
  description,
  icon: Icon,
  variant = 'default',
  size = 'default',
  trend,
  trendValue,
  className,
}: ModernStatsCardProps) {
  
  // Use design system variants instead of hardcoded classes
  const cardClasses = adminCardVariants({
    variant,
    hover: 'both',
    padding: size === 'sm' ? 'sm' : size === 'lg' ? 'lg' : 'default',
    interactive: false,
  });

  // Semantic color mapping based on variant
  const iconColorMap = {
    default: 'text-primary-600',
    pancake: 'text-orange-600', 
    user: 'text-blue-600',
    permission: 'text-purple-600',
    billing: 'text-green-600',
    analytics: 'text-indigo-600',
    warning: 'text-amber-600',
    error: 'text-red-600',
  };

  const iconBgMap = {
    default: 'bg-primary-100',
    pancake: 'bg-orange-100',
    user: 'bg-blue-100', 
    permission: 'bg-purple-100',
    billing: 'bg-green-100',
    analytics: 'bg-indigo-100',
    warning: 'bg-amber-100',
    error: 'bg-red-100',
  };

  const trendColorMap = {
    up: 'success',
    down: 'error', 
    neutral: 'default',
  } as const;

  return (
    <div className={cn(cardClasses, 'group', className)}>
      {/* Decorative background element */}
      <div className="absolute top-0 right-0 w-20 h-20 -mr-8 -mt-8 rounded-full bg-gradient-to-br from-primary-500/5 to-secondary-500/5 group-hover:scale-110 transition-transform duration-500" />
      
      <div className="relative">
        {/* Header with icon and value */}
        <div className="flex items-center justify-between mb-4">
          <div className={cn(
            'p-3 rounded-xl shadow-sm transition-colors',
            iconBgMap[variant],
            iconColorMap[variant]
          )}>
            <Icon className="h-6 w-6" />
          </div>
          
          <div className="text-right">
            <div className="text-2xl font-bold text-foreground leading-none">
              {value}
            </div>
            {trend && trendValue && (
              <div className="mt-1">
                <span className={cn(
                  'inline-flex items-center rounded-full px-2 py-1 text-xs font-medium',
                  adminBadgeVariants({ 
                    variant: trendColorMap[trend], 
                    size: 'sm' 
                  })
                )}>
                  {trend === 'up' ? '↗' : trend === 'down' ? '↘' : '→'} {trendValue}
                </span>
              </div>
            )}
          </div>
        </div>

        {/* Content */}
        <div>
          <h3 className="font-semibold text-foreground mb-1 text-sm">
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

/**
 * Grid component for multiple stats cards
 */
export interface StatsGridProps {
  cards: ModernStatsCardProps[];
  columns?: {
    default?: number;
    sm?: number;
    md?: number;
    lg?: number;
    xl?: number;
  };
  className?: string;
}

export function ModernStatsGrid({ 
  cards, 
  columns = { default: 1, md: 2, lg: 4 },
  className = ''
}: StatsGridProps) {
  const gridClasses = cn(
    'grid gap-4 md:gap-6',
    columns.default && `grid-cols-${columns.default}`,
    columns.sm && `sm:grid-cols-${columns.sm}`,
    columns.md && `md:grid-cols-${columns.md}`,
    columns.lg && `lg:grid-cols-${columns.lg}`,
    columns.xl && `xl:grid-cols-${columns.xl}`,
    className
  );

  return (
    <div className={gridClasses}>
      {cards.map((card, index) => (
        <ModernStatsCard
          key={card.title || `card-${index}`}
          {...card}
        />
      ))}
    </div>
  );
}

/**
 * Example usage:
 * 
 * ```tsx
 * import { Users, DollarSign, Activity, TrendingUp } from 'lucide-react';
 * 
 * const statsData = [
 *   {
 *     title: 'Total Users',
 *     value: '2,847',
 *     description: 'Active users this month',
 *     icon: Users,
 *     variant: 'user' as const,
 *     trend: 'up' as const,
 *     trendValue: '+12%'
 *   },
 *   {
 *     title: 'Revenue',
 *     value: '$89,432',
 *     description: 'Monthly recurring revenue',
 *     icon: DollarSign,
 *     variant: 'billing' as const,
 *     trend: 'up' as const,
 *     trendValue: '+8.2%'
 *   },
 *   // ... more cards
 * ];
 * 
 * return <ModernStatsGrid cards={statsData} />;
 * ```
 */