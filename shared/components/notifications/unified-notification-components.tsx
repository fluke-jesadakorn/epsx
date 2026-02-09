import { X } from 'lucide-react'
import React from 'react'
import { cn } from '../../utils/cn'
import { typeStyles, variantStyles, type TypeStyle, type VariantStyle } from './notification-styles'
import type { IconType, NotificationType, NotificationVariant } from './unified-notification-types'

// Helper to get type/variant styles
export function useNotificationStyles(type: NotificationType, variant: NotificationVariant) {
    const typeStyle = typeStyles[type]
    const variantStyle = variantStyles[variant]
    const colorStyle = variantStyle.useDark === true ? typeStyle.dark ?? typeStyle.light : typeStyle.light
    return { typeStyle, variantStyle, colorStyle }
}

export function NotificationIcon({
    icon,
    iconType,
    variant,
    typeStyle,
    variantStyle
}: {
    icon?: React.ReactNode
    iconType: IconType
    variant: NotificationVariant
    typeStyle: TypeStyle
    variantStyle: VariantStyle
}) {
    const getIcon = () => {
        if (icon !== undefined && icon !== null) { return icon }
        if (iconType === 'emoji') {
            return (
                <span className="text-lg font-bold">
                    {typeStyle.emojiIcon}
                </span>
            )
        }
        return typeStyle.lucideIcon
    }

    if (variant === 'pancake' || variant === 'admin') {
        return (
            <div className={cn(
                'w-10 h-10 flex items-center justify-center mr-3 flex-shrink-0',
                variantStyle.icon,
                typeStyle.accent
            )}>
                {getIcon()}
            </div>
        )
    }

    return (
        <div className="flex-shrink-0 mt-0.5">
            {getIcon()}
        </div>
    )
}

export function NotificationProgressBar({
    duration,
    progress,
    typeStyle
}: {
    duration: number
    progress: number
    typeStyle: TypeStyle
}) {
    if (duration <= 0) { return null }
    return (
        <div className="h-1 bg-gray-200">
            <div
                className={cn('h-full transition-all duration-75 ease-linear', typeStyle.accent)}
                style={{ width: `${progress}%` }}
            />
        </div>
    )
}

export function NotificationCloseButton({
    variant,
    variantStyle,
    onClose
}: {
    variant: NotificationVariant
    variantStyle: VariantStyle
    onClose: () => void
}) {
    return (
        <button
            onClick={onClose}
            className={cn(
                'flex-shrink-0 p-1 rounded-md transition-colors hover:bg-black/10',
                variantStyle.useDark === true ? 'text-white opacity-50 hover:opacity-100' : 'text-gray-500 hover:text-gray-700'
            )}
        >
            {variant === 'pancake' || variant === 'admin' ? (
                <span className="text-lg">×</span>
            ) : (
                <X className="w-4 h-4" />
            )}
        </button>
    )
}

export function NotificationContent({
    title,
    message,
    variantStyle,
    colorStyle
}: {
    title?: string
    message: string
    variantStyle: VariantStyle
    colorStyle: { bg: string; text: string }
}) {
    return (
        <div className="flex-1 min-w-0">
            {title !== undefined && title !== '' && (
                <h4 className={cn(
                    'font-semibold text-sm mb-1',
                    variantStyle.useDark === true ? 'text-white' : colorStyle.text
                )}>
                    {title}
                </h4>
            )}
            <p className={cn(
                'text-sm',
                variantStyle.useDark === true ? 'text-white opacity-90' : colorStyle.text,
                (title === undefined || title === '') && 'font-medium'
            )}>
                {message}
            </p>
        </div>
    )
}
