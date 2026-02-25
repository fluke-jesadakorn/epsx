/**
 * StatsCard Component - Modernized with Design System
 * 
 * Updated to use the new design system while maintaining backward compatibility.
 * Uses adminCardVariants and semantic color system instead of hardcoded values.
 */

import type { LucideIcon } from 'lucide-react';

import { cn } from '@/lib/utils';

export interface StatsCardProps {
  title: string;
  value: string | number;
  description?: string;
  icon: LucideIcon;
  variant?: 'default' | 'enhanced' | 'simple' | 'inline';
  // Legacy props for backward compatibility
  gradient?: string;
  textColor?: string;
  color?: string;
  className?: string;
  change?: string;
  // New design system props
  cardVariant?: 'default' | 'pancake' | 'user' | 'permission' | 'billing' | 'analytics' | 'warning' | 'error';
  trend?: 'up' | 'down' | 'neutral';
}

/**
 *
 * @param root0
 * @param root0.title
 * @param root0.value
 * @param root0.description
 * @param root0.icon
 * @param root0.variant
 * @param root0.gradient
 * @param root0.textColor
 * @param root0.className
 * @param root0.change
 * @param root0.trend
 */
// eslint-disable-next-line max-lines-per-function, complexity, sonarjs/cognitive-complexity
export function StatsCard({
  title,
  value,
  description,
  icon: Icon,
  variant = 'default',
  // Legacy props for backward compatibility
  gradient,
  textColor,
  className = '',
  change,
  // New design system props
  trend,
}: StatsCardProps) {

  // Semantic color mapping
  const purpleOrangeGradient = 'bg-gradient-to-br from-purple-500 to-orange-500';
  // eslint-disable-next-line sonarjs/no-duplicate-string
  const whiteText = 'text-white';

  const getIconColors = () => {
    const variantMap = {
      enhanced: { bg: purpleOrangeGradient, text: whiteText },
      simple: { bg: purpleOrangeGradient, text: whiteText },
      inline: { bg: 'bg-muted/30 border border-border/20', text: 'text-purple-400' },
      default: { bg: 'bg-gradient-to-br from-purple-500/20 to-orange-500/20', text: 'text-purple-400 border border-purple-500/20' },
    };

    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    return variantMap[variant] ?? variantMap.default;
  };

  // Use standard Tailwind classes instead of design system
  const getCardClasses = () => {
    // Glass morphism card styles
    const baseClasses = 'bg-muted/30 rounded-2xl border border-border/20 text-card-foreground hover:shadow-xl hover:border-purple-500/30 hover-lift transition-all duration-200';
    const paddingClasses = variant === 'inline' ? 'p-4' : 'p-6';

    return cn(baseClasses, paddingClasses);
  };

  const cardClasses = getCardClasses();

  const iconColors = getIconColors();

  // Enhanced variant with design system
  if (variant === 'enhanced') {
    return (
      <div className={cn(cardClasses, 'group', className)}>
        <div className="absolute top-0 right-0 w-20 h-20 -mr-8 -mt-8 rounded-full bg-gradient-to-br from-primary-500/5 to-secondary-500/5 group-hover:scale-110 transition-transform duration-500" />
        <div className="relative">
          <div className="flex items-center justify-between mb-4">
            <div className={cn(
              'p-3 rounded-xl shadow-sm transition-colors',
              gradient !== undefined && gradient !== '' ? `bg-gradient-to-r ${gradient}` : iconColors.bg,
              gradient !== undefined && gradient !== '' ? 'text-white' : iconColors.text
            )}>
              <Icon className="h-6 w-6" />
            </div>
            <div className="text-right">
              <div className={cn(
                'text-2xl font-bold leading-none',
                textColor ?? 'text-foreground'
              )}>
                {value}
              </div>
              {trend !== undefined && change !== undefined && change !== '' && (
                <div className="mt-1">
                  <span className={cn(
                    'inline-flex items-center rounded-full px-2 py-1 text-xs font-medium',
                    trend === 'up' ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400' :
                      trend === 'down' ? 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400' :
                        'bg-muted text-muted-foreground'
                  )}>
                    {trend === 'up' ? '↗' : trend === 'down' ? '↘' : '→'} {change}
                  </span>
                </div>
              )}
            </div>
          </div>
          <div>
            <h3 className="font-semibold text-foreground mb-1 text-sm">
              {title}
            </h3>
            {description !== undefined && description !== '' && (
              <p className="text-sm text-muted-foreground">
                {description}
              </p>
            )}
            {change !== undefined && change !== '' && trend === undefined && (
              <p className="text-xs text-muted-foreground mt-1">
                {change}
              </p>
            )}
          </div>
        </div>
      </div>
    );
  }

  // Simple variant with design system
  if (variant === 'simple') {
    return (
      <div className={cn(cardClasses, className)}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-muted-foreground">{title}</p>
            <p className="text-2xl font-bold text-foreground mt-1">{value}</p>
            {description !== undefined && description !== '' && (
              <p className="text-sm text-muted-foreground mt-1">{description}</p>
            )}
          </div>
          <div className={cn(
            'p-3 rounded-full shadow-sm',
            gradient !== undefined && gradient !== '' ? `bg-gradient-to-r ${gradient}` : iconColors.bg,
            gradient !== undefined && gradient !== '' ? 'text-white' : iconColors.text
          )}>
            <Icon className="h-6 w-6" />
          </div>
        </div>
      </div>
    );
  }

  // Inline variant with design system
  if (variant === 'inline') {
    return (
      <div className={cn(cardClasses, className)}>
        <div className="flex items-center">
          <Icon className={cn('w-5 h-5', iconColors.text)} />
          <span className="ml-2 text-sm font-medium text-muted-foreground">{title}</span>
        </div>
        <div className="mt-2">
          <span className="text-2xl font-bold text-foreground">{value}</span>
        </div>
        {description !== undefined && description !== '' && (
          <p className="text-xs text-muted-foreground mt-1">{description}</p>
        )}
      </div>
    );
  }

  // Default variant with design system
  return (
    <div className={cn(cardClasses, className)}>
      <div className="flex items-center justify-between mb-3">
        <div className={cn(
          'p-2 rounded-lg shadow-sm',
          gradient !== undefined && gradient !== '' ? `bg-gradient-to-r ${gradient}` : iconColors.bg,
          gradient !== undefined && gradient !== '' ? 'text-white' : iconColors.text
        )}>
          <Icon className="h-5 w-5" />
        </div>
        {textColor !== undefined && textColor !== '' && (
          <div className={cn('text-right', textColor)}>
            <div className="text-xl font-bold">
              {value}
            </div>
          </div>
        )}
      </div>
      <div>
        <h3 className="text-sm font-medium text-muted-foreground mb-1">
          {title}
        </h3>
        {textColor === undefined || textColor === '' && (
          <div className="text-2xl font-bold text-foreground">
            {value}
          </div>
        )}
        {description !== undefined && description !== '' && (
          <p className="text-xs text-muted-foreground mt-1">
            {description}
          </p>
        )}
      </div>
    </div>
  );
}

// Convenience wrapper for multiple stats cards
export interface StatsGridProps {
  stats: StatsCardProps[];
  variant?: StatsCardProps['variant'];
  columns?: {
    default?: number;
    sm?: number;
    md?: number;
    lg?: number;
    xl?: number;
  };
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.stats
 * @param root0.variant
 * @param root0.columns
 * @param root0.className
 */
// eslint-disable-next-line complexity
export function StatsGrid({
  stats,
  variant = 'default',
  columns = { default: 1, md: 2, lg: 4 },
  className = ''
}: StatsGridProps) {
  const gridClasses = [
    `grid gap-4`,
    columns.default !== undefined && columns.default !== 0 ? `grid-cols-${columns.default}` : '',
    columns.sm !== undefined && columns.sm !== 0 ? `sm:grid-cols-${columns.sm}` : '',
    columns.md !== undefined && columns.md !== 0 ? `md:grid-cols-${columns.md}` : '',
    columns.lg !== undefined && columns.lg !== 0 ? `lg:grid-cols-${columns.lg}` : '',
    columns.xl !== undefined && columns.xl !== 0 ? `xl:grid-cols-${columns.xl}` : '',
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={gridClasses}>
      {stats.map((stat, index) => (
        <StatsCard
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          key={stat.title ?? `stat-${index}`}
          {...stat}
          variant={stat.variant ?? variant}
        />
      ))}
    </div>
  );
}