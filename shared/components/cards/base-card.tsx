'use client'

/**
 * BASE CARD COMPONENT
 * Unified card component to replace 20+ duplicate card implementations
 * Consolidates AdminEPSCard, PermissionAssignmentCard, SystemHealthCard, etc.
 */

import React from 'react'
import { cn } from '../../utils'

// ============================================================================
// CARD VARIANT TYPES
// ============================================================================

export type CardVariant =
  | 'default'
  | 'elevated'
  | 'outlined'
  | 'filled'
  | 'minimal'
  | 'pancake'
  | 'pancakeElevated'
  | 'pancakeOutlined'

export type CardSize = 'sm' | 'md' | 'lg' | 'xl'

export const CARD_STATUS = {
  DEFAULT: 'default',
  SUCCESS: 'success',
  WARNING: 'warning',
  ERROR: 'error',
  INFO: 'info'
} as const;

export type CardStatus = typeof CARD_STATUS[keyof typeof CARD_STATUS];

export type CardOrientation = 'vertical' | 'horizontal'

// ============================================================================
// CARD COMPONENT PROPS
// ============================================================================

export interface BaseCardProps {
  // Content
  children: React.ReactNode
  title?: React.ReactNode
  subtitle?: React.ReactNode
  description?: React.ReactNode

  // Visual
  variant?: CardVariant
  size?: CardSize
  status?: CardStatus
  orientation?: CardOrientation

  // Header and footer
  header?: React.ReactNode
  footer?: React.ReactNode
  actions?: React.ReactNode

  // Behavior
  clickable?: boolean
  hoverable?: boolean
  loading?: boolean
  disabled?: boolean

  // Layout
  padding?: 'none' | 'sm' | 'md' | 'lg'
  gap?: 'none' | 'sm' | 'md' | 'lg'

  // Events
  onClick?: () => void
  onDoubleClick?: () => void
  onMouseEnter?: () => void
  onMouseLeave?: () => void

  // Styling
  className?: string
  style?: React.CSSProperties

  // Accessibility
  role?: string
  'aria-label'?: string
  'aria-describedby'?: string
  tabIndex?: number
}

// ============================================================================
// STYLE VARIANTS
// ============================================================================

const cardVariants = {
  default: [
    'bg-[hsl(var(--card))]',
    'border border-[hsl(var(--border))]',
    'shadow-sm'
  ],
  elevated: [
    'bg-[hsl(var(--card))]',
    'border border-[hsl(var(--border))]',
    'shadow-md hover:shadow-lg'
  ],
  outlined: [
    'bg-transparent',
    'border-2 border-[hsl(var(--border))]'
  ],
  filled: [
    'bg-[hsl(var(--muted))]',
    'border border-[hsl(var(--border))]'
  ],
  minimal: [
    'bg-transparent',
    'border-0'
  ],
  pancake: [
    'border-0',
    'bg-gradient-to-br from-orange-50 to-yellow-50',
    'dark:from-orange-950 dark:to-yellow-950',
    'text-card-foreground',
    'shadow-xl',
    'backdrop-blur-sm',
    'relative overflow-hidden group',
    'rounded-lg'
  ],
  pancakeElevated: [
    'border-0',
    'bg-gradient-to-br from-orange-50 to-yellow-50',
    'dark:from-orange-950 dark:to-yellow-950',
    'text-card-foreground',
    'shadow-xl hover:shadow-2xl',
    'hover:shadow-orange-200 dark:hover:shadow-orange-900',
    'backdrop-blur-sm',
    'hover:scale-[1.01]',
    'relative overflow-hidden group',
    'rounded-lg'
  ],
  pancakeOutlined: [
    'border-2 border-orange-400 dark:border-orange-600',
    'bg-gradient-to-br from-orange-50/50 to-yellow-50/50',
    'dark:from-orange-950/50 dark:to-yellow-950/50',
    'text-card-foreground',
    'shadow-md',
    'relative overflow-hidden group',
    'rounded-lg'
  ]
}

const sizeVariants = {
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
  xl: 'p-8'
}

const statusVariants = {
  [CARD_STATUS.DEFAULT]: '',
  [CARD_STATUS.SUCCESS]: 'border-green-200 dark:border-green-700 bg-green-50 dark:bg-green-900/20',
  [CARD_STATUS.WARNING]: 'border-yellow-200 dark:border-yellow-700 bg-yellow-50 dark:bg-yellow-900/20',
  [CARD_STATUS.ERROR]: 'border-red-200 dark:border-red-700 bg-red-50 dark:bg-red-900/20',
  [CARD_STATUS.INFO]: 'border-blue-200 dark:border-blue-700 bg-blue-50 dark:bg-blue-900/20'
}

const paddingVariants = {
  none: 'p-0',
  sm: 'p-2',
  md: 'p-4',
  lg: 'p-6'
}

const gapVariants = {
  none: 'space-y-0',
  sm: 'space-y-2',
  md: 'space-y-4',
  lg: 'space-y-6'
}

// ============================================================================
// BASE CARD COMPONENT
// ============================================================================

// ============================================================================
// HELPERS
// ============================================================================

const getVariantClasses = (variant: CardVariant) => cardVariants[variant];
const getStatusClasses = (status: CardStatus) => statusVariants[status] || '';
const getLayoutClasses = (options: { padding: BaseCardProps['padding'], size: CardSize, gap: BaseCardProps['gap'], orientation: CardOrientation }) => {
  const { padding, size, gap, orientation } = options;
  const sizeClasses = padding ? paddingVariants[padding] : sizeVariants[size];
  const gapClasses = gap ? gapVariants[gap] : gapVariants.md;
  const orientationClasses = orientation === 'horizontal' ? 'flex flex-row items-center' : 'flex flex-col';
  return cn(sizeClasses, gapClasses, orientationClasses);
};

const getStateClasses = (disabled: boolean, loading: boolean) => cn(
  disabled && 'opacity-50 cursor-not-allowed pointer-events-none',
  loading && 'animate-pulse'
);

const getInteractionClasses = (isInteractive: boolean, hoverable: boolean) => {
  if (isInteractive) {
    return 'cursor-pointer hover:shadow-md focus:outline-none focus:ring-2 focus:ring-[hsl(var(--ring))] focus:ring-offset-2 focus:ring-offset-[hsl(var(--background))]';
  }
  return hoverable ? 'hover:shadow-sm' : '';
};

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

const CardTitleSection: React.FC<{
  title?: React.ReactNode;
  subtitle?: React.ReactNode;
  actions?: React.ReactNode;
  size: CardSize;
  orientation: CardOrientation;
}> = ({ title, subtitle, actions, size, orientation }) => {
  if (!title && !subtitle && !actions) { return null; }

  const titleSize = size === 'sm' ? 'text-sm' : size === 'lg' ? 'text-lg' : 'text-base';
  const subtitleSize = size === 'sm' ? 'text-xs' : 'text-sm';

  return (
    <div className={cn('card-title-section', orientation === 'horizontal' ? 'flex-shrink-0 mr-4' : 'mb-3')}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {title && <h3 className={cn('font-semibold text-[hsl(var(--foreground))]', titleSize)}>{title}</h3>}
          {subtitle && (
            <p className={cn('text-[hsl(var(--muted-foreground))]', subtitleSize, title ? 'mt-1' : '')}>
              {subtitle}
            </p>
          )}
        </div>
        {actions && <div className="flex-shrink-0 ml-3">{actions}</div>}
      </div>
    </div>
  );
};

const CardHeaderSection: React.FC<{ header?: React.ReactNode }> = ({ header }) => {
  if (!header) { return null; }
  return <div className="card-header border-b border-[hsl(var(--border))] pb-3 mb-3">{header}</div>;
};

const CardFooterSection: React.FC<{ footer?: React.ReactNode }> = ({ footer }) => {
  if (!footer) { return null; }
  return <div className="card-footer border-t border-[hsl(var(--border))] pt-3 mt-3">{footer}</div>;
};

const CardDescriptionSection: React.FC<{
  description?: React.ReactNode;
  size: CardSize;
  orientation: CardOrientation;
}> = ({ description, size, orientation }) => {
  if (!description) { return null; }
  return (
    <div className={cn(
      'card-description text-[hsl(var(--foreground))]/80',
      size === 'sm' ? 'text-sm' : 'text-base',
      orientation === 'horizontal' ? 'flex-shrink-0 mr-4' : 'mb-3'
    )}>
      {description}
    </div>
  );
};

const getCardAriaProps = (isInteractive: boolean, props: Partial<BaseCardProps>) => {
  const { role, 'aria-label': ariaLabel, 'aria-describedby': ariaDescribedBy, tabIndex } = props;
  return {
    role: role ?? (isInteractive ? 'button' : undefined),
    'aria-label': ariaLabel,
    'aria-describedby': ariaDescribedBy,
    tabIndex: isInteractive ? (tabIndex ?? 0) : tabIndex
  };
};

// ============================================================================
// BASE CARD COMPONENT
// ============================================================================

const getBaseCardClasses = (props: BaseCardProps, isInteractive: boolean) => {
  const {
    variant = 'default', size = 'md', status = 'default', orientation = 'vertical',
    hoverable = false, padding, gap = 'md', disabled = false, loading = false
  } = props;

  const baseClasses = cn(
    'rounded-lg transition-all duration-200',
    getVariantClasses(variant),
    getStatusClasses(status),
    getLayoutClasses({ padding, size, gap, orientation }),
    getInteractionClasses(isInteractive, hoverable)
  );

  const stateClasses = getStateClasses(disabled, loading);
  return cn(baseClasses, stateClasses);
};

// ============================================================================
// BASE CARD COMPONENT
// ============================================================================

export const BaseCard = React.forwardRef<HTMLDivElement, BaseCardProps>((props, ref) => {
  const {
    children, title, subtitle, description,
    size = 'md', orientation = 'vertical',
    header, footer, actions, clickable = false,
    loading = false, disabled = false,
    onClick, onDoubleClick, onMouseEnter, onMouseLeave,
    className, style, ...rest
  } = props;

  const isInteractive = (clickable || Boolean(onClick)) && !disabled;
  const combinedClasses = getBaseCardClasses(props, isInteractive);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (isInteractive && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      onClick?.();
    }
  };

  const ariaProps = getCardAriaProps(isInteractive, props);

  return (
    <div
      ref={ref}
      className={cn(combinedClasses, className)}
      style={style}
      onClick={disabled ? undefined : onClick}
      onDoubleClick={disabled ? undefined : onDoubleClick}
      onMouseEnter={disabled ? undefined : onMouseEnter}
      onMouseLeave={disabled ? undefined : onMouseLeave}
      onKeyDown={handleKeyDown}
      {...ariaProps}
      {...rest}
    >
      <CardHeaderSection header={header} />
      <CardTitleSection title={title} subtitle={subtitle} actions={actions} size={size} orientation={orientation} />
      <CardDescriptionSection description={description} size={size} orientation={orientation} />
      <CardContent orientation={orientation} loading={loading}>{children}</CardContent>
      <CardFooterSection footer={footer} />
    </div>
  );
});

function CardContent({
  orientation,
  loading,
  children
}: {
  orientation: CardOrientation;
  loading: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className={cn(
      'card-content flex-1',
      orientation === 'horizontal' ? 'flex flex-col justify-center' : ''
    )}>
      {loading ? (
        <div className="space-y-2">
          <div className="h-4 bg-[hsl(var(--muted))] rounded animate-pulse" />
          <div className="h-4 bg-[hsl(var(--muted))] rounded animate-pulse w-3/4" />
        </div>
      ) : (
        children
      )}
    </div>
  )
}

BaseCard.displayName = 'base-card'

// ============================================================================
// SPECIALIZED CARD COMPONENTS
// ============================================================================

/**
 * Data Card - for displaying metrics and statistics
 */
export interface DataCardProps extends Omit<BaseCardProps, 'variant'> {
  value?: React.ReactNode
  label?: string
  change?: number
  changeLabel?: string
  trend?: 'up' | 'down' | 'flat'
  icon?: React.ReactNode
}

export const DataCard = React.forwardRef<HTMLDivElement, DataCardProps>(({
  value,
  label,
  change,
  changeLabel,
  trend,
  icon,
  children,
  ...props
}, ref) => {
  const getTrendColor = (t?: 'up' | 'down' | 'flat') => {
    switch (t) {
      case 'up': return 'text-[hsl(var(--success))]'
      case 'down': return 'text-[hsl(var(--destructive))]'
      default: return 'text-[hsl(var(--muted-foreground))]'
    }
  }

  const getTrendIcon = (t?: 'up' | 'down' | 'flat') => {
    switch (t) {
      case 'up': return '↗'
      case 'down': return '↘'
      default: return '→'
    }
  }

  return (
    <BaseCard ref={ref} variant="elevated" {...props}>
      <div className="flex items-center">
        {icon && (
          <div className="flex-shrink-0 mr-3 text-[hsl(var(--muted-foreground))]">
            {icon}
          </div>
        )}
        <div className="flex-1">
          {value && (
            <div className="text-2xl font-bold text-[hsl(var(--foreground))]">
              {value}
            </div>
          )}
          {label && (
            <div className="text-sm text-[hsl(var(--muted-foreground))]">
              {label}
            </div>
          )}
          {change !== undefined && (
            <div className={cn(
              'text-sm font-medium flex items-center mt-1',
              getTrendColor(trend)
            )}>
              <span className="mr-1">{getTrendIcon(trend)}</span>
              {Math.abs(change)}%
              {changeLabel && <span className="ml-1 text-[hsl(var(--muted-foreground))]">{changeLabel}</span>}
            </div>
          )}
        </div>
      </div>
      {children}
    </BaseCard>
  )
})

DataCard.displayName = 'DataCard'

/**
 * Status Card - for displaying status information
 */
export interface StatusCardProps extends Omit<BaseCardProps, 'status'> {
  status: 'online' | 'offline' | 'warning' | 'error'
  statusLabel?: string
  lastUpdated?: string
}

export const StatusCard = React.forwardRef<HTMLDivElement, StatusCardProps>(({
  status,
  statusLabel,
  lastUpdated,
  children,
  ...props
}, ref) => {
  const getStatusColor = (s: StatusCardProps['status']) => {
    switch (s) {
      case 'online': return 'text-[hsl(var(--success))]'
      case 'offline': return 'text-[hsl(var(--muted-foreground))]'
      case 'warning': return 'text-[hsl(var(--warning))]'
      case 'error': return 'text-[hsl(var(--destructive))]'
    }
  }

  return (
    <BaseCard ref={ref} variant="outlined" {...props}>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center">
          <div className={cn(
            'w-3 h-3 rounded-full mr-2',
            status === 'online' && 'bg-[hsl(var(--success))]',
            status === 'offline' && 'bg-[hsl(var(--muted-foreground))]',
            status === 'warning' && 'bg-[hsl(var(--warning))]',
            status === 'error' && 'bg-[hsl(var(--destructive))]'
          )} />
          <span className={cn(
            'font-medium capitalize',
            getStatusColor(status)
          )}>
            {statusLabel ?? status}
          </span>
        </div>
        {lastUpdated && (
          <span className="text-xs text-[hsl(var(--muted-foreground))]">
            {lastUpdated}
          </span>
        )}
      </div>
      {children}
    </BaseCard>
  )
})

StatusCard.displayName = 'StatusCard'

export default BaseCard