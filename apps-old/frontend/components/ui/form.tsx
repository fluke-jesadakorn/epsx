/**
 * FRONTEND FORM COMPONENT
 * Wraps shared form components with frontend-specific styling
 */

"use client"

import * as React from "react"
import {
  Form as BaseForm,
  FormField as BaseFormField,
  FormItem as BaseFormItem,
  FormLabel as BaseFormLabel,
  FormControl as BaseFormControl,
  FormDescription as BaseFormDescription,
  FormMessage as BaseFormMessage,
  useFormField as useBaseFormField,
} from "@/shared/components/ui/form"
import type { FormProps } from "@/shared/components/ui/form"
import { cn } from "@/lib/utils"
import type {
  FieldPath,
  FieldValues,
  UseFormReturn,
  ControllerProps
} from "react-hook-form"

type FormWrapperProps<TFieldValues extends FieldValues> = UseFormReturn<TFieldValues> & {
  onSubmit?: (data: TFieldValues) => void | Promise<void>
  children: React.ReactNode
  className?: string
}

const Form = <TFieldValues extends FieldValues>({
  children,
  onSubmit,
  className,
  ...formMethods
}: FormWrapperProps<TFieldValues>) => {
  const baseProps = {
    ...formMethods,
    onSubmit,
    className: cn("space-y-4", className),
    children,
  } as FormProps<TFieldValues>
  return <BaseForm {...baseProps} />
}

const FormField = <
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
  ...props
}: ControllerProps<TFieldValues, TName>) => (
   
  <BaseFormField {...(props as any)} />
)

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
