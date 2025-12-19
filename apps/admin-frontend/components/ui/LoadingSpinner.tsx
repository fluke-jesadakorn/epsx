/**
 * LoadingSpinner - Inline loading indicator component
 * For use in buttons, small sections, and inline loading states
 * Provides consistent loading UX across the admin dashboard
 */

import { cn } from '@/lib/utils'
import { Loader2 } from 'lucide-react'

export interface LoadingSpinnerProps {
    /** Size of the spinner */
    size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
    /** Optional label text */
    label?: string
    /** Additional CSS classes */
    className?: string
    /** Show label inline or below */
    labelPosition?: 'inline' | 'below'
    /** Color variant */
    variant?: 'default' | 'primary' | 'muted' | 'white'
}

const sizeClasses = {
    xs: 'h-3 w-3',
    sm: 'h-4 w-4',
    md: 'h-5 w-5',
    lg: 'h-6 w-6',
    xl: 'h-8 w-8'
}

const labelSizeClasses = {
    xs: 'text-xs',
    sm: 'text-sm',
    md: 'text-sm',
    lg: 'text-base',
    xl: 'text-lg'
}

const variantClasses = {
    default: 'text-gray-600 dark:text-gray-400',
    primary: 'text-blue-600 dark:text-blue-400',
    muted: 'text-gray-400 dark:text-gray-500',
    white: 'text-white'
}

/**
 * Loading spinner component with optional label
 * @param props - LoadingSpinnerProps
 */
export function LoadingSpinner({
    size = 'md',
    label,
    className,
    labelPosition = 'inline',
    variant = 'default'
}: LoadingSpinnerProps) {
    const containerClasses = cn(
        'flex items-center gap-2',
        labelPosition === 'below' && 'flex-col gap-1',
        className
    )

    return (
        <div className={containerClasses} role="status" aria-live="polite">
            <Loader2
                className={cn(
                    'animate-spin',
                    sizeClasses[size],
                    variantClasses[variant]
                )}
            />
            {label && (
                <span className={cn(
                    labelSizeClasses[size],
                    variantClasses[variant]
                )}>
                    {label}
                </span>
            )}
            {!label && <span className="sr-only">Loading...</span>}
        </div>
    )
}

/**
 * Full-page centered loading spinner
 */
export function PageLoadingSpinner({
    label = 'Loading...',
    className
}: {
    label?: string
    className?: string
}) {
    return (
        <div className={cn(
            'min-h-[400px] flex flex-col items-center justify-center gap-4',
            className
        )}>
            <LoadingSpinner size="xl" variant="primary" />
            <p className="text-gray-600 dark:text-gray-400 text-sm">{label}</p>
        </div>
    )
}

/**
 * Button loading spinner - for use inside buttons
 */
export function ButtonLoadingSpinner({
    className
}: {
    className?: string
}) {
    return (
        <Loader2
            className={cn(
                'h-4 w-4 animate-spin',
                className
            )}
        />
    )
}

/**
 * Section loading state - centered spinner with optional label
 */
export function SectionLoading({
    label = 'Loading...',
    className
}: {
    label?: string
    className?: string
}) {
    return (
        <div className={cn(
            'flex flex-col items-center justify-center py-12 gap-3',
            className
        )}>
            <div className="relative">
                <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full blur opacity-20" />
                <LoadingSpinner size="lg" variant="primary" />
            </div>
            <p className="text-gray-500 dark:text-gray-400 text-sm">{label}</p>
        </div>
    )
}

/**
 * Inline loading indicator for lists, counts, etc.
 */
export function InlineLoading({ className }: { className?: string }) {
    return (
        <span className={cn('inline-flex items-center gap-1.5', className)}>
            <Loader2 className="h-3 w-3 animate-spin text-gray-400" />
            <span className="text-gray-400 text-xs">Loading...</span>
        </span>
    )
}
