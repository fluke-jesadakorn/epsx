/**
 * FRONTEND INPUT COMPONENT
 * Migrated to use unified shared UI component
 */

import * as React from "react"
import { Input as SharedInput, type InputProps as SharedInputProps } from "../../../../shared/components/ui/input"

export interface InputProps extends SharedInputProps {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, error, ...props }, ref) => {
    return (
      <SharedInput
        ref={ref}
        type={type}
        className={className}
        error={error}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }
