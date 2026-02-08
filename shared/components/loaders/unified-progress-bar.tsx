
import { cn } from '../../utils/cn'

export type LoaderVariant = 'default' | 'pancake' | 'admin' | 'analytics' | 'premium' | 'white'

export interface UnifiedProgressBarProps {
    progress?: number
    variant?: LoaderVariant
    animated?: boolean
    showPercentage?: boolean
    className?: string
}

export function UnifiedProgressBar({
    progress = 0,
    variant = 'default',
    animated = true,
    showPercentage = false,
    className
}: UnifiedProgressBarProps) {
    const variants = {
        default: 'bg-gradient-to-r from-blue-500 to-blue-600',
        pancake: 'bg-gradient-to-r from-orange-400 to-yellow-500',
        admin: 'bg-gradient-to-r from-blue-600 to-indigo-700',
        analytics: 'bg-gradient-to-r from-indigo-500 to-purple-600',
        premium: 'bg-gradient-to-r from-purple-500 to-pink-600',
        white: 'bg-white'
    }

    const textVariants = {
        default: 'text-blue-600',
        pancake: 'text-orange-600',
        admin: 'text-blue-600',
        analytics: 'text-indigo-600',
        premium: 'text-purple-600',
        white: 'text-white'
    }

    return (
        <div className={cn('w-full', className)}>
            <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden relative">
                <div
                    className={cn(
                        'h-full rounded-full',
                        variants[variant]
                    )}
                    style={{
                        width: `${Math.min(Math.max(progress, 0), 100)}%`,
                        transition: animated ? 'width 0.5s ease-out' : 'none'
                    }}
                />
            </div>

            {showPercentage && (
                <div className={cn('text-sm font-medium mt-2', textVariants[variant])}>
                    {Math.round(progress)}%
                </div>
            )}
        </div>
    )
}
