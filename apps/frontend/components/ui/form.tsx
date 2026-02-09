/**
 * FRONTEND FORM COMPONENT
 * Migrated to use shared BaseForm with backward compatibility
 * Replaces React Hook Form duplicate implementation
 */

"use client"

import * as React from "react"
import {
  BaseForm,
  FormField as BaseFormField,
  FormItem as BaseFormItem,
  FormLabel as BaseFormLabel,
  FormControl as BaseFormControl,
  FormDescription as BaseFormDescription,
  FormMessage as BaseFormMessage,
  useFormField as useBaseFormField,
  type BaseFormProps as _BaseFormProps
} from "@/shared/components"
import { cn } from "@/lib/utils"
import type {
  FieldPath,
  FieldValues,
  UseFormReturn,
  ControllerProps
} from "react-hook-form"

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

// Enhanced Form component that handles React Hook Form integration
const Form = <TFieldValues extends FieldValues>({
  children,
  onSubmit,
  className,
  handleSubmit,
  ...props
}: UseFormReturn<TFieldValues> & {
  onSubmit?: (data: TFieldValues) => void
  children: React.ReactNode
  className?: string
}) => {
  return (
    <BaseForm
      onSubmit={onSubmit ? handleSubmit(onSubmit) : (e) => e.preventDefault()}
      className={cn("space-y-4", className)}
      {...(props as Record<string, unknown>)}
    >
      {children}
    </BaseForm>
  );
}

// Form field with React Hook Form Controller integration
const FormField = <
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
  ...props
}: ControllerProps<TFieldValues, TName>) => (
  <BaseFormField {...props} />
)

// Re-export shared components with same names for compatibility
const useFormField = useBaseFormField

function FormItem({ className, children, ...props }: React.ComponentProps<"div">) {
  return (
    <BaseFormItem
      className={cn("grid gap-2", className)}
      children={children}
      {...props}
    />
  )
}

function FormLabel({
  className,
  ...props
}: React.LabelHTMLAttributes<HTMLLabelElement>) {
  return (
    <BaseFormLabel
      className={cn(
        "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
        "data-[error=true]:text-destructive-foreground",
        className
      )}
      {...props}
    />
  )
}

interface FormControlProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
  asChild?: boolean
}

const FormControl = React.forwardRef<HTMLDivElement, FormControlProps>(
  ({ className, ...props }, ref) => {
    return (
      <BaseFormControl
        ref={ref}
        className={cn("relative", className)}
        {...props}
      />
    )
  }
)
FormControl.displayName = "FormControl"

function FormDescription({ className, ...props }: React.ComponentProps<"p">) {
  return (
    <BaseFormDescription
      className={cn("text-muted-foreground text-sm", className)}
      {...props}
    />
  )
}

function FormMessage({ className, ...props }: React.ComponentProps<"p">) {
  return (
    <BaseFormMessage
      className={cn("text-destructive-foreground text-sm", className)}
      {...props}
    />
  )
}

export {
  useFormField,
  Form,
  FormItem,
  FormLabel,
  FormControl,
  FormDescription,
  FormMessage,
  FormField,
}
