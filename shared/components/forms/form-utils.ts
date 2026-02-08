
import type React from 'react'

export const inputVariants = {
    default: [
        'border border-[hsl(var(--border))]',
        'bg-[hsl(var(--background))]',
        'focus:ring-2 focus:ring-[hsl(var(--ring))] focus:border-[hsl(var(--ring))]'
    ],
    filled: [
        'border-transparent',
        'bg-[hsl(var(--muted))]',
        'focus:bg-[hsl(var(--background))] focus:border-[hsl(var(--ring))] focus:ring-2 focus:ring-[hsl(var(--ring))]'
    ],
    ghost: [
        'border-transparent',
        'bg-transparent',
        'hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))]',
        'focus:ring-2 focus:ring-[hsl(var(--ring))]'
    ]
}

export const inputSizes = {
    sm: 'h-8 px-2 py-1 text-sm',
    md: 'h-10 px-3 py-2 text-sm',
    lg: 'h-12 px-4 py-3 text-base'
}

export const inputStates = {
    default: '',
    error: 'border-[hsl(var(--destructive))] focus:border-[hsl(var(--destructive))] focus:ring-[hsl(var(--destructive))]',
    success: 'border-[hsl(var(--success))] focus:border-[hsl(var(--success))] focus:ring-[hsl(var(--success))]',
    warning: 'border-[hsl(var(--warning))] focus:border-[hsl(var(--warning))] focus:ring-[hsl(var(--warning))]'
}

export const disabledClasses = 'disabled:cursor-not-allowed disabled:opacity-50'

export const containerClasses = 'flex flex-col gap-1.5'

export interface DescribedByOptions {
    ariaDescribedby?: string
    helperText?: React.ReactNode
    helperId: string
    error?: React.ReactNode
    errorId: string
}

export function buildDescribedBy({
    ariaDescribedby,
    helperText,
    helperId,
    error,
    errorId
}: DescribedByOptions) {
    return [
        ariaDescribedby,
        helperText ? helperId : null,
        error ? errorId : null
    ].filter(Boolean).join(' ') || undefined
}

export interface BaseInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    label?: string
    helperText?: React.ReactNode
    error?: React.ReactNode
    startIcon?: React.ReactNode
    endIcon?: React.ReactNode
    fullWidth?: boolean
    variant?: keyof typeof inputVariants
    inputSize?: keyof typeof inputSizes
    state?: keyof typeof inputStates
    containerClassName?: string
}

export interface BaseTextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
    label?: string
    helperText?: React.ReactNode
    error?: React.ReactNode
    fullWidth?: boolean
    variant?: keyof typeof inputVariants
    state?: keyof typeof inputStates
    containerClassName?: string
}

export interface BaseSelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
    label?: string
    helperText?: React.ReactNode
    error?: React.ReactNode
    fullWidth?: boolean
    variant?: keyof typeof inputVariants
    inputSize?: keyof typeof inputSizes
    state?: keyof typeof inputStates
    containerClassName?: string
    options: Array<{ label: string; value: string | number; disabled?: boolean }>
    placeholder?: string
}

export interface BaseCheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
    label?: string
    helperText?: React.ReactNode
    error?: React.ReactNode
    state?: keyof typeof inputStates
    containerClassName?: string
}

export interface BaseRadioProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
    label?: string
    helperText?: React.ReactNode
    error?: React.ReactNode
    state?: keyof typeof inputStates
    containerClassName?: string
}
