/**
 * BASE NAVIGATION COMPONENTS
 * Unified navigation system consolidating navigation menus, tabs, and mobile navigation
 * Replaces duplicate NavigationMenu, Tabs, MobileBottomNav implementations
 */

"use client"

import * as React from "react"
import { cn } from '../../utils'

// ============================================================================
// NAVIGATION TYPES
// ============================================================================

export type NavigationVariant = 'horizontal' | 'vertical' | 'mobile' | 'tabs' | 'breadcrumb'
export type NavigationSize = 'sm' | 'md' | 'lg'

export interface NavigationItem {
  id: string
  label: string
  href?: string
  icon?: React.ReactNode
  badge?: number | string
  disabled?: boolean
  active?: boolean
  children?: NavigationItem[]
}

// ============================================================================
// NAVIGATION CONTEXT
// ============================================================================

interface NavigationContextValue {
  activeItem?: string
  onItemChange?: (itemId: string) => void
  variant?: NavigationVariant
  size?: NavigationSize
}

const NavigationContext = React.createContext<NavigationContextValue>({})

export const useNavigation = () => {
  const context = React.useContext(NavigationContext)
  return context
}

// ============================================================================
// BASE NAVIGATION COMPONENT
// ============================================================================

export interface BaseNavigationProps {
  children: React.ReactNode
  variant?: NavigationVariant
  size?: NavigationSize
  activeItem?: string
  onItemChange?: (itemId: string) => void
  className?: string
  orientation?: 'horizontal' | 'vertical'
}

export const BaseNavigation = React.forwardRef<HTMLElement, BaseNavigationProps>(
  ({
    children,
    variant = 'horizontal',
    size = 'md',
    activeItem,
    onItemChange,
    className,
    orientation,
    ...props
  }, ref) => {
    const contextValue = React.useMemo(() => ({
      activeItem,
      onItemChange,
      variant,
      size
    }), [activeItem, onItemChange, variant, size])

    const baseClasses = [
      'navigation-base',
      variant === 'horizontal' && 'flex items-center space-x-1',
      variant === 'vertical' && 'flex flex-col space-y-1',
      variant === 'mobile' && 'flex items-center justify-around',
      variant === 'tabs' && 'inline-flex items-center rounded-md bg-gray-100 dark:bg-gray-800 p-1',
      variant === 'breadcrumb' && 'flex items-center space-x-2'
    ].filter(Boolean)

    return (
      <NavigationContext.Provider value={contextValue}>
        <nav
          ref={ref}
          className={cn(baseClasses, className)}
          role="navigation"
          {...props}
        >
          {children}
        </nav>
      </NavigationContext.Provider>
    )
  }
)
BaseNavigation.displayName = "BaseNavigation"

// ============================================================================
// NAVIGATION LIST COMPONENT
// ============================================================================

export interface NavigationListProps extends React.HTMLAttributes<HTMLUListElement> {
  children: React.ReactNode
}

const NavigationList = React.forwardRef<HTMLUListElement, NavigationListProps>(
  ({ className, children, ...props }, ref) => {
    const { variant } = useNavigation()

    const listClasses = [
      'navigation-list flex list-none',
      variant === 'horizontal' && 'flex-row space-x-1',
      variant === 'vertical' && 'flex-col space-y-1',
      variant === 'mobile' && 'flex-row justify-around w-full',
      variant === 'tabs' && 'flex-row space-x-1',
      variant === 'breadcrumb' && 'flex-row items-center space-x-2'
    ].filter(Boolean)

    return (
      <ul
        ref={ref}
        className={cn(listClasses, className)}
        {...props}
      >
        {children}
      </ul>
    )
  }
)
NavigationList.displayName = "NavigationList"

// ============================================================================
// NAVIGATION ITEM COMPONENT
// ============================================================================

export interface NavigationItemProps extends React.HTMLAttributes<HTMLLIElement> {
  children: React.ReactNode
  active?: boolean
  disabled?: boolean
}

const NavigationItem = React.forwardRef<HTMLLIElement, NavigationItemProps>(
  ({ className, children, active, disabled, ...props }, ref) => {
    const { variant } = useNavigation()

    const itemClasses = [
      'navigation-item',
      variant === 'mobile' && 'flex-1',
      disabled && 'opacity-50 pointer-events-none'
    ].filter(Boolean)

    return (
      <li
        ref={ref}
        className={cn(itemClasses, className)}
        {...props}
      >
        {children}
      </li>
    )
  }
)
NavigationItem.displayName = "NavigationItem"

// ============================================================================
// NAVIGATION LINK COMPONENT
// ============================================================================

export interface NavigationLinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  children: React.ReactNode
  active?: boolean
  disabled?: boolean
  icon?: React.ReactNode
  badge?: number | string
  as?: React.ElementType
}

const NavigationLink = React.forwardRef<HTMLAnchorElement, NavigationLinkProps>(
  ({
    className,
    children,
    active,
    disabled,
    icon,
    badge,
    as: Component = 'a',
    ...props
  }, ref) => {
    const { variant, size } = useNavigation()

    const sizeClasses = {
      sm: 'px-2 py-1 text-sm',
      md: 'px-3 py-2 text-sm',
      lg: 'px-4 py-3 text-base'
    }

    const linkClasses = [
      // Base styling
      'navigation-link inline-flex items-center justify-center',
      'font-medium transition-colors duration-200',
      'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
      'rounded-md',
      
      // Size styling
      sizeClasses[size || 'md'],
      
      // Variant-specific styling
      variant === 'horizontal' && [
        'hover:bg-gray-100 dark:hover:bg-gray-800',
        active && 'bg-gray-100 dark:bg-gray-800 text-blue-600 dark:text-blue-400'
      ],
      
      variant === 'vertical' && [
        'w-full justify-start',
        'hover:bg-gray-100 dark:hover:bg-gray-800',
        active && 'bg-gray-100 dark:bg-gray-800 text-blue-600 dark:text-blue-400'
      ],
      
      variant === 'mobile' && [
        'flex-col gap-1 min-w-0 flex-1',
        'text-gray-600 dark:text-gray-400',
        'hover:text-gray-900 dark:hover:text-gray-100',
        active && 'text-blue-600 dark:text-blue-400'
      ],
      
      variant === 'tabs' && [
        'whitespace-nowrap',
        'ring-offset-background',
        active 
          ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm'
          : 'text-gray-600 dark:text-gray-400 hover:bg-white/50 dark:hover:bg-gray-700/50'
      ],
      
      variant === 'breadcrumb' && [
        'hover:text-blue-600 dark:hover:text-blue-400',
        active && 'text-gray-900 dark:text-gray-100 font-semibold'
      ],
      
      // Disabled state
      disabled && [
        'opacity-50',
        'cursor-not-allowed',
        'pointer-events-none'
      ]
    ].filter(Boolean).flat()

    return (
      <Component
        ref={ref}
        className={cn(linkClasses, className)}
        aria-current={active ? 'page' : undefined}
        {...props}
      >
        {icon && (
          <span className={cn(
            'navigation-icon',
            variant === 'mobile' ? 'mb-1' : 'mr-2',
            variant === 'tabs' && !children && 'mr-0'
          )}>
            {icon}
          </span>
        )}
        
        {children && (
          <span className={cn(
            'navigation-label',
            variant === 'mobile' && 'text-xs text-center truncate w-full'
          )}>
            {children}
          </span>
        )}
        
        {badge && (
          <span className={cn(
            'navigation-badge',
            'ml-2 inline-flex items-center justify-center',
            'px-1.5 py-0.5 text-xs font-medium rounded-full',
            'bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-200',
            variant === 'mobile' && 'absolute -top-1 -right-1 ml-0 h-4 w-4 p-0'
          )}>
            {typeof badge === 'number' && badge > 9 ? '9+' : badge}
          </span>
        )}
      </Component>
    )
  }
)
NavigationLink.displayName = "NavigationLink"

// ============================================================================
// NAVIGATION TRIGGER (FOR DROPDOWNS)
// ============================================================================

export interface NavigationTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode
  active?: boolean
  icon?: React.ReactNode
  chevron?: boolean
}

const NavigationTrigger = React.forwardRef<HTMLButtonElement, NavigationTriggerProps>(
  ({
    className,
    children,
    active,
    icon,
    chevron = true,
    ...props
  }, ref) => {
    const { variant, size } = useNavigation()

    const sizeClasses = {
      sm: 'px-2 py-1 text-sm',
      md: 'px-3 py-2 text-sm', 
      lg: 'px-4 py-3 text-base'
    }

    const triggerClasses = [
      'navigation-trigger inline-flex items-center justify-center',
      'font-medium transition-colors duration-200',
      'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
      'rounded-md',
      'hover:bg-gray-100 dark:hover:bg-gray-800',
      
      // Size styling
      sizeClasses[size || 'md'],
      
      // Active state
      active && 'bg-gray-100 dark:bg-gray-800 text-blue-600 dark:text-blue-400'
    ].filter(Boolean)

    return (
      <button
        ref={ref}
        className={cn(triggerClasses, className)}
        {...props}
      >
        {icon && (
          <span className="mr-2">
            {icon}
          </span>
        )}
        {children}
        {chevron && (
          <svg
            className="ml-1 h-3 w-3 transition-transform duration-200 group-data-[state=open]:rotate-180"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        )}
      </button>
    )
  }
)
NavigationTrigger.displayName = "NavigationTrigger"

// ============================================================================
// NAVIGATION CONTENT (FOR DROPDOWNS)
// ============================================================================

export interface NavigationContentProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
  position?: 'left' | 'center' | 'right'
}

const NavigationContent = React.forwardRef<HTMLDivElement, NavigationContentProps>(
  ({ className, children, position = 'left', ...props }, ref) => {
    const contentClasses = [
      'navigation-content',
      'absolute top-full z-50 mt-1',
      'bg-white dark:bg-gray-800',
      'border border-gray-200 dark:border-gray-700',
      'rounded-md shadow-lg',
      'py-1',
      position === 'left' && 'left-0',
      position === 'center' && 'left-1/2 transform -translate-x-1/2',
      position === 'right' && 'right-0'
    ].filter(Boolean)

    return (
      <div
        ref={ref}
        className={cn(contentClasses, className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)
NavigationContent.displayName = "NavigationContent"

// ============================================================================
// BREADCRUMB SEPARATOR
// ============================================================================

export interface BreadcrumbSeparatorProps extends React.HTMLAttributes<HTMLSpanElement> {
  children?: React.ReactNode
}

const BreadcrumbSeparator = React.forwardRef<HTMLSpanElement, BreadcrumbSeparatorProps>(
  ({ className, children = '/', ...props }, ref) => {
    return (
      <span
        ref={ref}
        className={cn('text-gray-400 dark:text-gray-500 mx-2', className)}
        aria-hidden="true"
        {...props}
      >
        {children}
      </span>
    )
  }
)
BreadcrumbSeparator.displayName = "BreadcrumbSeparator"

// ============================================================================
// EXPORTS
// ============================================================================

export {
  BaseNavigation as Navigation,
  NavigationList,
  NavigationItem,
  NavigationLink,
  NavigationTrigger,
  NavigationContent,
  BreadcrumbSeparator
}

export default BaseNavigation