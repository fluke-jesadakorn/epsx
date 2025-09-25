/**
 * BASE INPUT COMPONENTS
 * Unified input components consolidating all form input types
 * Replaces duplicate Input, Select, Textarea, Checkbox implementations
 */

import * as React from "react"
import { cn } from '../../utils'

// ============================================================================
// INPUT TYPES
// ============================================================================

export type InputVariant = 'default' | 'filled' | 'outlined' | 'ghost'
export type InputSize = 'sm' | 'md' | 'lg'
export type InputState = 'default' | 'error' | 'success' | 'warning'

// ============================================================================
// SHARED INPUT STYLES
// ============================================================================

const inputVariants = {
  default: [
    'border border-gray-300 dark:border-gray-600',
    'bg-white dark:bg-gray-800',
    'focus:ring-2 focus:ring-blue-500 focus:border-blue-500'
  ],
  filled: [
    'border border-transparent',
    'bg-gray-100 dark:bg-gray-700',
    'focus:ring-2 focus:ring-blue-500 focus:bg-white dark:focus:bg-gray-800'
  ],
  outlined: [
    'border-2 border-gray-300 dark:border-gray-600',
    'bg-transparent',
    'focus:ring-0 focus:border-blue-500'
  ],
  ghost: [
    'border border-transparent',
    'bg-transparent',
    'focus:ring-1 focus:ring-gray-300 focus:bg-gray-50 dark:focus:bg-gray-800'
  ]
}

const inputSizes = {
  sm: 'h-8 px-2 py-1 text-sm',
  md: 'h-10 px-3 py-2 text-sm',
  lg: 'h-12 px-4 py-3 text-base'
}

const inputStates = {
  default: '',
  error: 'border-red-500 focus:border-red-500 focus:ring-red-500',
  success: 'border-green-500 focus:border-green-500 focus:ring-green-500',
  warning: 'border-yellow-500 focus:border-yellow-500 focus:ring-yellow-500'
}

// ============================================================================
// BASE INPUT COMPONENT
// ============================================================================

export interface BaseInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  variant?: InputVariant
  inputSize?: InputSize
  state?: InputState
  error?: boolean
  leftIcon?: React.ReactNode
  rightIcon?: React.ReactNode
  helperText?: string
  fullWidth?: boolean
}

export const BaseInput = React.forwardRef<HTMLInputElement, BaseInputProps>(
  ({
    className,
    type = 'text',
    variant = 'default',
    inputSize = 'md',
    state = 'default',
    error,
    leftIcon,
    rightIcon,
    helperText,
    fullWidth = true,
    disabled,
    'aria-describedby': ariaDescribedby,
    ...props
  }, ref) => {
    const helperId = React.useId()
    const errorId = React.useId()
    
    // Determine state based on error prop
    const effectiveState = error ? 'error' : state
    
    // Build describedBy string
    const describedBy = [
      ariaDescribedby,
      helperText ? helperId : null,
      error ? errorId : null
    ].filter(Boolean).join(' ') || undefined

    const baseClasses = [
      // Base styling
      'relative flex items-center',
      'rounded-md',
      'transition-all duration-200',
      'focus-within:outline-none',
      'placeholder:text-gray-400 dark:placeholder:text-gray-500',
      
      // Variant styling
      ...inputVariants[variant],
      
      // Size styling
      inputSizes[inputSize],
      
      // State styling
      inputStates[effectiveState],
      
      // Width
      fullWidth && 'w-full',
      
      // Disabled state
      disabled && [
        'opacity-50',
        'cursor-not-allowed',
        'bg-gray-100 dark:bg-gray-800'
      ]
    ].filter(Boolean).flat()

    return (
      <div className="space-y-1">
        <div className={cn(baseClasses, className)}>
          {leftIcon && (
            <div className="flex-shrink-0 mr-2 text-gray-400">
              {leftIcon}
            </div>
          )}
          <input
            ref={ref}
            type={type}
            className={cn(
              'flex-1 bg-transparent border-0',
              'focus:outline-none focus:ring-0',
              'placeholder:text-inherit',
              leftIcon && 'pl-0',
              rightIcon && 'pr-0'
            )}
            disabled={disabled}
            aria-invalid={effectiveState === 'error'}
            aria-describedby={describedBy}
            {...props}
          />
          {rightIcon && (
            <div className="flex-shrink-0 ml-2 text-gray-400">
              {rightIcon}
            </div>
          )}
        </div>
        
        {helperText && !error && (
          <p id={helperId} className="text-sm text-gray-500 dark:text-gray-400">
            {helperText}
          </p>
        )}
        
        {error && typeof error === 'string' && (
          <p id={errorId} className="text-sm text-red-600 dark:text-red-400" role="alert">
            {error}
          </p>
        )}
      </div>
    )
  }
)
BaseInput.displayName = "BaseInput"

// ============================================================================
// TEXTAREA COMPONENT
// ============================================================================

export interface BaseTextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  variant?: InputVariant
  inputSize?: InputSize
  state?: InputState
  error?: boolean | string
  helperText?: string
  fullWidth?: boolean
  resize?: 'none' | 'vertical' | 'horizontal' | 'both'
}

export const BaseTextarea = React.forwardRef<HTMLTextAreaElement, BaseTextareaProps>(
  ({
    className,
    variant = 'default',
    inputSize = 'md',
    state = 'default',
    error,
    helperText,
    fullWidth = true,
    resize = 'vertical',
    disabled,
    'aria-describedby': ariaDescribedby,
    ...props
  }, ref) => {
    const helperId = React.useId()
    const errorId = React.useId()
    
    // Determine state based on error prop
    const effectiveState = error ? 'error' : state
    
    // Build describedBy string
    const describedBy = [
      ariaDescribedby,
      helperText ? helperId : null,
      error ? errorId : null
    ].filter(Boolean).join(' ') || undefined

    const baseClasses = [
      // Base styling
      'rounded-md',
      'transition-all duration-200',
      'focus:outline-none',
      'placeholder:text-gray-400 dark:placeholder:text-gray-500',
      'min-h-[80px]',
      
      // Variant styling
      ...inputVariants[variant],
      
      // Size styling
      inputSizes[inputSize],
      
      // State styling
      inputStates[effectiveState],
      
      // Width
      fullWidth && 'w-full',
      
      // Resize
      `resize-${resize}`,
      
      // Disabled state
      disabled && [
        'opacity-50',
        'cursor-not-allowed',
        'bg-gray-100 dark:bg-gray-800'
      ]
    ].filter(Boolean).flat()

    return (
      <div className="space-y-1">
        <textarea
          ref={ref}
          className={cn(baseClasses, className)}
          disabled={disabled}
          aria-invalid={effectiveState === 'error'}
          aria-describedby={describedBy}
          {...props}
        />
        
        {helperText && !error && (
          <p id={helperId} className="text-sm text-gray-500 dark:text-gray-400">
            {helperText}
          </p>
        )}
        
        {error && typeof error === 'string' && (
          <p id={errorId} className="text-sm text-red-600 dark:text-red-400" role="alert">
            {error}
          </p>
        )}
      </div>
    )
  }
)
BaseTextarea.displayName = "BaseTextarea"

// ============================================================================
// SELECT COMPONENT
// ============================================================================

export interface BaseSelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  variant?: InputVariant
  inputSize?: InputSize
  state?: InputState
  error?: boolean | string
  helperText?: string
  fullWidth?: boolean
  placeholder?: string
  children: React.ReactNode
}

export const BaseSelect = React.forwardRef<HTMLSelectElement, BaseSelectProps>(
  ({
    className,
    variant = 'default',
    inputSize = 'md',
    state = 'default',
    error,
    helperText,
    fullWidth = true,
    placeholder,
    children,
    disabled,
    'aria-describedby': ariaDescribedby,
    ...props
  }, ref) => {
    const helperId = React.useId()
    const errorId = React.useId()
    
    // Determine state based on error prop
    const effectiveState = error ? 'error' : state
    
    // Build describedBy string
    const describedBy = [
      ariaDescribedby,
      helperText ? helperId : null,
      error ? errorId : null
    ].filter(Boolean).join(' ') || undefined

    const baseClasses = [
      // Base styling
      'rounded-md',
      'transition-all duration-200',
      'focus:outline-none',
      'appearance-none',
      'bg-no-repeat bg-right',
      'bg-[url("data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%236b7280\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'m6 8 4 4 4-4\'/%3e%3c/svg%3e")]',
      'pr-8',
      
      // Variant styling
      ...inputVariants[variant],
      
      // Size styling
      inputSizes[inputSize],
      
      // State styling
      inputStates[effectiveState],
      
      // Width
      fullWidth && 'w-full',
      
      // Disabled state
      disabled && [
        'opacity-50',
        'cursor-not-allowed',
        'bg-gray-100 dark:bg-gray-800'
      ]
    ].filter(Boolean).flat()

    return (
      <div className="space-y-1">
        <select
          ref={ref}
          className={cn(baseClasses, className)}
          disabled={disabled}
          aria-invalid={effectiveState === 'error'}
          aria-describedby={describedBy}
          {...props}
        >
          {placeholder && (
            <option value="" disabled hidden>
              {placeholder}
            </option>
          )}
          {children}
        </select>
        
        {helperText && !error && (
          <p id={helperId} className="text-sm text-gray-500 dark:text-gray-400">
            {helperText}
          </p>
        )}
        
        {error && typeof error === 'string' && (
          <p id={errorId} className="text-sm text-red-600 dark:text-red-400" role="alert">
            {error}
          </p>
        )}
      </div>
    )
  }
)
BaseSelect.displayName = "BaseSelect"

// ============================================================================
// CHECKBOX COMPONENT
// ============================================================================

export interface BaseCheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  label?: string
  description?: string
  inputSize?: 'sm' | 'md' | 'lg'
  error?: boolean | string
  indeterminate?: boolean
}

export const BaseCheckbox = React.forwardRef<HTMLInputElement, BaseCheckboxProps>(
  ({
    className,
    label,
    description,
    inputSize = 'md',
    error,
    indeterminate,
    disabled,
    id,
    'aria-describedby': ariaDescribedby,
    ...props
  }, ref) => {
    const helperId = React.useId()
    const errorId = React.useId()
    const defaultId = React.useId()
    const checkboxId = id || defaultId
    
    // Build describedBy string
    const describedBy = [
      ariaDescribedby,
      description ? helperId : null,
      error ? errorId : null
    ].filter(Boolean).join(' ') || undefined

    const sizeClasses = {
      sm: 'h-3 w-3',
      md: 'h-4 w-4',
      lg: 'h-5 w-5'
    }

    // Set indeterminate state
    React.useEffect(() => {
      if (ref && typeof ref === 'object' && ref.current) {
        ref.current.indeterminate = !!indeterminate
      }
    }, [indeterminate, ref])

    return (
      <div className="space-y-1">
        <div className="flex items-start space-x-3">
          <input
            ref={ref}
            type="checkbox"
            id={checkboxId}
            className={cn(
              'rounded border-2 border-gray-300 dark:border-gray-600',
              'bg-white dark:bg-gray-800',
              'text-blue-600 focus:ring-blue-500',
              'focus:ring-2 focus:ring-offset-0',
              'transition-colors duration-200',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              error && 'border-red-500 focus:ring-red-500',
              sizeClasses[inputSize],
              className
            )}
            disabled={disabled}
            aria-invalid={!!error}
            aria-describedby={describedBy}
            {...props}
          />
          
          {(label || description) && (
            <div className="flex-1">
              {label && (
                <label
                  htmlFor={checkboxId}
                  className={cn(
                    'text-sm font-medium text-gray-900 dark:text-gray-100',
                    'cursor-pointer',
                    disabled && 'opacity-50 cursor-not-allowed'
                  )}
                >
                  {label}
                </label>
              )}
              {description && (
                <p id={helperId} className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                  {description}
                </p>
              )}
            </div>
          )}
        </div>
        
        {error && typeof error === 'string' && (
          <p id={errorId} className="text-sm text-red-600 dark:text-red-400" role="alert">
            {error}
          </p>
        )}
      </div>
    )
  }
)
BaseCheckbox.displayName = "BaseCheckbox"

// ============================================================================
// RADIO COMPONENT
// ============================================================================

export interface BaseRadioProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  label?: string
  description?: string
  inputSize?: 'sm' | 'md' | 'lg'
  error?: boolean | string
}

export const BaseRadio = React.forwardRef<HTMLInputElement, BaseRadioProps>(
  ({
    className,
    label,
    description,
    inputSize = 'md',
    error,
    disabled,
    id,
    'aria-describedby': ariaDescribedby,
    ...props
  }, ref) => {
    const helperId = React.useId()
    const errorId = React.useId()
    const defaultId = React.useId()
    const radioId = id || defaultId
    
    // Build describedBy string
    const describedBy = [
      ariaDescribedby,
      description ? helperId : null,
      error ? errorId : null
    ].filter(Boolean).join(' ') || undefined

    const sizeClasses = {
      sm: 'h-3 w-3',
      md: 'h-4 w-4',
      lg: 'h-5 w-5'
    }

    return (
      <div className="space-y-1">
        <div className="flex items-start space-x-3">
          <input
            ref={ref}
            type="radio"
            id={radioId}
            className={cn(
              'border-2 border-gray-300 dark:border-gray-600',
              'bg-white dark:bg-gray-800',
              'text-blue-600 focus:ring-blue-500',
              'focus:ring-2 focus:ring-offset-0',
              'transition-colors duration-200',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              error && 'border-red-500 focus:ring-red-500',
              sizeClasses[inputSize],
              className
            )}
            disabled={disabled}
            aria-invalid={!!error}
            aria-describedby={describedBy}
            {...props}
          />
          
          {(label || description) && (
            <div className="flex-1">
              {label && (
                <label
                  htmlFor={radioId}
                  className={cn(
                    'text-sm font-medium text-gray-900 dark:text-gray-100',
                    'cursor-pointer',
                    disabled && 'opacity-50 cursor-not-allowed'
                  )}
                >
                  {label}
                </label>
              )}
              {description && (
                <p id={helperId} className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                  {description}
                </p>
              )}
            </div>
          )}
        </div>
        
        {error && typeof error === 'string' && (
          <p id={errorId} className="text-sm text-red-600 dark:text-red-400" role="alert">
            {error}
          </p>
        )}
      </div>
    )
  }
)
BaseRadio.displayName = "BaseRadio"

// ============================================================================
// EXPORTS
// ============================================================================

export {
  BaseInput as Input,
  BaseTextarea as Textarea,
  BaseSelect as Select,
  BaseCheckbox as Checkbox,
  BaseRadio as Radio
}

export default BaseInput