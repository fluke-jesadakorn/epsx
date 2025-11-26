/**
 * PANCAKESWAP THEMED BUTTON
 * PancakeSwap-themed wrapper around BaseButton for admin-frontend
 * Replaces apps/admin-frontend/components/ui/button.tsx
 */

import React from 'react'
import { BaseButton, type BaseButtonProps, type ButtonVariant } from './BaseButton'

// ============================================================================
// PANCAKESWAP VARIANT MAPPING
// ============================================================================

// Map original admin button variants to new pancake variants
type AdminButtonVariant = 
  | 'default'
  | 'destructive'
  | 'outline'
  | 'secondary'
  | 'ghost'
  | 'link'
  | 'pancake'
  | 'wp'

const variantMap: Record<AdminButtonVariant, ButtonVariant> = {
  default: 'pancake', // PancakeSwap primary style
  destructive: 'destructive',
  outline: 'pancakeOutline',
  secondary: 'pancakeSecondary',
  ghost: 'pancakeGhost',
  link: 'pancakeLink',
  pancake: 'pancake',
  wp: 'pancakeSecondary' // "wp" likely means Web3/wallet, use blue theme
}

// ============================================================================
// PANCAKESWAP BUTTON COMPONENT
// ============================================================================

export interface PancakeButtonProps extends Omit<BaseButtonProps, 'variant'> {
  variant?: AdminButtonVariant
  asChild?: boolean
}

export const PancakeButton = React.forwardRef<HTMLButtonElement, PancakeButtonProps>(({
  variant = 'default',
  children,
  className,
  ...props
}, ref) => {
  // Map admin variant to base button variant
  const baseVariant = variantMap[variant]

  return (
    <BaseButton
      ref={ref}
      variant={baseVariant}
      className={className}
      {...props}
    >
      {children}
    </BaseButton>
  )
})

PancakeButton.displayName = 'PancakeButton'

// ============================================================================
// LEGACY COMPATIBILITY EXPORTS
// ============================================================================

// For easy migration from admin button.tsx
export { PancakeButton as Button }
export type { PancakeButtonProps as ButtonProps }

// Export variant types for compatibility
export const buttonVariants = {
  default: 'pancake',
  destructive: 'destructive',
  outline: 'pancakeOutline',
  secondary: 'pancakeSecondary',
  ghost: 'pancakeGhost',
  link: 'pancakeLink',
  pancake: 'pancake',
  wp: 'pancakeSecondary'
} as const

export default PancakeButton