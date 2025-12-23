'use client';

import { AlertCircle, CheckCircle, Info, X, XCircle } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { cn } from '../../utils/cn';

export type NotificationType = 'info' | 'success' | 'warning' | 'error';
export type NotificationVariant = 'default' | 'pancake' | 'admin' | 'analytics' | 'premium';
export type IconType = 'lucide' | 'emoji';
export type Position = 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';

export interface UnifiedNotificationProps {
    type?: NotificationType;
    variant?: NotificationVariant;
    iconType?: IconType;
    title?: string;
    message: string;
    icon?: React.ReactNode;
    duration?: number;
    onClose?: () => void;
    visible?: boolean;
    position?: Position;
    className?: string;
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
    const [isVisible, setIsVisible] = useState(visible);
    const [progress, setProgress] = useState(100);

    useEffect(() => {
        setIsVisible(visible);
        if (visible && duration > 0) {
            const startTime = Date.now();
            const interval = setInterval(() => {
                const elapsed = Date.now() - startTime;
                const remaining = Math.max(0, 100 - (elapsed / duration) * 100);
                setProgress(remaining);

                if (remaining === 0) {
                    clearInterval(interval);
                    setIsVisible(false);
                    onClose?.();
                }
            }, 16);

            return () => clearInterval(interval);
        }
    }, [visible, duration, onClose]);

    const positions = {
        'top-right': 'top-4 right-4',
        'top-left': 'top-4 left-4',
        'bottom-right': 'bottom-4 right-4',
        'bottom-left': 'bottom-4 left-4',
        'top-center': 'top-4 left-1/2 -translate-x-1/2',
        'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2'
    };

    const typeStyles = {
        info: {
            lucideIcon: <Info className="w-5 h-5 text-blue-500" />,
            emojiIcon: 'ℹ',
            accent: 'bg-blue-500',
            light: { bg: 'bg-blue-50 border-blue-200', text: 'text-blue-900' },
            dark: { bg: 'bg-blue-600', text: 'text-white' }
        },
        success: {
            lucideIcon: <CheckCircle className="w-5 h-5 text-green-500" />,
            emojiIcon: '✓',
            accent: 'bg-green-500',
            light: { bg: 'bg-green-50 border-green-200', text: 'text-green-900' },
            dark: { bg: 'bg-blue-600', text: 'text-white' }
        },
        warning: {
            lucideIcon: <AlertCircle className="w-5 h-5 text-yellow-500" />,
            emojiIcon: '⚠',
            accent: 'bg-yellow-500',
            light: { bg: 'bg-yellow-50 border-yellow-200', text: 'text-yellow-900' },
            dark: { bg: 'bg-yellow-600', text: 'text-white' }
        },
        error: {
            lucideIcon: <XCircle className="w-5 h-5 text-red-500" />,
            emojiIcon: '✕',
            accent: 'bg-red-500',
            light: { bg: 'bg-red-50 border-red-200', text: 'text-red-900' },
            dark: { bg: 'bg-red-600', text: 'text-white' }
        }
    };

    const variantStyles = {
        default: {
            container: 'shadow-lg bg-white border',
            icon: 'bg-gray-100',
            text: 'text-gray-900',
            useDark: false
        },
        pancake: {
            container: 'shadow-2xl bg-white border-l-4 border-orange-500 backdrop-blur-xl',
            icon: 'text-white rounded-none',
            text: 'text-gray-800',
            useDark: true
        },
        admin: {
            container: 'shadow-2xl bg-slate-800 border-l-4 border-blue-500 backdrop-blur-xl',
            icon: 'text-white rounded-none',
            text: 'text-white',
            useDark: true
        },
        analytics: {
            container: 'shadow-lg backdrop-blur-sm bg-white/95 border',
            icon: 'bg-gray-100',
            text: 'text-gray-900',
            useDark: false
        },
        premium: {
            container: 'shadow-xl backdrop-blur-md bg-gradient-to-r from-white to-blue-50/50 border',
            icon: 'bg-blue-100',
            text: 'text-gray-900',
            useDark: false
        }
    };

    const typeStyle = typeStyles[type];
    const variantStyle = variantStyles[variant];
    const colorStyle = variantStyle.useDark ? typeStyle.dark : typeStyle.light;

    const getIcon = () => {
        if (icon) return icon;

        if (iconType === 'emoji') {
            return (
                <span className="text-lg font-bold">
                    {typeStyle.emojiIcon}
                </span>
            );
        }

        return typeStyle.lucideIcon;
    };

    const getIconContainer = () => {
        if (variant === 'pancake' || variant === 'admin') {
            return (
                <div className={cn(
                    'w-10 h-10 flex items-center justify-center mr-3 flex-shrink-0',
                    variantStyle.icon,
                    typeStyle.accent
                )}>
                    {getIcon()}
                </div>
            );
        }

        return (
            <div className="flex-shrink-0 mt-0.5">
                {getIcon()}
            </div>
        );
    };

    if (!isVisible) return null;

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
                {/* Progress Bar */}
                {duration > 0 && (
                    <div className="h-1 bg-gray-200">
                        <div
                            className={cn('h-full transition-all duration-75 ease-linear', typeStyle.accent)}
                            style={{ width: `${progress}%` }}
                        />
                    </div>
                )}

                <div className="p-4">
                    <div className="flex items-start gap-3">
                        {/* Icon */}
                        {getIconContainer()}

                        {/* Content */}
                        <div className="flex-1 min-w-0">
                            {title && (
                                <h4 className={cn(
                                    'font-semibold text-sm mb-1',
                                    variantStyle.useDark ? 'text-white' : colorStyle.text
                                )}>
                                    {title}
                                </h4>
                            )}
                            <p className={cn(
                                'text-sm',
                                variantStyle.useDark ? 'text-white opacity-90' : colorStyle.text,
                                !title && 'font-medium'
                            )}>
                                {message}
                            </p>
                        </div>

                        {/* Close Button */}
                        <button
                            onClick={() => {
                                setIsVisible(false);
                                onClose?.();
                            }}
                            className={cn(
                                'flex-shrink-0 p-1 rounded-md transition-colors hover:bg-black/10',
                                variantStyle.useDark ? 'text-white opacity-50 hover:opacity-100' : 'text-gray-500 hover:text-gray-700'
                            )}
                        >
                            {variant === 'pancake' || variant === 'admin' ? (
                                <span className="text-lg">×</span>
                            ) : (
                                <X className="w-4 h-4" />
                            )}
                        </button>
                    </div>
                </div>

                {/* Metro accent strip for pancake/admin variants */}
                {(variant === 'pancake' || variant === 'admin') && (
                    <div className={cn('h-1 w-full', typeStyle.accent)} />
                )}
            </div>

            <style jsx>{`
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
    );
}

// Toast Notification Manager
export interface ToastNotification extends Omit<UnifiedNotificationProps, 'visible' | 'onClose'> {
    id: string;
}

export interface UseUnifiedToastProps {
    variant?: NotificationVariant;
    iconType?: IconType;
    position?: Position;
    defaultDuration?: number;
}

export function useUnifiedToast({
    variant = 'default',
    iconType = 'lucide',
    position = 'top-right',
    defaultDuration = 5000
}: UseUnifiedToastProps = {}) {
    const [toasts, setToasts] = useState<ToastNotification[]>([]);

    const addToast = (notification: Omit<ToastNotification, 'id' | 'variant' | 'iconType'>) => {
        const id = Date.now().toString();
        const newToast = {
            ...notification,
            id,
            variant,
            iconType,
            duration: notification.duration ?? defaultDuration
        };

        setToasts(prev => [...prev, newToast]);
        return id;
    };

    const removeToast = (id: string) => {
        setToasts(prev => prev.filter(toast => toast.id !== id));
    };

    const success = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'success' });

    const error = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'error' });

    const warning = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'warning' });

    const info = (message: string, options?: Partial<ToastNotification>) =>
        addToast({ ...options, message, type: 'info' });

    const clear = () => setToasts([]);

    const ToastContainer = () => (
        <div className="fixed inset-0 pointer-events-none z-50">
            {toasts.map((toast, index) => {
                const offset = index * 80 + 16;
                const isTopPosition = position.includes('top');
                const isRightPosition = position.includes('right');
                const isCenterPosition = position.includes('center');

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
                );
            })}
        </div>
    );

    return {
        success,
        error,
        warning,
        info,
        addToast,
        removeToast,
        clear,
        ToastContainer
    };
}

// Alert Component (static, non-dismissible)
export interface UnifiedAlertProps {
    type?: NotificationType;
    variant?: NotificationVariant;
    iconType?: IconType;
    title?: string;
    children: React.ReactNode;
    icon?: React.ReactNode;
    className?: string;
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
    const typeStyles = {
        info: {
            lucideIcon: <Info className="w-5 h-5 text-blue-500" />,
            emojiIcon: 'ℹ',
            light: { bg: 'bg-blue-50 border-blue-200', text: 'text-blue-900' }
        },
        success: {
            lucideIcon: <CheckCircle className="w-5 h-5 text-green-500" />,
            emojiIcon: '✓',
            light: { bg: 'bg-green-50 border-green-200', text: 'text-green-900' }
        },
        warning: {
            lucideIcon: <AlertCircle className="w-5 h-5 text-yellow-500" />,
            emojiIcon: '⚠',
            light: { bg: 'bg-yellow-50 border-yellow-200', text: 'text-yellow-900' }
        },
        error: {
            lucideIcon: <XCircle className="w-5 h-5 text-red-500" />,
            emojiIcon: '✕',
            light: { bg: 'bg-red-50 border-red-200', text: 'text-red-900' }
        }
    };

    const variantStyles = {
        default: '',
        pancake: 'shadow-lg',
        admin: 'shadow-lg bg-slate-800 text-white',
        analytics: 'backdrop-blur-sm bg-white/95',
        premium: 'shadow-md bg-gradient-to-r from-white to-blue-50/50'
    };

    const typeStyle = typeStyles[type];
    const isAdmin = variant === 'admin';

    const getIcon = () => {
        if (icon) return icon;

        if (iconType === 'emoji') {
            return (
                <span className="text-lg font-bold">
                    {typeStyle.emojiIcon}
                </span>
            );
        }

        return typeStyle.lucideIcon;
    };

    return (
        <div
            className={cn(
                'border rounded-lg p-4',
                isAdmin ? 'border-slate-600 bg-slate-800 text-white' : typeStyle.light.bg,
                variantStyles[variant],
                className
            )}
        >
            <div className="flex gap-3">
                <div className="flex-shrink-0">
                    {getIcon()}
                </div>
                <div className="flex-1">
                    {title && (
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
    );
}

// Specialized notification hooks for backward compatibility
export const usePancakeToast = () => useUnifiedToast({ variant: 'pancake', iconType: 'emoji' });
export const useAdminToast = () => useUnifiedToast({ variant: 'admin', iconType: 'emoji' });
export const useAnalyticsToast = () => useUnifiedToast({ variant: 'analytics' });
export const useProfessionalToast = () => useUnifiedToast({ variant: 'premium' });
export const useMetroToast = usePancakeToast;

// Legacy component aliases for backward compatibility
export const MetroNotification = (props: UnifiedNotificationProps) =>
    <UnifiedNotification {...props} iconType="emoji" />;
export const ProfessionalNotification = UnifiedNotification;
export const ProfessionalAlert = UnifiedAlert;
