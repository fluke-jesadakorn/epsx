/**
 * FRONTEND INPUT COMPONENT
 * Migrated to use shared BaseInput with backward compatibility
 */

import * as React from "react"
import { BaseInput, type BaseInputProps } from "../../../../shared/components"
import { cn } from "@/lib/utils"

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: boolean
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <BaseInput
        ref={ref}
        type={type}
        variant="default"
        className={cn(
          "rounded-md border border-input bg-background",
          "ring-offset-background file:border-0 file:bg-transparent",
          "file:text-sm file:font-medium placeholder:text-muted-foreground",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
          className
        )}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }