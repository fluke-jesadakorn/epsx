/**
 * FRONTEND BUTTON COMPONENT
 * Migrated to use unified shared UI component
 */

import * as React from "react"
import { Button as SharedButton, buttonVariants, type ButtonProps as SharedButtonProps } from "../../../../shared/components/ui/button"

export interface ButtonProps extends SharedButtonProps {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <SharedButton
        ref={ref}
        variant={variant}
        size={size}
        className={className}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }
