/**
 * BASE TABS COMPONENT
 * Unified tabs system consolidating tab navigation with state management
 * Replaces duplicate Tabs implementations across apps
 */

"use client"

import * as React from "react"
import { cn } from '../../utils'

// ============================================================================
// TABS CONTEXT
// ============================================================================

interface TabsContextValue {
  value: string
  onValueChange: (value: string) => void
  orientation?: 'horizontal' | 'vertical'
  size?: 'sm' | 'md' | 'lg'
  variant?: 'default' | 'pills' | 'underline' | 'cards'
}

const TabsContext = React.createContext<TabsContextValue | null>(null)

export const useTabs = () => {
  const context = React.useContext(TabsContext)
  if (!context) {
    throw new Error('Tabs components must be used within a Tabs provider')
  }
  return context
}

// ============================================================================
// BASE TABS COMPONENT
// ============================================================================

export interface BaseTabsProps {
  children: React.ReactNode
  value: string
  onValueChange: (value: string) => void
  orientation?: 'horizontal' | 'vertical'
  size?: 'sm' | 'md' | 'lg'
  variant?: 'default' | 'pills' | 'underline' | 'cards'
  className?: string
}

export const BaseTabs = React.forwardRef<HTMLDivElement, BaseTabsProps>(
  ({
    children,
    value,
    onValueChange,
    orientation = 'horizontal',
    size = 'md',
    variant = 'default',
    className,
    ...props
  }, ref) => {
    const contextValue = React.useMemo(() => ({
      value,
      onValueChange,
      orientation,
      size,
      variant
    }), [value, onValueChange, orientation, size, variant])

    return (
      <TabsContext.Provider value={contextValue}>
        <div
          ref={ref}
          className={cn(
            'tabs-root w-full',
            orientation === 'vertical' && 'flex',
            className
          )}
          data-orientation={orientation}
          {...props}
        >
          {children}
        </div>
      </TabsContext.Provider>
    )
  }
)
BaseTabs.displayName = "BaseTabs"

// ============================================================================
// TABS LIST COMPONENT
// ============================================================================

export interface TabsListProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
}

const TabsList = React.forwardRef<HTMLDivElement, TabsListProps>(
  ({ className, children, ...props }, ref) => {
    const { orientation, variant, size } = useTabs()

    const variantClasses = {
      default: [
        'bg-gray-100 dark:bg-gray-800',
        'border border-gray-200 dark:border-gray-700',
        'rounded-md p-1'
      ],
      pills: [
        'bg-gray-100 dark:bg-gray-800',
        'rounded-full p-1'
      ],
      underline: [
        'border-b border-gray-200 dark:border-gray-700',
        'bg-transparent'
      ],
      cards: [
        'border-b border-gray-200 dark:border-gray-700',
        'bg-transparent space-x-1'
      ]
    }

    const orientationClasses = {
      horizontal: 'flex flex-row',
      vertical: 'flex flex-col space-y-1 w-48 flex-shrink-0'
    }

    const baseClasses = [
      'tabs-list',
      'text-gray-500 dark:text-gray-400',
      orientationClasses[orientation],
      ...variantClasses[variant]
    ].filter(Boolean).flat()

    return (
      <div
        ref={ref}
        className={cn(baseClasses, className)}
        role="tablist"
        aria-orientation={orientation}
        {...props}
      >
        {children}
      </div>
    )
  }
)
TabsList.displayName = "TabsList"

// ============================================================================
// TABS TRIGGER COMPONENT
// ============================================================================

export interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value: string
  children: React.ReactNode
  icon?: React.ReactNode
  badge?: number | string
  disabled?: boolean
}

const TabsTrigger = React.forwardRef<HTMLButtonElement, TabsTriggerProps>(
  ({
    value,
    children,
    icon,
    badge,
    disabled,
    className,
    ...props
  }, ref) => {
    const { value: selectedValue, onValueChange, variant, size, orientation } = useTabs()
    const isActive = selectedValue === value

    const sizeClasses = {
      sm: 'px-2 py-1 text-sm',
      md: 'px-3 py-2 text-sm',
      lg: 'px-4 py-3 text-base'
    }

    const variantClasses = {
      default: {
        base: 'rounded-md transition-all duration-200',
        active: 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm',
        inactive: 'hover:bg-white/50 dark:hover:bg-gray-700/50 hover:text-gray-700 dark:hover:text-gray-300'
      },
      pills: {
        base: 'rounded-full transition-all duration-200',
        active: 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm',
        inactive: 'hover:bg-white/50 dark:hover:bg-gray-700/50 hover:text-gray-700 dark:hover:text-gray-300'
      },
      underline: {
        base: 'border-b-2 border-transparent transition-all duration-200 rounded-none',
        active: 'border-blue-600 dark:border-blue-400 text-blue-600 dark:text-blue-400',
        inactive: 'hover:border-gray-300 dark:hover:border-gray-600 hover:text-gray-700 dark:hover:text-gray-300'
      },
      cards: {
        base: 'border border-transparent rounded-t-md -mb-px transition-all duration-200',
        active: 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 text-gray-900 dark:text-gray-100 shadow-sm',
        inactive: 'hover:bg-gray-50 dark:hover:bg-gray-800/50 hover:text-gray-700 dark:hover:text-gray-300'
      }
    }

    const orientationClasses = {
      horizontal: variant === 'underline' ? 'flex-1' : '',
      vertical: 'w-full justify-start'
    }

    const currentVariant = variantClasses[variant]

    const baseClasses = [
      'tabs-trigger',
      'inline-flex items-center justify-center',
      'font-medium whitespace-nowrap',
      'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
      'disabled:opacity-50 disabled:pointer-events-none',
      
      // Size
      sizeClasses[size],
      
      // Orientation
      orientationClasses[orientation],
      
      // Variant base
      currentVariant.base,
      
      // Active/inactive state
      isActive ? currentVariant.active : currentVariant.inactive
    ].filter(Boolean)

    return (
      <button
        ref={ref}
        type="button"
        role="tab"
        aria-selected={isActive}
        aria-controls={`tabs-content-${value}`}
        data-state={isActive ? 'active' : 'inactive'}
        className={cn(baseClasses, className)}
        onClick={() => onValueChange(value)}
        disabled={disabled}
        {...props}
      >
        {icon && (
          <span className="mr-2 flex-shrink-0">
            {icon}
          </span>
        )}
        
        <span className="flex-1">
          {children}
        </span>
        
        {badge && (
          <span className={cn(
            'ml-2 inline-flex items-center justify-center',
            'px-1.5 py-0.5 text-xs font-medium rounded-full',
            isActive 
              ? 'bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
          )}>
            {typeof badge === 'number' && badge > 9 ? '9+' : badge}
          </span>
        )}
      </button>
    )
  }
)
TabsTrigger.displayName = "TabsTrigger"

// ============================================================================
// TABS CONTENT COMPONENT
// ============================================================================

export interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string
  children: React.ReactNode
  forceMount?: boolean
}

const TabsContent = React.forwardRef<HTMLDivElement, TabsContentProps>(
  ({ value, children, forceMount = false, className, ...props }, ref) => {
    const { value: selectedValue, orientation } = useTabs()
    const isActive = selectedValue === value
    const shouldRender = forceMount || isActive

    if (!shouldRender) return null

    return (
      <div
        ref={ref}
        id={`tabs-content-${value}`}
        role="tabpanel"
        aria-labelledby={`tabs-trigger-${value}`}
        data-state={isActive ? 'active' : 'inactive'}
        className={cn(
          'tabs-content',
          'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
          orientation === 'vertical' ? 'flex-1 ml-6' : 'mt-4',
          !isActive && 'hidden',
          className
        )}
        tabIndex={0}
        {...props}
      >
        {children}
      </div>
    )
  }
)
TabsContent.displayName = "TabsContent"

// ============================================================================
// CONVENIENCE HOOKS
// ============================================================================

/**
 * Hook to get current tab state
 */
const useTabsState = () => {
  const { value, onValueChange } = useTabs()
  return { value, onValueChange }
}

/**
 * Hook to check if specific tab is active
 */
const useTabActive = (tabValue: string) => {
  const { value } = useTabs()
  return value === tabValue
}

// ============================================================================
// SPECIALIZED TAB COMPONENTS
// ============================================================================

/**
 * Icon Tab - for icon-only tabs
 */
export interface IconTabProps extends Omit<TabsTriggerProps, 'children'> {
  icon: React.ReactNode
  label: string // For accessibility
}

const IconTab = React.forwardRef<HTMLButtonElement, IconTabProps>(
  ({ icon, label, ...props }, ref) => {
    return (
      <TabsTrigger
        ref={ref}
        icon={icon}
        aria-label={label}
        title={label}
        {...props}
      >
        <span className="sr-only">{label}</span>
      </TabsTrigger>
    )
  }
)
IconTab.displayName = "IconTab"

/**
 * Counter Tab - tab with counter badge
 */
export interface CounterTabProps extends TabsTriggerProps {
  count: number
  showZero?: boolean
}

const CounterTab = React.forwardRef<HTMLButtonElement, CounterTabProps>(
  ({ count, showZero = false, ...props }, ref) => {
    const displayCount = count > 0 || showZero ? count : undefined
    
    return (
      <TabsTrigger
        ref={ref}
        badge={displayCount}
        {...props}
      />
    )
  }
)
CounterTab.displayName = "CounterTab"

// ============================================================================
// EXPORTS
// ============================================================================

export {
  BaseTabs as Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  IconTab,
  CounterTab,
  useTabsState,
  useTabActive
}

export default BaseTabs