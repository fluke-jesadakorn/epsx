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
  iconBgColor?: string;
  iconColor?: string;
}

const purpleOrangeGradient = 'bg-gradient-to-br from-purple-500 to-orange-500';
const whiteText = 'text-white';

const VARIANT_ICON_COLORS = {
  enhanced: { bg: purpleOrangeGradient, text: whiteText },
  simple: { bg: purpleOrangeGradient, text: whiteText },
  inline: { bg: 'bg-muted/30 border border-border/20', text: 'text-purple-400' },
  default: { bg: 'bg-gradient-to-br from-purple-500/20 to-orange-500/20', text: 'text-purple-400 border border-purple-500/20' },
} as const;

function getIconColors(variant: StatsCardProps['variant']) {
  return VARIANT_ICON_COLORS[variant ?? 'default'];
}

function getCardClasses(variant: StatsCardProps['variant']) {
  const baseClasses = 'bg-muted/30 rounded-2xl border border-border/20 text-card-foreground hover:shadow-xl hover:border-purple-500/30 hover-lift transition-all duration-200';
  const paddingClasses = variant === 'inline' ? 'p-4' : 'p-6';
  return cn(baseClasses, paddingClasses);
}

interface EnhancedCardProps {
  cardClasses: string;
  className: string;
  gradient: string | undefined;
  textColor: string | undefined;
  iconColors: { bg: string; text: string };
  Icon: LucideIcon;
  value: string | number;
  trend: StatsCardProps['trend'];
  change: string | undefined;
  title: string;
  description: string | undefined;
}

function getTrendClass(trend: StatsCardProps['trend']): string {
  if (trend === 'up') { return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'; }
  if (trend === 'down') { return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'; }
  return 'bg-muted text-muted-foreground';
}

function getTrendArrow(trend: StatsCardProps['trend']): string {
  if (trend === 'up') { return '↗'; }
  if (trend === 'down') { return '↘'; }
  return '→';
}

function getIconClass(gradient: string | undefined, iconColors: { bg: string; text: string }): string {
  return cn(
    'p-3 rounded-xl shadow-sm transition-colors',
    gradient !== undefined && gradient !== '' ? `bg-gradient-to-r ${gradient}` : iconColors.bg,
    gradient !== undefined && gradient !== '' ? whiteText : iconColors.text
  );
}

function EnhancedCard({ cardClasses, className, gradient, textColor, iconColors, Icon, value, trend, change, title, description }: EnhancedCardProps) {
  const iconClass = getIconClass(gradient, iconColors);
  const showTrendBadge = trend !== undefined && change !== undefined && change !== '';
  const showChangeLine = change !== undefined && change !== '' && trend === undefined;

  return (
    <div className={cn(cardClasses, 'group', className)}>
      <div className="absolute top-0 right-0 w-20 h-20 -mr-8 -mt-8 rounded-full bg-gradient-to-br from-primary-500/5 to-secondary-500/5 group-hover:scale-110 transition-transform duration-500" />
      <div className="relative">
        <div className="flex items-center justify-between mb-4">
          <div className={iconClass}>
            <Icon className="h-6 w-6" />
          </div>
          <div className="text-right">
            <div className={cn('text-2xl font-bold leading-none', textColor ?? 'text-foreground')}>
              {value}
            </div>
            {showTrendBadge && (
              <div className="mt-1">
                <span className={cn('inline-flex items-center rounded-full px-2 py-1 text-xs font-medium', getTrendClass(trend))}>
                  {getTrendArrow(trend)} {change}
                </span>
              </div>
            )}
          </div>
        </div>
        <div>
          <h3 className="font-semibold text-foreground mb-1 text-sm">{title}</h3>
          {description !== undefined && description !== '' && (
            <p className="text-sm text-muted-foreground">{description}</p>
          )}
          {showChangeLine && (
            <p className="text-xs text-muted-foreground mt-1">{change}</p>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * @param gradient - optional gradient class string
 * @param iconColors - fallback icon color classes
 * @param iconColors.bg - background color class
 * @param iconColors.text - text color class
 * @param shape - border radius shape variant
 */
function resolveIconClass(gradient: string | undefined, iconColors: { bg: string; text: string }, shape: 'full' | 'lg'): string {
  const bg = gradient !== undefined && gradient !== '' ? `bg-gradient-to-r ${gradient}` : iconColors.bg;
  const text = gradient !== undefined && gradient !== '' ? whiteText : iconColors.text;
  return cn(`p-3 rounded-${shape} shadow-sm`, bg, text);
}

interface SimpleCardProps {
  cardClasses: string; className: string; gradient?: string;
  iconColors: { bg: string; text: string }; Icon: LucideIcon;
  title: string; value: string | number; description?: string;
}

function SimpleCard({ cardClasses, className, gradient, iconColors, Icon, title, value, description }: SimpleCardProps) {
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
        <div className={resolveIconClass(gradient, iconColors, 'full')}>
          <Icon className="h-6 w-6" />
        </div>
      </div>
    </div>
  );
}

interface InlineCardProps {
  cardClasses: string; className: string;
  iconColors: { bg: string; text: string }; Icon: LucideIcon;
  title: string; value: string | number; description?: string;
}

function InlineCard({ cardClasses, className, iconColors, Icon, title, value, description }: InlineCardProps) {
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

interface DefaultCardProps {
  cardClasses: string; className: string; gradient?: string; textColor?: string;
  iconColors: { bg: string; text: string }; iconBgColor?: string; iconColor?: string;
  Icon: LucideIcon; title: string; value: string | number; description?: string;
}

interface DefaultIconCtx { iconBgColor?: string; iconColor?: string; gradient?: string; iconColors: { bg: string; text: string }; }

function resolveDefaultIconClass({ iconBgColor, iconColor, gradient, iconColors }: DefaultIconCtx): { bg: string; text: string } {
  const hasGradient = gradient !== undefined && gradient !== '';
  return {
    bg: iconBgColor ?? (hasGradient ? `bg-gradient-to-r ${gradient}` : iconColors.bg),
    text: iconColor ?? (hasGradient ? whiteText : iconColors.text),
  };
}

function DefaultCard({ cardClasses, className, gradient, textColor, iconColors, iconBgColor, iconColor, Icon, title, value, description }: DefaultCardProps) {
  const { bg: iconBg, text: iconText } = resolveDefaultIconClass({ iconBgColor, iconColor, gradient, iconColors });
  return (
    <div className={cn(cardClasses, className)}>
      <div className="flex items-center justify-between mb-3">
        <div className={cn('p-2 rounded-lg shadow-sm', iconBg, iconText)}>
          <Icon className="h-5 w-5" />
        </div>
        {textColor !== undefined && textColor !== '' && (
          <div className={cn('text-right', textColor)}>
            <div className="text-xl font-bold">{value}</div>
          </div>
        )}
      </div>
      <div>
        <h3 className="text-sm font-medium text-muted-foreground mb-1">{title}</h3>
        {(textColor === undefined || textColor === '') && (
          <div className="text-2xl font-bold text-foreground">{value}</div>
        )}
        {description !== undefined && description !== '' && (
          <p className="text-xs text-muted-foreground mt-1">{description}</p>
        )}
      </div>
    </div>
  );
}

export function StatsCard({
  title,
  value,
  description,
  icon: Icon,
  variant = 'default',
  gradient,
  textColor,
  className = '',
  change,
  trend,
  iconBgColor,
  iconColor,
}: StatsCardProps) {
  const cardClasses = getCardClasses(variant);
  const iconColors = getIconColors(variant);

  if (variant === 'enhanced') {
    return (
      <EnhancedCard
        cardClasses={cardClasses}
        className={className}
        gradient={gradient}
        textColor={textColor}
        iconColors={iconColors}
        Icon={Icon}
        value={value}
        trend={trend}
        change={change}
        title={title}
        description={description}
      />
    );
  }

  if (variant === 'simple') {
    return <SimpleCard cardClasses={cardClasses} className={className} gradient={gradient} iconColors={iconColors} Icon={Icon} title={title} value={value} description={description} />;
  }

  if (variant === 'inline') {
    return <InlineCard cardClasses={cardClasses} className={className} iconColors={iconColors} Icon={Icon} title={title} value={value} description={description} />;
  }

  return <DefaultCard cardClasses={cardClasses} className={className} gradient={gradient} textColor={textColor} iconColors={iconColors} iconBgColor={iconBgColor} iconColor={iconColor} Icon={Icon} title={title} value={value} description={description} />;
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

function buildGridClass(columns: StatsGridProps['columns'], className: string) {
  const cols = columns ?? {};
  return [
    'grid gap-4',
    cols.default !== undefined && cols.default !== 0 ? `grid-cols-${cols.default}` : '',
    cols.sm !== undefined && cols.sm !== 0 ? `sm:grid-cols-${cols.sm}` : '',
    cols.md !== undefined && cols.md !== 0 ? `md:grid-cols-${cols.md}` : '',
    cols.lg !== undefined && cols.lg !== 0 ? `lg:grid-cols-${cols.lg}` : '',
    cols.xl !== undefined && cols.xl !== 0 ? `xl:grid-cols-${cols.xl}` : '',
    className
  ].filter(Boolean).join(' ');
}

/**
 *
 * @param root0
 * @param root0.stats
 * @param root0.variant
 * @param root0.columns
 * @param root0.className
 */
export function StatsGrid({
  stats,
  variant = 'default',
  columns = { default: 1, md: 2, lg: 4 },
  className = ''
}: StatsGridProps) {
  const gridClasses = buildGridClass(columns, className);

  return (
    <div className={gridClasses}>
      {stats.map((stat, index) => (
        <StatsCard
          key={stat.title !== '' ? stat.title : `stat-${index}`}
          {...stat}
          variant={stat.variant ?? variant}
        />
      ))}
    </div>
  );
}
