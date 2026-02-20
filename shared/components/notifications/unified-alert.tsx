
import React from 'react'
import { cn } from '../../utils/cn'
import { typeStyles, variantStyles } from './notification-styles'
import type { IconType, NotificationType, NotificationVariant } from './unified-notification-types'

export interface UnifiedAlertProps {
    type?: NotificationType
    variant?: NotificationVariant
    iconType?: IconType
    title?: string
    children: React.ReactNode
    icon?: React.ReactNode
    className?: string
}

export function UnifiedAlert({
    type = 'info',
    variant = 'default',
    iconType = 'lucide',
    title,
    children,
    icon,
    className
}: UnifiedAlertProps) {
    const typeStyle = typeStyles[type]
    const isAdmin = variant === 'admin'

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

    return (
        <div
            className={cn(
                'border rounded-lg p-4',
                isAdmin ? 'border-slate-600 bg-gray-100 dark:bg-slate-800 text-white' : typeStyle.light.bg,
                variantStyles[variant].alertClass,
                className
            )}
        >
            <div className="flex gap-3">
                <div className="flex-shrink-0">
                    {getIcon()}
                </div>
                <div className="flex-1">
                    {title !== undefined && title !== '' && (
                        <h4 className={cn(
                            'font-semibold text-sm mb-1',
                            isAdmin ? 'text-white' : typeStyle.light.text
                        )}>
                            {title}
                        </h4>
                    )}
                    <div className={cn(
                        'text-sm',
                        isAdmin ? 'text-gray-200' : typeStyle.light.text
                    )}>
                        {children}
                    </div>
                </div>
            </div>
        </div>
    )
}
