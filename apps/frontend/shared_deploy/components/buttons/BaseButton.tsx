/**
 * BASE BUTTON COMPONENT
 * Unified button component to replace 15+ duplicate button implementations
 * Consolidates AdminPaginationButton, EditProfileButton, CleanupButton, etc.
 */

import React from 'react'
import { cn } from '../../utils'

// ============================================================================
// BUTTON VARIANT TYPES
// ============================================================================

export type ButtonVariant = 
  | 'primary'
  | 'secondary'
  | 'tertiary'
  | 'ghost'
  | 'link'
  | 'destructive'
  | 'success'
  | 'warning'
  | 'pancake'
  | 'pancakeSecondary'
  | 'pancakeOutline'
  | 'pancakeGhost'
  | 'pancakeLink'

export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

export type ButtonShape = 'rectangle' | 'rounded' | 'pill' | 'square' | 'circle'

// ============================================================================
// BUTTON COMPONENT PROPS
// ============================================================================

export interface BaseButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  // Content
  children?: React.ReactNode
  
  // Visual
  variant?: ButtonVariant
  size?: ButtonSize
  shape?: ButtonShape
  
  // State
  loading?: boolean
  disabled?: boolean
  active?: boolean
  
  // Icons
  leftIcon?: React.ReactNode
  rightIcon?: React.ReactNode
  icon?: React.ReactNode // For icon-only buttons
  
  // Behavior
  fullWidth?: boolean
  
  // As different element
  as?: React.ElementType
  href?: string
  target?: string
  
  // Custom styling
  className?: string
  
  // Loading text
  loadingText?: string
}

// ============================================================================
// STYLE VARIANTS
// ============================================================================

const variantStyles = {
  primary: [
    'bg-[hsl(var(--primary))] hover:bg-[hsl(var(--primary))]/90',
    'text-[hsl(var(--primary-foreground))]',
    'border border-[hsl(var(--primary))] hover:border-[hsl(var(--primary))]/90',
    'shadow-sm hover:shadow-md',
    'focus:ring-[hsl(var(--primary))]'
  ],
  secondary: [
    'bg-[hsl(var(--secondary))] hover:bg-[hsl(var(--secondary))]/80',
    'text-[hsl(var(--secondary-foreground))]',
    'border border-[hsl(var(--secondary))]/30 hover:border-[hsl(var(--secondary))]/50',
    'focus:ring-[hsl(var(--secondary))]'
  ],
  tertiary: [
    'bg-[hsl(var(--card))] hover:bg-[hsl(var(--accent))]',
    'text-[hsl(var(--foreground))]',
    'border border-[hsl(var(--border))] hover:border-[hsl(var(--border))]/80',
    'focus:ring-[hsl(var(--foreground))]/20'
  ],
  ghost: [
    'bg-transparent hover:bg-[hsl(var(--accent))]',
    'text-[hsl(var(--foreground))] hover:text-[hsl(var(--foreground))]/90',
    'border border-transparent',
    'focus:ring-[hsl(var(--foreground))]/20'
  ],
  link: [
    'bg-transparent',
    'text-[hsl(var(--primary))] hover:text-[hsl(var(--primary))]/80',
    'border border-transparent',
    'underline hover:no-underline',
    'focus:ring-[hsl(var(--primary))]',
    'shadow-none'
  ],
  destructive: [
    'bg-[hsl(var(--destructive))] hover:bg-[hsl(var(--destructive))]/90',
    'text-[hsl(var(--destructive-foreground))]',
    'border border-[hsl(var(--destructive))] hover:border-[hsl(var(--destructive))]/90',
    'shadow-sm hover:shadow-md',
    'focus:ring-[hsl(var(--destructive))]'
  ],
  success: [
    'bg-green-600 hover:bg-green-700',
    'text-white',
    'border border-green-600 hover:border-green-700',
    'shadow-sm hover:shadow-md',
    'focus:ring-green-500'
  ],
  warning: [
    'bg-yellow-600 hover:bg-yellow-700',
    'text-white',
    'border border-yellow-600 hover:border-yellow-700',
    'shadow-sm hover:shadow-md',
    'focus:ring-yellow-500'
  ],
  pancake: [
    'bg-gradient-to-r from-yellow-400 via-yellow-500 to-orange-500',
    'text-black',
    'border-0',
    'hover:from-yellow-300 hover:to-orange-400',
    'shadow-xl hover:shadow-2xl',
    'font-normal uppercase tracking-wider',
    'focus:ring-yellow-500'
  ],
  pancakeSecondary: [
    'bg-gradient-to-r from-blue-600 to-blue-700',
    'text-white',
    'border-0',
    'hover:from-blue-500 hover:to-blue-600',
    'shadow-xl hover:shadow-2xl',
    'font-normal uppercase tracking-wider',
    'focus:ring-blue-500'
  ],
  pancakeOutline: [
    'border-2 border-yellow-400',
    'bg-transparent',
    'text-yellow-600 dark:text-yellow-400',
    'hover:bg-yellow-400 hover:text-black',
    'font-light uppercase tracking-wider',
    'focus:ring-yellow-500'
  ],
  pancakeGhost: [
    'bg-transparent',
    'text-yellow-600 dark:text-yellow-400',
    'border border-transparent',
    'hover:bg-yellow-400/20 hover:text-yellow-600',
    'dark:hover:text-yellow-400',
    'font-light uppercase tracking-wider',
    'focus:ring-yellow-500'
  ],
  pancakeLink: [
    'bg-transparent',
    'text-yellow-600 dark:text-yellow-400',
    'border border-transparent',
    'underline-offset-4 hover:underline',
    'hover:text-yellow-500 dark:hover:text-yellow-300',
    'font-light uppercase tracking-wider',
    'focus:ring-yellow-500',
    'shadow-none'
  ]
}

const sizeStyles = {
  xs: [
    'text-xs',
    'px-2 py-1',
    'h-6'
  ],
  sm: [
    'text-sm',
    'px-3 py-1.5',
    'h-8'
  ],
  md: [
    'text-sm',
    'px-4 py-2',
    'h-10'
  ],
  lg: [
    'text-base',
    'px-6 py-2.5',
    'h-12'
  ],
  xl: [
    'text-lg',
    'px-8 py-3',
    'h-14'
  ]
}

const shapeStyles = {
  rectangle: 'rounded-none',
  rounded: 'rounded-md',
  pill: 'rounded-full',
  square: 'rounded-md aspect-square',
  circle: 'rounded-full aspect-square'
}

// Icon sizes for different button sizes
const iconSizes = {
  xs: 'w-3 h-3',
  sm: 'w-4 h-4',
  md: 'w-4 h-4',
  lg: 'w-5 h-5',
  xl: 'w-6 h-6'
}

// ============================================================================
// BASE BUTTON COMPONENT
// ============================================================================

export const BaseButton = React.forwardRef<HTMLButtonElement, BaseButtonProps>(({
  children,
  variant = 'primary',
  size = 'md',
  shape = 'rounded',
  loading = false,
  disabled = false,
  active = false,
  leftIcon,
  rightIcon,
  icon,
  fullWidth = false,
  as,
  href,
  target,
  className,
  loadingText,
  type = 'button',
  ...props
}, ref) => {
  // Determine if button should be disabled
  const isDisabled = disabled || loading

  // Choose the component to render
  const Component = as || (href ? 'a' : 'button')

  // Icon-only button
  const isIconOnly = !!icon && !children

  // Build classes
  const baseClasses = [
    // Base styling
    'inline-flex items-center justify-center',
    'font-medium',
    'transition-all duration-200',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-[hsl(var(--background))]',
    'select-none',
    
    // Variant styling
    ...variantStyles[variant],
    
    // Size styling
    ...sizeStyles[size],
    
    // Shape styling
    shapeStyles[shape],
    
    // Full width
    fullWidth && 'w-full',
    
    // Active state
    active && 'ring-2 ring-offset-2',
    
    // Disabled state
    isDisabled && [
      'opacity-50',
      'cursor-not-allowed',
      'pointer-events-none'
    ],
    
    // Icon-only adjustments
    isIconOnly && 'p-0'
  ].filter(Boolean).flat()

  // Loading spinner component
  const LoadingSpinner = () => (
    <svg 
      className={cn('animate-spin', iconSizes[size])} 
      fill="none" 
      viewBox="0 0 24 24"
    >
      <circle 
        className="opacity-25" 
        cx="12" 
        cy="12" 
        r="10" 
        stroke="currentColor" 
        strokeWidth="4"
      />
      <path 
        className="opacity-75" 
        fill="currentColor" 
        d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>
  )

  // Content to render
  const renderContent = () => {
    if (loading) {
      return (
        <>
          <LoadingSpinner />
          {loadingText && (
            <span className="ml-2">{loadingText}</span>
          )}
        </>
      )
    }

    if (isIconOnly) {
      return (
        <span className={iconSizes[size]}>
          {icon}
        </span>
      )
    }

    return (
      <>
        {leftIcon && (
          <span className={cn(iconSizes[size], 'mr-2')}>
            {leftIcon}
          </span>
        )}
        {children}
        {rightIcon && (
          <span className={cn(iconSizes[size], 'ml-2')}>
            {rightIcon}
          </span>
        )}
      </>
    )
  }

  // Component props
  const componentProps = {
    ref,
    className: cn(baseClasses, className),
    disabled: isDisabled,
    type: Component === 'button' ? type : undefined,
    href: Component === 'a' ? href : undefined,
    target: Component === 'a' ? target : undefined,
    'aria-disabled': isDisabled,
    'aria-pressed': active,
    ...props
  }

  return (
    <Component {...componentProps}>
      {renderContent()}
    </Component>
  )
})

BaseButton.displayName = 'BaseButton'

// ============================================================================
// SPECIALIZED BUTTON COMPONENTS
// ============================================================================

/**
 * Icon Button - for icon-only buttons
 */
export interface IconButtonProps extends Omit<BaseButtonProps, 'children' | 'leftIcon' | 'rightIcon'> {
  icon: React.ReactNode
  'aria-label': string
}

export const IconButton = React.forwardRef<HTMLButtonElement, IconButtonProps>(({
  icon,
  shape = 'circle',
  variant = 'ghost',
  ...props
}, ref) => {
  return (
    <BaseButton
      ref={ref}
      icon={icon}
      shape={shape}
      variant={variant}
      {...props}
    />
  )
})

IconButton.displayName = 'IconButton'

/**
 * Loading Button - button with built-in loading state
 */
export interface LoadingButtonProps extends BaseButtonProps {
  isLoading?: boolean
  loadingText?: string
}

export const LoadingButton = React.forwardRef<HTMLButtonElement, LoadingButtonProps>(({
  isLoading = false,
  loadingText = 'Loading...',
  children,
  ...props
}, ref) => {
  return (
    <BaseButton
      ref={ref}
      loading={isLoading}
      loadingText={loadingText}
      {...props}
    >
      {children}
    </BaseButton>
  )
})

LoadingButton.displayName = 'LoadingButton'

/**
 * Action Button - for common actions with predefined variants
 */
export interface ActionButtonProps extends Omit<BaseButtonProps, 'variant'> {
  action: 'create' | 'edit' | 'delete' | 'save' | 'cancel' | 'submit' | 'reset'
}

export const ActionButton = React.forwardRef<HTMLButtonElement, ActionButtonProps>(({
  action,
  children,
  ...props
}, ref) => {
  const getActionVariant = (action: ActionButtonProps['action']): ButtonVariant => {
    switch (action) {
      case 'create':
      case 'save':
      case 'submit':
        return 'primary'
      case 'edit':
        return 'secondary'
      case 'delete':
        return 'destructive'
      case 'cancel':
      case 'reset':
        return 'ghost'
      default:
        return 'secondary'
    }
  }

  const getActionIcon = (action: ActionButtonProps['action']) => {
    switch (action) {
      case 'create':
        return '+'
      case 'edit':
        return '✏️'
      case 'delete':
        return '🗑️'
      case 'save':
        return '💾'
      case 'cancel':
        return '✕'
      case 'submit':
        return '✓'
      case 'reset':
        return '↺'
      default:
        return null
    }
  }

  return (
    <BaseButton
      ref={ref}
      variant={getActionVariant(action)}
      leftIcon={getActionIcon(action)}
      {...props}
    >
      {children || action.charAt(0).toUpperCase() + action.slice(1)}
    </BaseButton>
  )
})

ActionButton.displayName = 'ActionButton'

/**
 * Pagination Button - for pagination controls
 */
export interface PaginationButtonProps extends Omit<BaseButtonProps, 'variant' | 'size'> {
  direction?: 'previous' | 'next'
  page?: number
  current?: boolean
}

export const PaginationButton = React.forwardRef<HTMLButtonElement, PaginationButtonProps>(({
  direction,
  page,
  current = false,
  children,
  ...props
}, ref) => {
  const getIcon = () => {
    if (direction === 'previous') return '‹'
    if (direction === 'next') return '›'
    return null
  }

  return (
    <BaseButton
      ref={ref}
      variant={current ? 'primary' : 'ghost'}
      size="sm"
      shape="rounded"
      active={current}
      leftIcon={direction === 'previous' ? getIcon() : undefined}
      rightIcon={direction === 'next' ? getIcon() : undefined}
      {...props}
    >
      {children || page}
    </BaseButton>
  )
})

PaginationButton.displayName = 'PaginationButton'

export default BaseButton