
import { cn } from '../../utils/cn'

export interface UnifiedSkeletonProps {
    variant?: 'text' | 'card' | 'avatar' | 'button'
    lines?: number
    className?: string
}

export function UnifiedSkeleton({
    variant = 'text',
    lines = 1,
    className
}: UnifiedSkeletonProps) {
    const variants = {
        text: 'h-4 bg-gray-200 rounded',
        card: 'h-32 bg-gray-200 rounded-lg',
        avatar: 'w-10 h-10 bg-gray-200 rounded-full',
        button: 'h-10 bg-gray-200 rounded-md'
    }

    if (variant === 'text' && lines > 1) {
        return (
            <div className={cn('space-y-2', className)}>
                {Array.from({ length: lines }).map((_, i) => (
                    <div
                        // eslint-disable-next-line react/no-array-index-key
                        key={`skeleton-line-${i}`}
                        className={cn(
                            variants.text,
                            'animate-pulse',
                            i === lines - 1 ? 'w-3/4' : 'w-full'
                        )}
                    />
                ))}
            </div>
        )
    }

    return (
        <div className={cn(variants[variant], 'animate-pulse', className)} />
    )
}
