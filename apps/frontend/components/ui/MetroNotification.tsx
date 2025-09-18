'use client';

import { useEffect, useState } from 'react';
import { cn } from '@/lib/utils';

interface MetroNotificationProps {
  type?: 'info' | 'success' | 'warning' | 'error';
  variant?: 'pancake' | 'admin' | 'analytics';
  title?: string;
  message: string;
  icon?: string;
  duration?: number;
  onClose?: () => void;
  visible?: boolean;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
}

export function MetroNotification({
  type = 'info',
  variant = 'pancake',
  title,
  message,
  icon,
  duration = 5000,
  onClose,
  visible = true,
  position = 'top-right'
}: MetroNotificationProps) {
  const [isVisible, setIsVisible] = useState(visible);

  useEffect(() => {
    setIsVisible(visible);
    if (visible && duration > 0) {
      const timer = setTimeout(() => {
        setIsVisible(false);
        onClose?.();
      }, duration);
      return () => clearTimeout(timer);
    }
  }, [visible, duration, onClose]);

  const positions = {
    'top-right': 'top-4 right-4',
    'top-left': 'top-4 left-4',
    'bottom-right': 'bottom-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'top-center': 'top-4 left-1/2 transform -translate-x-1/2',
    'bottom-center': 'bottom-4 left-1/2 transform -translate-x-1/2'
  };

  const typeStyles = {
    info: {
      bg: 'bg-blue-500',
      accent: 'bg-blue-600',
      icon: 'ℹ'
    },
    success: {
      bg: 'bg-green-500',
      accent: 'bg-green-600',
      icon: '✓'
    },
    warning: {
      bg: 'bg-yellow-500',
      accent: 'bg-yellow-600',
      icon: '⚠'
    },
    error: {
      bg: 'bg-red-500',
      accent: 'bg-red-600',
      icon: '✕'
    }
  };

  const variantStyles = {
    pancake: {
      bg: 'bg-white',
      text: 'text-gray-800',
      border: 'border-orange-500'
    },
    admin: {
      bg: 'bg-slate-800',
      text: 'text-white',
      border: 'border-blue-500'
    },
    analytics: {
      bg: 'bg-gray-900',
      text: 'text-gray-100',
      border: 'border-slate-500'
    }
  };

  const typeStyle = typeStyles[type];
  const variantStyle = variantStyles[variant];

  if (!isVisible) return null;

  return (
    <div className={cn('fixed z-50 max-w-sm w-full', positions[position])}>
      <div
        className={cn(
          'shadow-2xl overflow-hidden border-l-4 backdrop-blur-xl',
          variantStyle.bg,
          variantStyle.text,
          variantStyle.border
        )}
      >

        {/* Progress Bar */}
        {duration > 0 && (
          <div className="h-1 bg-gray-200">
            <div
              className={cn('h-1 transition-all duration-75 ease-linear', typeStyle.bg)}
              style={{ width: `${(duration - (Date.now() - Date.now())) / duration * 100}%` }}
            />
          </div>
        )}

            <div className="p-4">
              <div className="flex items-start">
                {/* Icon Section */}
                <div className={cn('text-white rounded-none w-10 h-10 flex items-center justify-center mr-3 flex-shrink-0', typeStyle.bg)}>
                  <span className="text-lg font-bold">
                    {icon || typeStyle.icon}
                  </span>
                </div>

                {/* Content */}
                <div className="flex-1 min-w-0">
                  {title && (
                    <h4 className="font-bold text-sm mb-1">
                      {title}
                    </h4>
                  )}
                  <p className="text-sm opacity-90">
                    {message}
                  </p>
                </div>

                {/* Close Button */}
                <button
                  onClick={() => {
                    setIsVisible(false);
                    onClose?.();
                  }}
                  className="ml-3 opacity-50 hover:opacity-100 transition-opacity text-lg"
                >
                  ×
                </button>
              </div>
            </div>

        {/* Metro accent strip */}
        <div className={cn('h-1 w-full', typeStyle.accent)} />
      </div>
    </div>
  );
}

// Toast Notification Manager
interface ToastNotification extends Omit<MetroNotificationProps, 'visible' | 'onClose'> {
  id: string;
}

interface UseMetroToastProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  position?: MetroNotificationProps['position'];
  defaultDuration?: number;
}

export function useMetroToast({ 
  variant = 'pancake', 
  position = 'top-right',
  defaultDuration = 5000 
}: UseMetroToastProps = {}) {
  const [toasts, setToasts] = useState<ToastNotification[]>([]);

  const addToast = (notification: Omit<ToastNotification, 'id' | 'variant'>) => {
    const id = Date.now().toString();
    const newToast = {
      ...notification,
      id,
      variant,
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
            <MetroNotification
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
    ToastContainer
  };
}

// Pancake-themed shortcuts
export const usePancakeToast = () => useMetroToast({ variant: 'pancake' });
export const useAdminToast = () => useMetroToast({ variant: 'admin' });
export const useAnalyticsToast = () => useMetroToast({ variant: 'analytics' });