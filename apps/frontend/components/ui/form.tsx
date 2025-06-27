"use client"

import * as React from "react"
import {
  Controller,
  FormProvider,
  useFormContext,
} from "react-hook-form"

import { cn } from "@/lib/utils"

import type {
  ControllerProps,
  FieldPath,
  FieldValues,
  UseFormReturn,
} from "react-hook-form"


interface FormItemContextValue {
  id: string
}

interface FormFieldContextValue<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
  name: TName
}

const FormItemContext = React.createContext<FormItemContextValue>({
  id: "",
})

const FormFieldContext = React.createContext<FormFieldContextValue>({} as FormFieldContextValue)

const Form = <TFieldValues extends FieldValues>({
  children,
  onSubmit,
  ...props
}: UseFormReturn<TFieldValues> & {
  onSubmit?: (data: TFieldValues) => void
  children: React.ReactNode
}) => (
  <FormProvider {...props}>
    <form
      onSubmit={onSubmit ? props.handleSubmit(onSubmit) : (e) => e.preventDefault()}
      className="space-y-4"
    >
      {children}
    </form>
  </FormProvider>
)

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

const useFormField = () => {
  const fieldContext = React.useContext(FormFieldContext)
  const itemContext = React.useContext(FormItemContext)
  const { getFieldState, formState } = useFormContext()

  if (!fieldContext) {
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

function FormItem({ className, ...props }: React.ComponentProps<"div">) {
  const id = React.useId()

  return (
    <FormItemContext.Provider value={{ id }}>
      <div
        data-slot="form-item"
        className={cn("grid gap-2", className)}
        {...props}
      />
    </FormItemContext.Provider>
  )
}

function FormLabel({
  className,
  ...props
}: React.LabelHTMLAttributes<HTMLLabelElement>) {
  const { error, formItemId } = useFormField()

  return (
    <label
      {...props}
      data-slot="form-label"
      data-error={!!error}
      className={cn(
        "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 data-[error=true]:text-destructive-foreground",
        className
      )}
      htmlFor={formItemId}
    />
  )
}

interface FormControlProps extends React.HTMLAttributes<HTMLDivElement> {
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
      "aria-invalid": !!error,
      className: cn("relative", className)
    }

    if (asChild) {
      return React.cloneElement(children as React.ReactElement, sharedProps)
    }

    return <div {...sharedProps}>{children}</div>
  }
)
FormControl.displayName = "FormControl"

function FormDescription({ className, ...props }: React.ComponentProps<"p">) {
  const { formDescriptionId } = useFormField()

  return (
    <p
      data-slot="form-description"
      id={formDescriptionId}
      className={cn("text-muted-foreground text-sm", className)}
      {...props}
    />
  )
}

function FormMessage({ className, ...props }: React.ComponentProps<"p">) {
  const { error, formMessageId } = useFormField()
  const body = error ? String(error?.message) : props.children

  if (!body) {
    return null
  }

  return (
    <p
      data-slot="form-message"
      id={formMessageId}
      className={cn("text-destructive-foreground text-sm", className)}
      {...props}
    >
      {body}
    </p>
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
