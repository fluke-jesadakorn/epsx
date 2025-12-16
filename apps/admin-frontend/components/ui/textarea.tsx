import type { TextareaProps as SharedTextareaProps } from "@/shared/components/ui/textarea"
import { Textarea as SharedTextarea, textareaVariants } from "@/shared/components/ui/textarea"
import * as React from "react"

// Wrapper to preserve Admin 'wp' default variant
const Textarea = React.forwardRef<HTMLTextAreaElement, SharedTextareaProps>(
  ({ variant = "wp", ...props }, ref) => {
    return <SharedTextarea variant={variant} ref={ref} {...props} />
  }
)
Textarea.displayName = "Textarea"

// Re-export variants as adminTextareaVariants for backward compatibility
export { textareaVariants as adminTextareaVariants, Textarea }
export type { SharedTextareaProps as TextareaProps }
