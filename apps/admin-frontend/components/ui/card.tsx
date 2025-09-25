/**
 * ADMIN FRONTEND CARD COMPONENT
 * Migrated to use shared PancakeCard with backward compatibility
 */

import * as React from "react"
import { 
  PancakeCard, 
  PancakeCardHeader, 
  PancakeCardTitle, 
  PancakeCardDescription, 
  PancakeCardContent, 
  PancakeCardFooter 
} from "../../../../shared/components"
import { cn } from "@/lib/utils"

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

// Keep the same interface for seamless migration
const Card = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, children, onClick, ...props }, ref) => (
  <PancakeCard
    ref={ref}
    variant="elevated" // Use elevated variant to match existing styling
    className={className}
    onClick={onClick ? () => onClick({} as any) : undefined}
    {...(props as any)}
  >
    {children}
  </PancakeCard>
))
Card.displayName = "Card"

const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <PancakeCardHeader
    ref={ref}
    className={className}
    {...props}
  />
))
CardHeader.displayName = "CardHeader"

const CardTitle = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <PancakeCardTitle
    ref={ref}
    className={className}
    {...props}
  />
))
CardTitle.displayName = "CardTitle"

const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <PancakeCardDescription
    ref={ref}
    className={className}
    {...props}
  />
))
CardDescription.displayName = "CardDescription"

const CardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <PancakeCardContent
    ref={ref}
    className={className}
    {...props}
  />
))
CardContent.displayName = "CardContent"

const CardFooter = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <PancakeCardFooter
    ref={ref}
    className={className}
    {...props}
  />
))
CardFooter.displayName = "CardFooter"

export { Card, CardHeader, CardFooter, CardTitle, CardDescription, CardContent }