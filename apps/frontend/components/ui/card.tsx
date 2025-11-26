/**
 * FRONTEND CARD COMPONENT
 * Migrated to use shared BaseCard with backward compatibility
 */

import * as React from "react"
import { BaseCard, type BaseCardProps } from "../../../../shared/components"
import { cn } from "@/lib/utils"

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

export type CardProps = React.HTMLAttributes<HTMLDivElement>

const Card = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, children, onClick, onDoubleClick, onMouseEnter, onMouseLeave, ...props }, ref) => (
  <BaseCard
    ref={ref as any}
    variant="default" // Use default variant for standard styling
    className={cn("rounded-lg", className)}
    children={children || null}
    onClick={onClick ? () => onClick({} as React.MouseEvent<HTMLDivElement>) : undefined}
    onDoubleClick={onDoubleClick ? () => onDoubleClick({} as React.MouseEvent<HTMLDivElement>) : undefined}
    onMouseEnter={onMouseEnter ? () => onMouseEnter({} as React.MouseEvent<HTMLDivElement>) : undefined}
    onMouseLeave={onMouseLeave ? () => onMouseLeave({} as React.MouseEvent<HTMLDivElement>) : undefined}
    {...props}
  />
))
Card.displayName = "Card"

const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex flex-col space-y-1.5 p-6", className)}
    {...props}
  />
))
CardHeader.displayName = "CardHeader"

const CardTitle = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h3
    ref={ref}
    className={cn(
      "text-2xl font-semibold leading-none tracking-tight",
      className
    )}
    {...props}
  />
))
CardTitle.displayName = "CardTitle"

const CardDescription = React.forwardRef<
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

const CardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn("p-6 pt-0", className)} {...props} />
))
CardContent.displayName = "CardContent"

const CardFooter = React.forwardRef<
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

export { Card, CardHeader, CardFooter, CardTitle, CardDescription, CardContent }