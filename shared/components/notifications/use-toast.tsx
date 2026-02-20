
import { useState } from 'react'
import { UnifiedNotification, type UnifiedNotificationProps } from './notification'
import type { IconType, NotificationVariant, Position } from './notification-types'

// Toast Notification Manager
export interface ToastNotification extends Omit<UnifiedNotificationProps, 'visible' | 'onClose'> {
    id: string
}

export interface UseUnifiedToastProps {
    variant?: NotificationVariant
    iconType?: IconType
    position?: Position
    defaultDuration?: number
}

export function useUnifiedToast({
    variant = 'default',
    iconType = 'lucide',
    position = 'top-right',
    defaultDuration = 5000
}: UseUnifiedToastProps = {}) {
    const [toasts, setToasts] = useState<ToastNotification[]>([])

    const addToast = (notification: Omit<ToastNotification, 'id' | 'variant' | 'iconType'>) => {
        const id = Date.now().toString()
        const newToast = {
            ...notification,
            id,
            variant,
            iconType,
            duration: notification.duration ?? defaultDuration
        }

        setToasts(prev => [...prev, newToast])
        return id
    }

    const removeToast = (id: string) => {
        setToasts(prev => prev.filter(toast => toast.id !== id))
    }

    const success = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'success' })

    const error = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'error' })

    const warning = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'warning' })

    const info = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'info' })

    const clear = () => setToasts([])

    const ToastContainer = () => (
        <div className="fixed inset-0 pointer-events-none z-50">
            {toasts.map((toast, index) => {
                const offset = index * 80 + 16
                const isTopPosition = position.includes('top')
                const isRightPosition = position.includes('right')
                const isCenterPosition = position.includes('center')

                return (
                    <div
                        key={toast.id}
                        className="absolute pointer-events-auto"
                        style={{
                            [isTopPosition ? 'top' : 'bottom']: `${offset}px`,
                            [isCenterPosition ? 'left' : isRightPosition ? 'right' : 'left']:
                                isCenterPosition ? '50%' : '16px',
                            transform: isCenterPosition ? 'translateX(-50%)' : undefined,
                        }}
                    >
                        <UnifiedNotification
                            {...toast}
                            visible={true}
                            position="top-right" // Override since we handle positioning manually
                            onClose={() => removeToast(toast.id)}
                        />
                    </div>
                )
            })}
        </div>
    )

    return {
        success,
        error,
        warning,
        info,
        addToast,
        removeToast,
        clear,
        ToastContainer
    }
}

// Specialized notification hooks for backward compatibility
export const usePancakeToast = () => useUnifiedToast({ variant: 'pancake', iconType: 'emoji' })
export const useAdminToast = () => useUnifiedToast({ variant: 'admin', iconType: 'emoji' })
export const useAnalyticsToast = () => useUnifiedToast({ variant: 'analytics' })
export const useProfessionalToast = () => useUnifiedToast({ variant: 'premium' })
export const useMetroToast = usePancakeToast
