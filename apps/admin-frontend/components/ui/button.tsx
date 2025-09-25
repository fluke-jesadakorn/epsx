/**
 * ADMIN FRONTEND BUTTON COMPONENT
 * Migrated to use shared PancakeButton with backward compatibility
 */

import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { PancakeButton, type PancakeButtonProps } from "../../../../shared/components"
import { cn } from "@/lib/utils"

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

// Keep the same interface for seamless migration
export interface ButtonProps extends PancakeButtonProps {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, asChild = false, ...props }, ref) => {
    if (asChild) {
      return (
        <Slot className={cn("", className)}>
          <PancakeButton ref={ref} {...props} />
        </Slot>
      )
    }
    
    return (
      <PancakeButton
        ref={ref}
        className={className}
        {...props}
      />
    )
  }
)

Button.displayName = "Button"

// Legacy export for backward compatibility
export const buttonVariants = {
  default: "default",
  destructive: "destructive", 
  outline: "outline",
  secondary: "secondary",
  ghost: "ghost",
  link: "link",
  pancake: "pancake",
  wp: "wp"
} as const

export { Button }