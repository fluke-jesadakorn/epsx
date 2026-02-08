/**
 * PANCAKESWAP THEMED CARD COMPONENTS
 * PancakeSwap-themed wrapper around BaseCard for admin-frontend
 * Replaces apps/admin-frontend/components/ui/card.tsx
 */

"use client"

import * as React from "react"
import { cn } from '../../utils'
import { BaseCard, type BaseCardProps } from './base-card'

// ============================================================================
// PANCAKESWAP CARD SYSTEM
// ============================================================================

/**
 * Main PancakeSwap Card component
 */
export interface PancakeCardProps extends Omit<BaseCardProps, 'variant'> {
  variant?: 'default' | 'elevated' | 'outlined'
}

export const Card = React.forwardRef<HTMLDivElement, PancakeCardProps & React.HTMLAttributes<HTMLDivElement>>(
  ({ variant = 'elevated', className, ...props }, ref) => {
    const baseVariant = variant === 'default' ? 'pancake' :
      variant === 'elevated' ? 'pancakeElevated' :
        'pancakeOutlined'

    return (
      <BaseCard
        ref={ref}
        variant={baseVariant}
        className={cn(
          // Apply transition-all for smooth animations
          'transition-all duration-300',
          className
        )}
        {...props}
      />
    )
  }
)
Card.displayName = "Card"

/**
 * Card Header with PancakeSwap styling
 */
export const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex flex-col space-y-1.5 p-6", className)}
    {...props}
  />
))
CardHeader.displayName = "Cardheader"

/**
 * Card Title with gradient text styling
 */
export const CardTitle = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h3
    ref={ref}
    className={cn(
      "text-2xl font-bold leading-none tracking-tight",
      "bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent",
      className
    )}
    {...props}
  />
))
CardTitle.displayName = "CardTitle"

/**
 * Card Description
 */
export const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
CardDescription.displayName = "CardDescription"

/**
 * Card Content
 */
export const CardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn("p-6 pt-0", className)} {...props} />
))
CardContent.displayName = "CardContent"

/**
 * Card Footer
 */
export const CardFooter = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex items-center p-6 pt-0", className)}
    {...props}
  />
))
CardFooter.displayName = "CardFooter"

// ============================================================================
// SPECIALIZED PANCAKE CARDS
// ============================================================================

/**
 * Stats Card - for displaying metrics with PancakeSwap theming
 */
export interface StatsCardProps extends PancakeCardProps {
  title: string
  value: string | number
  change?: string
  changeType?: 'positive' | 'negative' | 'neutral'
  icon?: React.ReactNode
}

export const StatsCard = React.forwardRef<HTMLDivElement, StatsCardProps>(({
  title,
  value,
  change,
  changeType = 'neutral',
  icon,
  className,
  ...props
}, ref) => {
  const changeColorMap = {
    positive: 'text-green-600 dark:text-green-400',
    negative: 'text-red-600 dark:text-red-400',
    neutral: 'text-gray-600 dark:text-gray-400'
  }

  return (
    <Card ref={ref} className={cn("p-6", className)} {...props}>
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <p className="text-sm font-medium text-muted-foreground">{title}</p>
          <div className="flex items-center space-x-2">
            <p className="text-2xl font-bold">{value}</p>
            {change && (
              <span className={cn("text-sm font-medium", changeColorMap[changeType])}>
                {change}
              </span>
            )}
          </div>
        </div>
        {icon && (
          <div className="text-muted-foreground">
            {icon}
          </div>
        )}
      </div>
    </Card>
  )
})
StatsCard.displayName = "stats-card"

/**
 * Analytics Card - for displaying charts and analytics
 */
export interface AnalyticsCardProps extends PancakeCardProps {
  title: string
  subtitle?: string
}

export const AnalyticsCard = React.forwardRef<HTMLDivElement, AnalyticsCardProps>(({
  title,
  subtitle,
  children,
  className,
  ...props
}, ref) => {
  return (
    <Card ref={ref} className={cn("", className)} {...props}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {subtitle && <CardDescription>{subtitle}</CardDescription>}
      </CardHeader>
      <CardContent>
        {children}
      </CardContent>
    </Card>
  )
})
AnalyticsCard.displayName = "analytics-card"

export { Card as PancakeCard }
