
import { type ReactNode } from 'react'
import { cn } from '../../utils/cn'
import { UnifiedLoader, type LoaderVariant } from './unified-loader'

export interface UnifiedLoadingProps {
    type?: 'page' | 'section' | 'inline'
    variant?: LoaderVariant
    message?: string
    children?: ReactNode
    className?: string
}

export function UnifiedLoading({
    type = 'inline',
    variant = 'default',
    message = 'Loading...',
    children,
    className
}: UnifiedLoadingProps) {
    if (type === 'page') {
        return (
            <div className={cn(
                'min-h-screen flex items-center justify-center bg-gray-50',
                className
            )}>
                <div className="text-center">
                    <UnifiedLoader variant={variant} size="lg" type="spinner" />
                    <h2 className="mt-4 text-lg font-semibold text-gray-900">
                        Loading EPSX Analytics
                    </h2>
                    <p className="mt-2 text-gray-600">{message}</p>
                </div>
            </div>
        )
    }

    if (type === 'section') {
        return (
            <div className={cn(
                'flex items-center justify-center py-12',
                className
            )}>
                <div className="text-center">
                    <UnifiedLoader variant={variant} size="md" type="dots" />
                    <p className="mt-3 text-sm text-gray-600">{message}</p>
                </div>
            </div>
        )
    }

    return (
        <div className={cn('flex items-center justify-center py-4', className)}>
            <UnifiedLoader variant={variant} size="sm" message={message} />
            {children}
        </div>
    )
}
