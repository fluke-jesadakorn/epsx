/**
 * Form components with React Hook Form integration
 * Standard shadcn-style form system
 */

"use client"

import * as React from "react"
import {
  Controller,
  FormProvider,
  useFormContext,
  type ControllerProps,
  type FieldPath,
  type FieldValues,
  type UseFormReturn,
} from "react-hook-form"
import { cn } from '../../utils'

// ============================================================================
// FORM CONTEXT AND HOOKS
// ============================================================================

interface FormItemContextValue {
  id: string
}

interface FormFieldContextValue<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
  name: TName
}

const FormItemContext = React.createContext<FormItemContextValue | undefined>(undefined)

const FormFieldContext = React.createContext<FormFieldContextValue | undefined>(undefined)

export const useFormField = () => {
  const fieldContext = React.useContext(FormFieldContext)
  const itemContext = React.useContext(FormItemContext)
  const { getFieldState, formState } = useFormContext()

  if (!fieldContext || !itemContext) {
    throw new Error("useFormField should be used within <FormField>")
  }

  const fieldState = getFieldState(fieldContext.name, formState)

  return {
    id: itemContext.id,
    name: fieldContext.name,
    formItemId: `${itemContext.id}-form-item`,
    formDescriptionId: `${itemContext.id}-form-item-description`,
    formMessageId: `${itemContext.id}-form-item-message`,
    ...fieldState,
  }
}

// ============================================================================
// FORM COMPONENT
// ============================================================================

export interface FormProps<TFieldValues extends FieldValues = FieldValues>
  extends UseFormReturn<TFieldValues> {
  children: React.ReactNode
  onSubmit?: (data: TFieldValues) => void | Promise<void>
  className?: string
  noValidate?: boolean
}

export const Form = <TFieldValues extends FieldValues = FieldValues>({
  children,
  onSubmit,
  className,
  noValidate = true,
  ...formMethods
}: FormProps<TFieldValues>) => {
  const handleSubmit = async (data: TFieldValues) => {
    try {
      await onSubmit?.(data)
    } catch {
      // Submission error handled by caller
    }
  }

  return (
    <FormProvider {...formMethods}>
      <form
        onSubmit={onSubmit ? formMethods.handleSubmit(handleSubmit) : (e) => e.preventDefault()}
        className={cn('space-y-6', className)}
        noValidate={noValidate}
      >
        {children}
      </form>
    </FormProvider>
  )
}
Form.displayName = "Form"

// ============================================================================
// FORM FIELD COMPONENTS
// ============================================================================

const FormField = <
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
  ...props
}: ControllerProps<TFieldValues, TName>) => (
  <FormFieldContext.Provider value={{ name: props.name }}>
    <Controller {...props} />
  </FormFieldContext.Provider>
)

export interface FormItemProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
}

const FormItem = React.forwardRef<HTMLDivElement, FormItemProps>(
  ({ className, ...props }, ref) => {
    const id = React.useId()

    return (
      <FormItemContext.Provider value={{ id }}>
        <div
          ref={ref}
          className={cn("space-y-2", className)}
          {...props}
        />
      </FormItemContext.Provider>
    )
  }
)
FormItem.displayName = "FormItem"

export interface FormLabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {
  required?: boolean
}

const FormLabel = React.forwardRef<HTMLLabelElement, FormLabelProps>(
  ({ className, required, children, ...props }, ref) => {
    const { error, formItemId } = useFormField()

    return (
      <label
        ref={ref}
        className={cn(
          "text-sm font-medium leading-none",
          "peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
          error && "text-red-600 dark:text-red-400",
          Boolean(required) && "after:content-['*'] after:ml-0.5 after:text-red-500",
          className
        )}
        htmlFor={formItemId}
        {...props}
      >
        {children}
      </label>
    )
  }
)
FormLabel.displayName = "FormLabel"

export interface FormControlProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
  asChild?: boolean
}

const FormControl = React.forwardRef<HTMLDivElement, FormControlProps>(
  ({ asChild = false, children, className, ...props }, ref) => {
    const { error, formItemId, formDescriptionId, formMessageId } = useFormField()

    const sharedProps = {
      ref,
      ...props,
      id: formItemId,
      "aria-describedby": !error
        ? formDescriptionId
        : `${formDescriptionId} ${formMessageId}`,
      "aria-invalid": Boolean(error),
      className: cn("relative", className)
    }

    if (asChild) {
      return React.cloneElement(children as React.ReactElement, sharedProps)
    }

    return <div {...sharedProps}>{children}</div>
  }
)
FormControl.displayName = "FormControl"

export interface FormDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> { }

const FormDescription = React.forwardRef<HTMLParagraphElement, FormDescriptionProps>(
  ({ className, ...props }, ref) => {
    const { formDescriptionId } = useFormField()

    return (
      <p
        ref={ref}
        id={formDescriptionId}
        className={cn("text-sm text-gray-500 dark:text-gray-400", className)}
        {...props}
      />
    )
  }
)
FormDescription.displayName = "FormDescription"

export interface FormMessageProps extends React.HTMLAttributes<HTMLParagraphElement> { }

const FormMessage = React.forwardRef<HTMLParagraphElement, FormMessageProps>(
  ({ className, children, ...props }, ref) => {
    const { error, formMessageId } = useFormField()
    const body = error ? String(error.message) : children

    const hasBody = body !== null && body !== undefined && body !== '';
    if (!hasBody) {
      return null
    }

    return (
      <p
        ref={ref}
        id={formMessageId}
        className={cn("text-sm text-red-600 dark:text-red-400", className)}
        role="alert"
        {...props}
      >
        {body}
      </p>
    )
  }
)
FormMessage.displayName = "FormMessage"

// ============================================================================
// CONVENIENCE WRAPPER
// ============================================================================

export interface FormFieldWrapperProps {
  label: string
  required?: boolean
  description?: string
  error?: string
  className?: string
  children: React.ReactNode
}

const FormFieldWrapper: React.FC<FormFieldWrapperProps> = ({
  label,
  required,
  description,
  error,
  className,
  children
}) => {
  return (
    <FormItem className={className}>
      <FormLabel required={required}>
        {label}
      </FormLabel>
      <FormControl>
        {children}
      </FormControl>
      {Boolean(description) && (
        <FormDescription>
          {description}
        </FormDescription>
      )}
      {Boolean(error) && (
        <FormMessage>
          {error}
        </FormMessage>
      )}
    </FormItem>
  )
}

export {
  FormControl,
  FormDescription,
  FormField,
  FormFieldWrapper,
  FormItem,
  FormLabel,
  FormMessage
}
