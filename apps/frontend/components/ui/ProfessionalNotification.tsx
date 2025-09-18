'use client';

import { useEffect, useState } from 'react';
import { cn } from '@/lib/utils';
import { CheckCircle, AlertCircle, Info, XCircle, X } from 'lucide-react';

interface ProfessionalNotificationProps {
  type?: 'info' | 'success' | 'warning' | 'error';
  variant?: 'default' | 'analytics' | 'premium';
  title?: string;
  message: string;
  icon?: React.ReactNode;
  duration?: number;
  onClose?: () => void;
  visible?: boolean;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
  className?: string;
}

export function ProfessionalNotification({
  type = 'info',
  variant = 'default',
  title,
  message,
  icon,
  duration = 5000,
  onClose,
  visible = true,
  position = 'top-right',
  className
}: ProfessionalNotificationProps) {
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
      bg: 'bg-blue-50 border-blue-200',
      text: 'text-blue-900',
      icon: <Info className="w-5 h-5 text-blue-500" />,
      accent: 'bg-blue-500'
    },
    success: {
      bg: 'bg-green-50 border-green-200',
      text: 'text-green-900',
      icon: <CheckCircle className="w-5 h-5 text-green-500" />,
      accent: 'bg-green-500'
    },
    warning: {
      bg: 'bg-yellow-50 border-yellow-200',
      text: 'text-yellow-900',
      icon: <AlertCircle className="w-5 h-5 text-yellow-500" />,
      accent: 'bg-yellow-500'
    },
    error: {
      bg: 'bg-red-50 border-red-200',
      text: 'text-red-900',
      icon: <XCircle className="w-5 h-5 text-red-500" />,
      accent: 'bg-red-500'
    }
  };

  const variantStyles = {
    default: 'shadow-lg',
    analytics: 'shadow-lg backdrop-blur-sm bg-white/95',
    premium: 'shadow-xl backdrop-blur-md bg-gradient-to-r from-white to-blue-50/50'
  };

  const typeStyle = typeStyles[type];

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
          'border rounded-lg overflow-hidden',
          typeStyle.bg,
          variantStyles[variant]
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
            <div className="flex-shrink-0 mt-0.5">
              {icon || typeStyle.icon}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0">
              {title && (
                <h4 className={cn('font-semibold text-sm mb-1', typeStyle.text)}>
                  {title}
                </h4>
              )}
              <p className={cn('text-sm', typeStyle.text, !title && 'font-medium')}>
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
                typeStyle.text
              )}
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>
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
interface ToastNotification extends Omit<ProfessionalNotificationProps, 'visible' | 'onClose'> {
  id: string;
}

interface UseProfessionalToastProps {
  variant?: 'default' | 'analytics' | 'premium';
  position?: ProfessionalNotificationProps['position'];
  defaultDuration?: number;
}

export function useProfessionalToast({ 
  variant = 'default', 
  position = 'top-right',
  defaultDuration = 5000 
}: UseProfessionalToastProps = {}) {
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
            <ProfessionalNotification
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

// Professional Alert Component (static, non-dismissible)
interface ProfessionalAlertProps {
  type?: 'info' | 'success' | 'warning' | 'error';
  variant?: 'default' | 'analytics' | 'premium';
  title?: string;
  children: React.ReactNode;
  icon?: React.ReactNode;
  className?: string;
}

export function ProfessionalAlert({
  type = 'info',
  variant = 'default',
  title,
  children,
  icon,
  className
}: ProfessionalAlertProps) {
  const typeStyles = {
    info: {
      bg: 'bg-blue-50 border-blue-200',
      text: 'text-blue-900',
      icon: <Info className="w-5 h-5 text-blue-500" />
    },
    success: {
      bg: 'bg-green-50 border-green-200',
      text: 'text-green-900',
      icon: <CheckCircle className="w-5 h-5 text-green-500" />
    },
    warning: {
      bg: 'bg-yellow-50 border-yellow-200',
      text: 'text-yellow-900',
      icon: <AlertCircle className="w-5 h-5 text-yellow-500" />
    },
    error: {
      bg: 'bg-red-50 border-red-200',
      text: 'text-red-900',
      icon: <XCircle className="w-5 h-5 text-red-500" />
    }
  };

  const variantStyles = {
    default: '',
    analytics: 'backdrop-blur-sm bg-white/95',
    premium: 'shadow-md bg-gradient-to-r from-white to-blue-50/50'
  };

  const typeStyle = typeStyles[type];

  return (
    <div
      className={cn(
        'border rounded-lg p-4',
        typeStyle.bg,
        variantStyles[variant],
        className
      )}
    >
      <div className="flex gap-3">
        <div className="flex-shrink-0">
          {icon || typeStyle.icon}
        </div>
        <div className="flex-1">
          {title && (
            <h4 className={cn('font-semibold text-sm mb-1', typeStyle.text)}>
              {title}
            </h4>
          )}
          <div className={cn('text-sm', typeStyle.text)}>
            {children}
          </div>
        </div>
      </div>
    </div>
  );
}