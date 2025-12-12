/**
 * ADMIN INPUT COMPONENT
 * Migrated to use unified shared UI component
 */

import * as React from "react"
import { Input as SharedInput, type InputProps as SharedInputProps, inputVariants as sharedInputVariants } from "../../../../shared/components/ui/input"

export interface InputProps extends SharedInputProps {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, variant = "wp", ...props }, ref) => {
    return (
      <SharedInput
        ref={ref}
        variant={variant as any} // Cast if variants mismatch slightly, but generally compatible
        className={className}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

// Export variants for compatibility if needed, though mostly used internally
export const adminInputVariants = sharedInputVariants 

export { Input }
