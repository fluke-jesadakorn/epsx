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

export type CardStatus = 
  | 'default'
  | 'success'
  | 'warning'
  | 'error'
  | 'info'

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
    'bg-white dark:bg-gray-800',
    'border border-gray-200 dark:border-gray-700',
    'shadow-sm'
  ],
  elevated: [
    'bg-white dark:bg-gray-800',
    'border border-gray-200 dark:border-gray-700',
    'shadow-md hover:shadow-lg'
  ],
  outlined: [
    'bg-transparent',
    'border-2 border-gray-300 dark:border-gray-600'
  ],
  filled: [
    'bg-gray-50 dark:bg-gray-700',
    'border border-gray-200 dark:border-gray-600'
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
  default: '',
  success: 'border-green-200 dark:border-green-700 bg-green-50 dark:bg-green-900/20',
  warning: 'border-yellow-200 dark:border-yellow-700 bg-yellow-50 dark:bg-yellow-900/20',
  error: 'border-red-200 dark:border-red-700 bg-red-50 dark:bg-red-900/20',
  info: 'border-blue-200 dark:border-blue-700 bg-blue-50 dark:bg-blue-900/20'
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

export const BaseCard = React.forwardRef<HTMLDivElement, BaseCardProps>(({
  children,
  title,
  subtitle,
  description,
  variant = 'default',
  size = 'md',
  status = 'default',
  orientation = 'vertical',
  header,
  footer,
  actions,
  clickable = false,
  hoverable = false,
  loading = false,
  disabled = false,
  padding,
  gap = 'md',
  onClick,
  onDoubleClick,
  onMouseEnter,
  onMouseLeave,
  className,
  style,
  role,
  'aria-label': ariaLabel,
  'aria-describedby': ariaDescribedBy,
  tabIndex,
  ...props
}, ref) => {
  // Determine if card should be interactive
  const isInteractive = clickable || !!onClick
  
  // Build base classes
  const baseClasses = [
    // Base styling
    'rounded-lg',
    'transition-all',
    'duration-200',
    
    // Variant styling
    ...cardVariants[variant],
    
    // Size styling (only if padding not explicitly set)
    padding ? paddingVariants[padding] : sizeVariants[size],
    
    // Status styling
    statusVariants[status],
    
    // Gap styling
    gapVariants[gap],
    
    // Orientation
    orientation === 'horizontal' ? 'flex flex-row items-center' : 'flex flex-col',
    
    // Interactive states
    isInteractive && [
      'cursor-pointer',
      'hover:shadow-md',
      'focus:outline-none',
      'focus:ring-2',
      'focus:ring-blue-500',
      'focus:ring-offset-2'
    ],
    
    // Hoverable effects
    hoverable && !isInteractive && 'hover:shadow-sm',
    
    // Disabled state
    disabled && [
      'opacity-50',
      'cursor-not-allowed',
      'pointer-events-none'
    ],
    
    // Loading state
    loading && 'animate-pulse'
  ].filter(Boolean).flat()

  // Handle keyboard navigation
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (isInteractive && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault()
      onClick?.()
    }
  }

  return (
    <div
      ref={ref}
      className={cn(baseClasses, className)}
      style={style}
      onClick={disabled ? undefined : onClick}
      onDoubleClick={disabled ? undefined : onDoubleClick}
      onMouseEnter={disabled ? undefined : onMouseEnter}
      onMouseLeave={disabled ? undefined : onMouseLeave}
      onKeyDown={handleKeyDown}
      role={role || (isInteractive ? 'button' : undefined)}
      aria-label={ariaLabel}
      aria-describedby={ariaDescribedBy}
      tabIndex={isInteractive ? (tabIndex ?? 0) : tabIndex}
      {...props}
    >
      {/* Header Section */}
      {header && (
        <div className="card-header border-b border-gray-200 dark:border-gray-700 pb-3 mb-3">
          {header}
        </div>
      )}

      {/* Title Section */}
      {(title || subtitle || actions) && (
        <div className={cn(
          'card-title-section',
          orientation === 'horizontal' ? 'flex-shrink-0 mr-4' : 'mb-3'
        )}>
          <div className="flex items-start justify-between">
            <div className="flex-1">
              {title && (
                <h3 className={cn(
                  'font-semibold text-gray-900 dark:text-gray-100',
                  size === 'sm' ? 'text-sm' : size === 'lg' ? 'text-lg' : 'text-base'
                )}>
                  {title}
                </h3>
              )}
              {subtitle && (
                <p className={cn(
                  'text-gray-500 dark:text-gray-400',
                  size === 'sm' ? 'text-xs' : 'text-sm',
                  title ? 'mt-1' : ''
                )}>
                  {subtitle}
                </p>
              )}
            </div>
            {actions && (
              <div className="flex-shrink-0 ml-3">
                {actions}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Description */}
      {description && (
        <div className={cn(
          'card-description text-gray-600 dark:text-gray-300',
          size === 'sm' ? 'text-sm' : 'text-base',
          orientation === 'horizontal' ? 'flex-shrink-0 mr-4' : 'mb-3'
        )}>
          {description}
        </div>
      )}

      {/* Main Content */}
      <div className={cn(
        'card-content flex-1',
        orientation === 'horizontal' ? 'flex flex-col justify-center' : ''
      )}>
        {loading ? (
          <div className="space-y-2">
            <div className="h-4 bg-gray-200 dark:bg-gray-600 rounded animate-pulse"></div>
            <div className="h-4 bg-gray-200 dark:bg-gray-600 rounded animate-pulse w-3/4"></div>
          </div>
        ) : (
          children
        )}
      </div>

      {/* Footer Section */}
      {footer && (
        <div className="card-footer border-t border-gray-200 dark:border-gray-700 pt-3 mt-3">
          {footer}
        </div>
      )}
    </div>
  )
})

BaseCard.displayName = 'BaseCard'

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
  const getTrendColor = (trend?: 'up' | 'down' | 'flat') => {
    switch (trend) {
      case 'up': return 'text-green-600 dark:text-green-400'
      case 'down': return 'text-red-600 dark:text-red-400'
      default: return 'text-gray-600 dark:text-gray-400'
    }
  }

  const getTrendIcon = (trend?: 'up' | 'down' | 'flat') => {
    switch (trend) {
      case 'up': return '↗'
      case 'down': return '↘'
      default: return '→'
    }
  }

  return (
    <BaseCard ref={ref} variant="elevated" {...props}>
      <div className="flex items-center">
        {icon && (
          <div className="flex-shrink-0 mr-3 text-gray-400">
            {icon}
          </div>
        )}
        <div className="flex-1">
          {value && (
            <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {value}
            </div>
          )}
          {label && (
            <div className="text-sm text-gray-500 dark:text-gray-400">
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
              {changeLabel && <span className="ml-1 text-gray-500">{changeLabel}</span>}
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
  const getStatusColor = (status: StatusCardProps['status']) => {
    switch (status) {
      case 'online': return 'text-green-600 dark:text-green-400'
      case 'offline': return 'text-gray-600 dark:text-gray-400'
      case 'warning': return 'text-yellow-600 dark:text-yellow-400'
      case 'error': return 'text-red-600 dark:text-red-400'
    }
  }

  const getStatusBgColor = (status: StatusCardProps['status']) => {
    switch (status) {
      case 'online': return 'bg-green-100 dark:bg-green-900/20'
      case 'offline': return 'bg-gray-100 dark:bg-gray-900/20'
      case 'warning': return 'bg-yellow-100 dark:bg-yellow-900/20'
      case 'error': return 'bg-red-100 dark:bg-red-900/20'
    }
  }

  return (
    <BaseCard ref={ref} variant="outlined" {...props}>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center">
          <div className={cn(
            'w-3 h-3 rounded-full mr-2',
            status === 'online' && 'bg-green-500',
            status === 'offline' && 'bg-gray-400',
            status === 'warning' && 'bg-yellow-500',
            status === 'error' && 'bg-red-500'
          )} />
          <span className={cn(
            'font-medium capitalize',
            getStatusColor(status)
          )}>
            {statusLabel || status}
          </span>
        </div>
        {lastUpdated && (
          <span className="text-xs text-gray-500">
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