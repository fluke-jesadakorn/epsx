
import React, { forwardRef } from 'react'
import { cn } from '../../utils/cn'
import {
  buildDescribedBy,
  containerClasses,
  disabledClasses,
  inputSizes,
  inputStates,
  inputVariants,
  type BaseCheckboxProps,
  type BaseInputProps,
  type BaseRadioProps,
  type BaseSelectProps,
  type BaseTextareaProps
} from './form-utils'

// ============================================================================
// BASE INPUT COMPONENT
// ============================================================================

const BaseInput = forwardRef<HTMLInputElement, BaseInputProps>(({
  className,
  containerClassName,
  label,
  helperText,
  error,
  startIcon,
  endIcon,
  fullWidth = false,
  variant = 'default',
  inputSize = 'md',
  state = 'default',
  disabled,
  id,
  'aria-describedby': ariaDescribedby,
  ...props
}, ref) => {
  const generatedId = React.useId()
  const inputId = id ?? generatedId
  const helperId = React.useId()
  const errorId = React.useId()
  const describedBy = buildDescribedBy({ ariaDescribedby, helperText, helperId, error, errorId })

  // Resolve state if error is present but state is default
  const finalState = error && state === 'default' ? 'error' : state

  return (
    <div className={cn(containerClasses, fullWidth && 'w-full', containerClassName)}>
      <LabelContent htmlFor={inputId} label={label} />

      <div className="relative">
        {/* Start Icon */}
        {startIcon && (
          <div className="absolute left-3 top-1/2 -translate-y-1/2 text-[hsl(var(--muted-foreground))] pointer-events-none">
            {startIcon}
          </div>
        )}

        <input
          ref={ref}
          id={inputId}
          disabled={disabled}
          aria-invalid={Boolean(error)}
          aria-describedby={describedBy}
          className={cn(
            'flex w-full rounded-md shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-[hsl(var(--muted-foreground))] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
            inputVariants[variant],
            inputSizes[inputSize],
            inputStates[finalState],
            startIcon && 'pl-10',
            endIcon && 'pr-10',
            disabledClasses,
            className
          )}
          {...props}
        />

        {/* End Icon */}
        {endIcon && (
          <div className="absolute right-3 top-1/2 -translate-y-1/2 text-[hsl(var(--muted-foreground))] pointer-events-none">
            {endIcon}
          </div>
        )}
      </div>

      <HelperContent
        helperText={helperText}
        error={error}
        helperId={helperId}
        errorId={errorId}
      />
    </div>
  )
})
BaseInput.displayName = 'base-input'

// ============================================================================
// BASE TEXTAREA COMPONENT
// ============================================================================

const BaseTextarea = forwardRef<HTMLTextAreaElement, BaseTextareaProps>(({
  className,
  containerClassName,
  label,
  helperText,
  error,
  fullWidth = false,
  variant = 'default',
  state = 'default',
  disabled,
  id,
  'aria-describedby': ariaDescribedby,
  ...props
}, ref) => {
  const generatedId = React.useId()
  const inputId = id ?? generatedId
  const helperId = React.useId()
  const errorId = React.useId()
  const describedBy = buildDescribedBy({ ariaDescribedby, helperText, helperId, error, errorId })

  const finalState = error && state === 'default' ? 'error' : state

  return (
    <div className={cn(containerClasses, fullWidth && 'w-full', containerClassName)}>
      <LabelContent htmlFor={inputId} label={label} />

      <textarea
        ref={ref}
        id={inputId}
        disabled={disabled}
        aria-invalid={Boolean(error)}
        aria-describedby={describedBy}
        className={cn(
          'flex min-h-[80px] w-full rounded-md px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-[hsl(var(--muted-foreground))] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
          inputVariants[variant],
          inputStates[finalState],
          disabledClasses,
          className
        )}
        {...props}
      />

      <HelperContent
        helperText={helperText}
        error={error}
        helperId={helperId}
        errorId={errorId}
      />
    </div>
  )
})
BaseTextarea.displayName = 'BaseTextarea'

// ============================================================================
// BASE SELECT COMPONENT
// ============================================================================

const BaseSelect = forwardRef<HTMLSelectElement, BaseSelectProps>(({
  className,
  containerClassName,
  label,
  helperText,
  error,
  options,
  fullWidth = false,
  variant = 'default',
  inputSize = 'md',
  state = 'default',
  disabled,
  id,
  'aria-describedby': ariaDescribedby,
  placeholder,
  ...props
}, ref) => {
  const generatedId = React.useId()
  const inputId = id ?? generatedId
  const helperId = React.useId()
  const errorId = React.useId()
  const describedBy = buildDescribedBy({ ariaDescribedby, helperText, helperId, error, errorId })

  const finalState = error && state === 'default' ? 'error' : state

  return (
    <div className={cn(containerClasses, fullWidth && 'w-full', containerClassName)}>
      <LabelContent htmlFor={inputId} label={label} />

      <div className="relative">
        <select
          ref={ref}
          id={inputId}
          disabled={disabled}
          aria-invalid={Boolean(error)}
          aria-describedby={describedBy}
          className={cn(
            'flex w-full appearance-none rounded-md shadow-sm transition-colors focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
            inputVariants[variant],
            inputSizes[inputSize],
            inputStates[finalState],
            disabledClasses,
            className
          )}
          {...props}
        >
          {placeholder && (
            <option value="" disabled selected>
              {placeholder}
            </option>
          )}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value} disabled={opt.disabled}>
              {opt.label}
            </option>
          ))}
        </select>

        {/* Custom Chevron */}
        <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-[hsl(var(--muted-foreground))]">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="m6 9 6 6 6-6" />
          </svg>
        </div>
      </div>

      <HelperContent
        helperText={helperText}
        error={error}
        helperId={helperId}
        errorId={errorId}
      />
    </div>
  )
})
BaseSelect.displayName = 'BaseSelect'

// ============================================================================
// BASE CHECKBOX COMPONENT
// ============================================================================

const BaseCheckbox = forwardRef<HTMLInputElement, BaseCheckboxProps>(({
  className,
  containerClassName,
  label,
  helperText,
  error,
  state = 'default',
  disabled,
  id,
  'aria-describedby': ariaDescribedby,
  ...props
}, ref) => {
  const generatedId = React.useId()
  const inputId = id ?? generatedId
  const helperId = React.useId()
  const errorId = React.useId()
  const describedBy = buildDescribedBy({ ariaDescribedby, helperText, helperId, error, errorId })

  const finalState = error && state === 'default' ? 'error' : state

  return (
    <div className={cn(containerClasses, containerClassName)}>
      <div className="flex items-start space-x-2">
        <input
          type="checkbox"
          ref={ref}
          id={inputId}
          disabled={disabled}
          aria-invalid={Boolean(error)}
          aria-describedby={describedBy}
          className={cn(
            'h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50',
            finalState === 'error' && 'border-red-300 text-red-600 focus:ring-red-500',
            disabledClasses,
            className
          )}
          {...props}
        />
        {label && (
          <label htmlFor={inputId} className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
            {label}
          </label>
        )}
      </div>

      <div className="ml-6">
        <HelperContent
          helperText={helperText}
          error={error}
          helperId={helperId}
          errorId={errorId}
        />
      </div>
    </div>
  )
})
BaseCheckbox.displayName = 'BaseCheckbox'

// ============================================================================
// BASE RADIO COMPONENT
// ============================================================================

const BaseRadio = forwardRef<HTMLInputElement, BaseRadioProps>(({
  className,
  containerClassName,
  label,
  helperText,
  error,
  state = 'default',
  disabled,
  id,
  'aria-describedby': ariaDescribedby,
  ...props
}, ref) => {
  const generatedId = React.useId()
  const inputId = id ?? generatedId
  const helperId = React.useId()
  const errorId = React.useId()
  const describedBy = buildDescribedBy({ ariaDescribedby, helperText, helperId, error, errorId })

  const finalState = error && state === 'default' ? 'error' : state

  return (
    <div className={cn(containerClasses, containerClassName)}>
      <div className="flex items-center space-x-2">
        <input
          type="radio"
          ref={ref}
          id={inputId}
          disabled={disabled}
          // Radio buttons don't technically support aria-invalid on individual definitions in standard HTML5 
          // usually, but in React we often pass it. However, lint rule complained.
          // aria-invalid={!!error} 
          aria-describedby={describedBy}
          className={cn(
            'aspect-square h-4 w-4 rounded-full border border-[hsl(var(--primary))] text-[hsl(var(--primary))] shadow focus:outline-none focus-visible:ring-1 focus-visible:ring-[hsl(var(--ring))] disabled:cursor-not-allowed disabled:opacity-50',
            finalState === 'error' && 'border-[hsl(var(--destructive))] text-[hsl(var(--destructive))]',
            disabledClasses,
            className
          )}
          {...props}
        />
        {label && (
          <label htmlFor={inputId} className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
            {label}
          </label>
        )}
      </div>

      <div className="ml-6">
        <HelperContent
          helperText={helperText}
          error={error}
          helperId={helperId}
          errorId={errorId}
        />
      </div>
    </div>
  )
})
BaseRadio.displayName = 'BaseRadio'

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function LabelContent({ htmlFor, label }: { htmlFor: string, label?: string }) {
  if (!label) { return null }
  return (
    <label htmlFor={htmlFor} className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
      {label}
    </label>
  )
}

function HelperContent({
  helperText,
  error,
  helperId,
  errorId
}: {
  helperText?: React.ReactNode,
  error?: React.ReactNode,
  helperId: string,
  errorId: string
}) {
  if (!error && !helperText) { return null }

  return (
    <>
      {error ? (
        <p id={errorId} className="text-sm font-medium text-[hsl(var(--destructive))]">
          {error}
        </p>
      ) : helperText ? (
        <p id={helperId} className="text-sm text-[hsl(var(--muted-foreground))]">
          {helperText}
        </p>
      ) : null}
    </>
  )
}

export { BaseCheckbox, BaseInput, BaseRadio, BaseSelect, BaseTextarea }
