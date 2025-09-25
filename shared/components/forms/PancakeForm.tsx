/**
 * PANCAKESWAP THEMED FORM COMPONENTS
 * PancakeSwap-themed wrapper around BaseForm for admin-frontend
 * Replaces apps/admin-frontend/components/ui/form-components.tsx
 */

"use client"

import * as React from "react"
import { 
  BaseForm, 
  FormField as BaseFormField, 
  FormItem, 
  FormLabel, 
  FormControl, 
  FormDescription, 
  FormMessage,
  type BaseFormProps
} from './BaseForm'
import { 
  BaseInput,
  type BaseInputProps 
} from './BaseInput'
import { PancakeButton } from '../buttons/PancakeButton'
import { cn } from '../../utils'

// ============================================================================
// PANCAKESWAP FORM VARIANTS
// ============================================================================

/**
 * PancakeSwap Form wrapper
 */
export interface PancakeFormProps extends BaseFormProps {
  variant?: 'pancake' | 'standard'
}

export const Form = React.forwardRef<HTMLFormElement, PancakeFormProps>(({
  variant = 'pancake',
  className,
  ...props
}, ref) => {
  return (
    <BaseForm
      ref={ref}
      className={cn(
        // PancakeSwap form styling
        variant === 'pancake' && [
          'space-y-6 p-6',
          'bg-gradient-to-br from-orange-50/50 to-yellow-50/50',
          'dark:from-orange-950/30 dark:to-yellow-950/30',
          'border border-orange-200 dark:border-orange-800',
          'rounded-xl backdrop-blur-sm'
        ],
        className
      )}
      {...props}
    />
  )
})
Form.displayName = "PancakeForm"

/**
 * PancakeSwap Input with enhanced styling
 */
export interface PancakeInputProps extends BaseInputProps {
  error?: boolean
  helperText?: string
}

export const Input = React.forwardRef<HTMLInputElement, PancakeInputProps>(({
  variant = 'outlined',
  className,
  error,
  helperText,
  ...props
}, ref) => {
  return (
    <div className="space-y-1">
      <BaseInput
        ref={ref}
        variant={variant}
        error={error}
        className={cn(
          // PancakeSwap input styling
          'border-2 rounded-lg transition-all duration-200',
          'bg-white dark:bg-gray-900',
          'border-orange-200 dark:border-orange-700',
          'focus:border-orange-400 focus:ring-orange-400/20',
          'placeholder:text-orange-400/60',
          'text-gray-900 dark:text-gray-100',
          error && 'border-red-400 focus:border-red-400 focus:ring-red-400/20',
          className
        )}
        {...props}
      />
      {error && typeof error === 'string' && (
        <p className="text-sm text-red-600 dark:text-red-400" role="alert">
          {error}
        </p>
      )}
      {helperText && !error && (
        <p className="text-sm text-orange-600/80 dark:text-orange-400/80">
          {helperText}
        </p>
      )}
    </div>
  )
})
Input.displayName = "PancakeInput"

/**
 * PancakeSwap Label with gradient text
 */
export interface PancakeLabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {
  required?: boolean
}

export const Label = React.forwardRef<HTMLLabelElement, PancakeLabelProps>(({
  className,
  required,
  children,
  ...props
}, ref) => {
  return (
    <label
      ref={ref}
      className={cn(
        'block text-sm font-medium leading-none',
        'text-gray-900 dark:text-gray-100',
        'peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
        required && "after:content-['*'] after:ml-0.5 after:text-orange-500",
        className
      )}
      {...props}
    >
      {children}
    </label>
  )
})
Label.displayName = "PancakeLabel"

/**
 * Enhanced FormField for PancakeSwap theming
 */
export interface PancakeFormFieldProps {
  label: string
  id: string
  required?: boolean
  error?: string
  helperText?: string
  children: React.ReactNode
}

export const FormField: React.FC<PancakeFormFieldProps> = ({
  label,
  id,
  required,
  error,
  helperText,
  children
}) => {
  return (
    <FormItem className="space-y-2">
      <Label htmlFor={id} required={required}>
        {label}
      </Label>
      <FormControl>
        {React.cloneElement(children as React.ReactElement<any>, {
          id,
          'aria-invalid': error ? 'true' : 'false',
          'aria-describedby': error || helperText ? `${id}-helper` : undefined,
        })}
      </FormControl>
      {error && (
        <FormMessage>
          {error}
        </FormMessage>
      )}
      {helperText && !error && (
        <FormDescription>
          {helperText}
        </FormDescription>
      )}
    </FormItem>
  )
}

/**
 * Button alias - uses the main PancakeButton component
 */
export const Button = PancakeButton

/**
 * PancakeSwap Badge
 */
export interface PancakeBadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'destructive' | 'outline'
}

export const Badge: React.FC<PancakeBadgeProps> = ({
  className,
  variant = 'default',
  ...props
}) => {
  const variants = {
    default: 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white hover:from-orange-400 hover:to-yellow-400',
    secondary: 'bg-gradient-to-r from-blue-500 to-blue-600 text-white hover:from-blue-400 hover:to-blue-500',
    destructive: 'bg-gradient-to-r from-red-500 to-red-600 text-white hover:from-red-400 hover:to-red-500',
    outline: 'border-2 border-orange-400 text-orange-600 dark:text-orange-400 bg-transparent hover:bg-orange-50 dark:hover:bg-orange-950/20',
  }

  return (
    <div
      className={cn(
        'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold',
        'transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2',
        variants[variant],
        className
      )}
      {...props}
    />
  )
}

/**
 * PancakeSwap Select
 */
export interface PancakeSelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  error?: boolean
}

export const Select: React.FC<PancakeSelectProps> = ({ 
  className, 
  error, 
  children, 
  ...props 
}) => {
  return (
    <select
      className={cn(
        'flex h-10 w-full items-center justify-between rounded-lg border-2 px-3 py-2 text-sm',
        'bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100',
        'border-orange-200 dark:border-orange-700',
        'focus:outline-none focus:ring-2 focus:ring-orange-400/20 focus:border-orange-400',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'placeholder:text-orange-400/60',
        error && 'border-red-400 focus:border-red-400 focus:ring-red-400/20',
        className
      )}
      {...props}
    >
      {children}
    </select>
  )
}

/**
 * PancakeSwap Checkbox
 */
export const Checkbox: React.FC<React.InputHTMLAttributes<HTMLInputElement>> = ({ 
  className, 
  ...props 
}) => {
  return (
    <input
      type="checkbox"
      className={cn(
        'h-4 w-4 rounded border-2',
        'border-orange-300 dark:border-orange-600',
        'bg-white dark:bg-gray-900',
        'text-orange-600 focus:ring-orange-500',
        'focus:outline-none focus:ring-2 focus:ring-orange-400/20 focus:ring-offset-2',
        'disabled:cursor-not-allowed disabled:opacity-50',
        className
      )}
      {...props}
    />
  )
}

/**
 * PancakeSwap Textarea
 */
export interface PancakeTextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  error?: boolean
}

export const Textarea: React.FC<PancakeTextareaProps> = ({ 
  className, 
  error, 
  ...props 
}) => {
  return (
    <textarea
      className={cn(
        'flex min-h-[80px] w-full rounded-lg border-2 px-3 py-2 text-sm',
        'bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100',
        'border-orange-200 dark:border-orange-700',
        'focus:outline-none focus:ring-2 focus:ring-orange-400/20 focus:border-orange-400',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'placeholder:text-orange-400/60',
        error && 'border-red-400 focus:border-red-400 focus:ring-red-400/20',
        className
      )}
      {...props}
    />
  )
}

// ============================================================================
// EXPORTS
// ============================================================================

export {
  Form as PancakeForm,
  Input as PancakeInput,
  Label as PancakeLabel,
  Button as PancakeFormButton,
  Badge as PancakeBadge,
  Select as PancakeSelect,
  Checkbox as PancakeCheckbox,
  Textarea as PancakeTextarea,
  FormField as PancakeFormField
}

export type {
  PancakeFormProps,
  PancakeInputProps,
  PancakeLabelProps,
  PancakeBadgeProps,
  PancakeSelectProps,
  PancakeTextareaProps,
  PancakeFormFieldProps
}