/**
 * BASE BUTTON COMPONENT
 * Unified button component to replace 15+ duplicate button implementations
 * Consolidates AdminPaginationButton, EditProfileButton, CleanupButton, etc.
 */

import React from 'react';
import { cn } from '../../utils';

// ============================================================================
// BUTTON VARIANT TYPES
// ============================================================================

export const BUTTON_VARIANTS = {
  PRIMARY: 'primary',
  SECONDARY: 'secondary',
  TERTIARY: 'tertiary',
  GHOST: 'ghost',
  LINK: 'link',
  DESTRUCTIVE: 'destructive',
  SUCCESS: 'success',
  WARNING: 'warning',
  PANCAKE: 'pancake',
  PANCAKE_SECONDARY: 'pancakeSecondary',
  PANCAKE_OUTLINE: 'pancakeOutline',
  PANCAKE_GHOST: 'pancakeGhost',
  PANCAKE_LINK: 'pancakeLink'
} as const;

export type ButtonVariant = typeof BUTTON_VARIANTS[keyof typeof BUTTON_VARIANTS];

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

const PRIMARY_BG = 'bg-[hsl(var(--primary))] hover:bg-[hsl(var(--primary))]/90';
const PRIMARY_TEXT = 'text-[hsl(var(--primary-foreground))]';
const PRIMARY_BORDER = 'border border-[hsl(var(--primary))] hover:border-[hsl(var(--primary))]/90';
const PRIMARY_RING = 'focus:ring-[hsl(var(--primary))]';
const SHADOW_HOVER = 'shadow-sm hover:shadow-md';

const TRANSPARENT_BG = 'bg-transparent';
const BORDER_TRANSPARENT = 'border border-transparent';
const PANCAKE_COMMON = 'uppercase tracking-wider focus:ring-yellow-500';
const PANCAKE_SHADOW = 'shadow-xl hover:shadow-2xl';

const variantStyles = {
  [BUTTON_VARIANTS.PRIMARY]: [
    PRIMARY_BG,
    PRIMARY_TEXT,
    PRIMARY_BORDER,
    SHADOW_HOVER,
    PRIMARY_RING
  ],
  [BUTTON_VARIANTS.SECONDARY]: [
    'bg-[hsl(var(--secondary))] hover:bg-[hsl(var(--secondary))]/80',
    'text-[hsl(var(--secondary-foreground))]',
    'border border-[hsl(var(--secondary))]/30 hover:border-[hsl(var(--secondary))]/50',
    'focus:ring-[hsl(var(--secondary))]'
  ],
  [BUTTON_VARIANTS.TERTIARY]: [
    'bg-[hsl(var(--card))] hover:bg-[hsl(var(--accent))]',
    'text-[hsl(var(--foreground))]',
    'border border-[hsl(var(--border))] hover:border-[hsl(var(--border))]/80',
    'focus:ring-[hsl(var(--foreground))]/20'
  ],
  [BUTTON_VARIANTS.GHOST]: [
    `${TRANSPARENT_BG} hover:bg-[hsl(var(--accent))]`,
    'text-[hsl(var(--foreground))] hover:text-[hsl(var(--foreground))]/90',
    BORDER_TRANSPARENT,
    'focus:ring-[hsl(var(--foreground))]/20'
  ],
  [BUTTON_VARIANTS.LINK]: [
    TRANSPARENT_BG,
    'text-[hsl(var(--primary))] hover:text-[hsl(var(--primary))]/80',
    BORDER_TRANSPARENT,
    'underline hover:no-underline',
    PRIMARY_RING,
    'shadow-none'
  ],
  [BUTTON_VARIANTS.DESTRUCTIVE]: [
    'bg-[hsl(var(--destructive))] hover:bg-[hsl(var(--destructive))]/90',
    'text-[hsl(var(--destructive-foreground))]',
    'border border-[hsl(var(--destructive))] hover:border-[hsl(var(--destructive))]/90',
    SHADOW_HOVER,
    'focus:ring-[hsl(var(--destructive))]'
  ],
  [BUTTON_VARIANTS.SUCCESS]: [
    'bg-green-600 hover:bg-green-700',
    'text-white',
    'border border-green-600 hover:border-green-700',
    SHADOW_HOVER,
    'focus:ring-green-500'
  ],
  [BUTTON_VARIANTS.WARNING]: [
    'bg-yellow-600 hover:bg-yellow-700',
    'text-white',
    'border border-yellow-600 hover:border-yellow-700',
    SHADOW_HOVER,
    'focus:ring-yellow-500'
  ],
  [BUTTON_VARIANTS.PANCAKE]: [
    'bg-gradient-to-r from-yellow-400 via-yellow-500 to-orange-500',
    'text-black',
    'border-0',
    'hover:from-yellow-300 hover:to-orange-400',
    PANCAKE_SHADOW,
    'font-normal',
    PANCAKE_COMMON
  ],
  [BUTTON_VARIANTS.PANCAKE_SECONDARY]: [
    'bg-gradient-to-r from-blue-600 to-blue-700',
    'text-white',
    'border-0',
    'hover:from-blue-500 hover:to-blue-600',
    PANCAKE_SHADOW,
    'font-normal uppercase tracking-wider',
    'focus:ring-blue-500'
  ],
  [BUTTON_VARIANTS.PANCAKE_OUTLINE]: [
    'border-2 border-yellow-400',
    TRANSPARENT_BG,
    'text-yellow-600 dark:text-yellow-400',
    'hover:bg-yellow-400 hover:text-black',
    'font-light',
    PANCAKE_COMMON
  ],
  [BUTTON_VARIANTS.PANCAKE_GHOST]: [
    'bg-transparent',
    'text-yellow-600 dark:text-yellow-400',
    'border border-transparent',
    'hover:bg-yellow-400/20 hover:text-yellow-600',
    'dark:hover:text-yellow-400',
    'font-light',
    PANCAKE_COMMON
  ],
  [BUTTON_VARIANTS.PANCAKE_LINK]: [
    TRANSPARENT_BG,
    'text-yellow-600 dark:text-yellow-400',
    'border border-transparent',
    'underline-offset-4 hover:underline',
    'hover:text-yellow-500 dark:hover:text-yellow-300',
    'font-light',
    PANCAKE_COMMON,
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

// Loading spinner component
function LoadingSpinner({ size }: { size: ButtonSize }) {
  return (
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
}

// ============================================================================
// BASE BUTTON COMPONENT
// ============================================================================

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface ButtonContentProps {
  loading: boolean;
  loadingText?: string;
  isIconOnly: boolean;
  icon?: React.ReactNode;
  size: ButtonSize;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  children?: React.ReactNode;
}

const ButtonContent: React.FC<ButtonContentProps> = ({
  loading, loadingText, isIconOnly, icon, size, leftIcon, rightIcon, children
}) => {
  if (loading) {
    return (
      <span className="flex items-center justify-center">
        <LoadingSpinner size={size} />
        {typeof loadingText === 'string' && loadingText !== '' && <span className="ml-2">{loadingText}</span>}
      </span>
    );
  }

  if (isIconOnly) {
    return <span className={iconSizes[size]}>{icon}</span>;
  }

  return (
    <>
      {leftIcon !== null && leftIcon !== undefined && <span className={cn(iconSizes[size], 'mr-2')}>{leftIcon}</span>}
      {children}
      {rightIcon !== null && rightIcon !== undefined && <span className={cn(iconSizes[size], 'ml-2')}>{rightIcon}</span>}
    </>
  );
};

interface ButtonClassesOptions {
  variant: ButtonVariant;
  size: ButtonSize;
  shape: ButtonShape;
  fullWidth: boolean;
  active: boolean;
  isDisabled: boolean;
  isIconOnly: boolean;
}

const getButtonClasses = ({
  variant, size, shape, fullWidth, active, isDisabled, isIconOnly
}: ButtonClassesOptions) => {
  return cn(
    'inline-flex items-center justify-center font-medium transition-all duration-200 select-none',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-[hsl(var(--background))]',
    variantStyles[variant],
    sizeStyles[size],
    shapeStyles[shape],
    fullWidth && 'w-full',
    active && 'ring-2 ring-offset-2',
    isDisabled && 'opacity-50 cursor-not-allowed pointer-events-none',
    isIconOnly && 'p-0'
  );
};

// ============================================================================
// HELPERS
// ============================================================================

const getButtonComponent = (as: React.ElementType | undefined, href: string | undefined): React.ElementType => {
  return as ?? (href !== undefined && href !== '' ? 'a' : 'button');
};

const getIsIconOnly = (icon: React.ReactNode, children: React.ReactNode): boolean => {
  return (icon !== null && icon !== undefined) && (children === null || children === undefined);
};

// ============================================================================
// BASE BUTTON COMPONENT
// ============================================================================

export const BaseButton = React.forwardRef<HTMLButtonElement, BaseButtonProps>((props, ref) => {
  const {
    variant = 'primary',
    size = 'md',
    shape = 'rounded',
    loading = false,
    disabled = false,
    active = false,
    fullWidth = false,
    type = 'button',
    as,
    href,
    target,
    className,
    loadingText,
    leftIcon,
    rightIcon,
    icon,
    children,
    ...rest
  } = props;

  const isDisabled = disabled || loading;
  const Component = getButtonComponent(as, href);
  const isIconOnly = getIsIconOnly(icon, children);

  const baseClasses = getButtonClasses({
    variant, size, shape, fullWidth, active, isDisabled, isIconOnly
  });

  const componentProps: Record<string, unknown> = {
    ref,
    className: cn(baseClasses, className),
    'aria-disabled': isDisabled,
    'aria-pressed': active,
    ...rest
  };

  if (Component === 'button') {
    componentProps.type = type;
    componentProps.disabled = isDisabled;
  } else if (Component === 'a') {
    componentProps.href = href;
    componentProps.target = target;
  }

  return (
    <Component {...componentProps}>
      <ButtonContent
        loading={loading}
        loadingText={loadingText}
        isIconOnly={isIconOnly}
        icon={icon}
        size={size}
        leftIcon={leftIcon}
        rightIcon={rightIcon}
      >
        {children}
      </ButtonContent>
    </Component>
  );
});

BaseButton.displayName = 'base-button'

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
  variant = BUTTON_VARIANTS.GHOST,
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
  const getActionVariant = (act: ActionButtonProps['action']): ButtonVariant => {
    switch (act) {
      case 'create':
      case 'save':
      case 'submit':
        return BUTTON_VARIANTS.PRIMARY
      case 'edit':
        return BUTTON_VARIANTS.SECONDARY
      case 'delete':
        return BUTTON_VARIANTS.DESTRUCTIVE
      case 'cancel':
      case 'reset':
        return BUTTON_VARIANTS.GHOST
      default:
        return BUTTON_VARIANTS.SECONDARY
    }
  }

  const getActionIcon = (act: ActionButtonProps['action']) => {
    switch (act) {
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
      {children ?? action.charAt(0).toUpperCase() + action.slice(1)}
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
    if (direction === 'previous') { return '‹' }
    if (direction === 'next') { return '›' }
    return null
  }

  return (
    <BaseButton
      ref={ref}
      variant={current ? BUTTON_VARIANTS.PRIMARY : BUTTON_VARIANTS.GHOST}
      size="sm"
      shape="rounded"
      active={current}
      leftIcon={direction === 'previous' ? getIcon() : undefined}
      rightIcon={direction === 'next' ? getIcon() : undefined}
      {...props}
    >
      {children ?? page}
    </BaseButton>
  )
})

PaginationButton.displayName = 'pagination-button'

export default BaseButton