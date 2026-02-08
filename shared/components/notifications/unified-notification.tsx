
'use client'

import React, { useEffect, useState } from 'react'
import { cn } from '../../utils/cn'
import { positions } from './notification-styles'
import { UnifiedAlert } from './unified-alert'
import {
    NotificationCloseButton,
    NotificationContent,
    NotificationIcon,
    NotificationProgressBar,
    useNotificationStyles
} from './unified-notification-components'
import type { IconType, NotificationType, NotificationVariant, Position } from './unified-notification-types'

export interface UnifiedNotificationProps {
    type?: NotificationType
    variant?: NotificationVariant
    iconType?: IconType
    title?: string
    message: string
    icon?: React.ReactNode
    duration?: number
    onClose?: () => void
    visible?: boolean
    position?: Position
    className?: string
}

export function UnifiedNotification({
    type = 'info',
    variant = 'default',
    iconType = 'lucide',
    title,
    message,
    icon,
    duration = 5000,
    onClose,
    visible = true,
    position = 'top-right',
    className
}: UnifiedNotificationProps) {
    const [isVisible, setIsVisible] = useState(visible)
    const [progress, setProgress] = useState(100)

    useEffect(() => {
        setIsVisible(visible)
        let interval: ReturnType<typeof setInterval> | undefined

        if (visible && duration > 0) {
            const startTime = Date.now()
            interval = setInterval(() => {
                const elapsed = Date.now() - startTime
                const remaining = Math.max(0, 100 - (elapsed / duration) * 100)
                setProgress(remaining)

                if (remaining === 0) {
                    clearInterval(interval)
                    setIsVisible(false)
                    onClose?.()
                }
            }, 16)
        }

        return () => {
            if (interval) { clearInterval(interval) }
        }
    }, [visible, duration, onClose])

    const { typeStyle, variantStyle, colorStyle } = useNotificationStyles(type, variant)

    if (!isVisible) { return null }

    return (
        <div
            className={cn(
                'fixed z-50 max-w-sm w-full transition-all duration-300 ease-out',
                positions[position],
                className
            )}
            style={{
                animation: 'slideIn 0.3s ease-out'
            }}
        >
            <div
                className={cn(
                    'overflow-hidden rounded-lg',
                    variantStyle.container,
                    variantStyle.useDark ? colorStyle.bg : `${colorStyle.bg} ${colorStyle.text}`
                )}
            >
                <NotificationProgressBar
                    duration={duration}
                    progress={progress}
                    typeStyle={typeStyle}
                />

                <div className="p-4">
                    <div className="flex items-start gap-3">
                        <NotificationIcon
                            icon={icon}
                            iconType={iconType}
                            variant={variant}
                            typeStyle={typeStyle}
                            variantStyle={variantStyle}
                        />

                        <NotificationContent
                            title={title}
                            message={message}
                            variantStyle={variantStyle}
                            colorStyle={colorStyle}
                        />

                        <NotificationCloseButton
                            variant={variant}
                            variantStyle={variantStyle}
                            onClose={() => {
                                setIsVisible(false)
                                onClose?.()
                            }}
                        />
                    </div>
                </div>

                {/* Metro accent strip for pancake/admin variants */}
                {(variant === 'pancake' || variant === 'admin') && (
                    <div className={cn('h-1 w-full', typeStyle.accent)} />
                )}
            </div>

            <style>{`
        @keyframes slideIn {
          from {
            transform: translateX(${position.includes('right') ? '100%' : position.includes('left') ? '-100%' : '0'}) 
                      translateY(${position.includes('top') ? '-100%' : position.includes('bottom') ? '100%' : '0'});
            opacity: 0;
          }
          to {
            transform: translateX(0) translateY(0);
            opacity: 1;
          }
        }
      `}</style>
        </div>
    )
}

export * from './unified-alert'
export * from './unified-notification-types'
export * from './use-unified-toast'

// Legacy component aliases for backward compatibility
export const MetroNotification = (props: UnifiedNotificationProps) =>
    <UnifiedNotification {...props} iconType="emoji" />
export const ProfessionalNotification = UnifiedNotification
export const ProfessionalAlert = UnifiedAlert
