/**
 * FRONTEND BUTTON COMPONENT
 * Migrated to use shared BaseButton with backward compatibility
 */

import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { BaseButton, type BaseButtonProps } from "../../../../shared/components"
import { cn } from "@/lib/utils"

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

// Map frontend variants to BaseButton variants
type FrontendVariant = 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
type FrontendSize = 'default' | 'sm' | 'lg' | 'icon'

const variantMap: Record<FrontendVariant, BaseButtonProps['variant']> = {
  default: 'primary',
  destructive: 'destructive',
  outline: 'tertiary', // outline maps to tertiary
  secondary: 'secondary',
  ghost: 'ghost',
  link: 'link'
}

const sizeMap: Record<FrontendSize, BaseButtonProps['size']> = {
  default: 'md',
  sm: 'sm',
  lg: 'lg',
  icon: 'md' // icon buttons handled by BaseButton's icon prop
}

export interface ButtonProps extends Omit<BaseButtonProps, 'variant' | 'size'> {
  variant?: FrontendVariant
  size?: FrontendSize
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'default', size = 'default', asChild = false, ...props }, ref) => {
    const baseVariant = variantMap[variant]
    const baseSize = sizeMap[size]
    
    if (asChild) {
      return (
        <Slot className={cn("", className)}>
          <BaseButton
            ref={ref}
            variant={baseVariant}
            size={baseSize}
            {...props}
          />
        </Slot>
      )
    }
    
    return (
      <BaseButton
        ref={ref}
        variant={baseVariant}
        size={baseSize}
        className={className}
        {...props}
      />
    )
  }
)

Button.displayName = "Button"

// Legacy export for backward compatibility
export const buttonVariants = {
  default: "primary",
  destructive: "destructive",
  outline: "tertiary",
  secondary: "secondary", 
  ghost: "ghost",
  link: "link"
} as const

export { Button }