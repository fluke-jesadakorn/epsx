'use client';

import { motion, AnimatePresence } from 'framer-motion';
import { useEffect, useState } from 'react';

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

  return (
    <AnimatePresence>
      {isVisible && (
        <motion.div
          className={`fixed ${positions[position]} z-50 max-w-sm w-full`}
          initial={{ x: position.includes('right') ? 400 : position.includes('left') ? -400 : 0, y: position.includes('top') ? -100 : position.includes('bottom') ? 100 : 0, opacity: 0 }}
          animate={{ x: 0, y: 0, opacity: 1 }}
          exit={{ x: position.includes('right') ? 400 : position.includes('left') ? -400 : 0, y: position.includes('top') ? -100 : position.includes('bottom') ? 100 : 0, opacity: 0 }}
          transition={{ type: 'spring', damping: 20, stiffness: 300 }}
        >
          <motion.div
            className={`
              ${variantStyle.bg}
              ${variantStyle.text}
              shadow-2xl
              overflow-hidden
              border-l-4 ${variantStyle.border}
              backdrop-blur-xl
            `}
            whileHover={{ scale: 1.02, y: -2 }}
            layout
          >
            {/* Windows Phone Live Tile Animation */}
            <motion.div
              className="absolute inset-0 opacity-5"
              animate={{
                background: [
                  'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)',
                  'linear-gradient(135deg, transparent 40%, rgba(255,255,255,0.1) 60%, transparent 80%)',
                  'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)'
                ]
              }}
              transition={{ duration: 4, repeat: Infinity }}
            />

            {/* Progress Bar */}
            <motion.div
              className={`h-1 ${typeStyle.bg}`}
              initial={{ width: '100%' }}
              animate={{ width: '0%' }}
              transition={{ duration: duration / 1000, ease: 'linear' }}
            />

            <div className="p-4">
              <div className="flex items-start">
                {/* Icon Section */}
                <motion.div
                  className={`${typeStyle.bg} text-white rounded-none w-10 h-10 flex items-center justify-center mr-3 flex-shrink-0`}
                  animate={{ rotate: [0, 5, 0, -5, 0] }}
                  transition={{ duration: 2, repeat: Infinity }}
                >
                  <span className="text-lg font-bold">
                    {icon || typeStyle.icon}
                  </span>
                </motion.div>

                {/* Content */}
                <div className="flex-1 min-w-0">
                  {title && (
                    <motion.h4
                      className="font-bold text-sm mb-1"
                      initial={{ opacity: 0, y: -10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ delay: 0.1 }}
                    >
                      {title}
                    </motion.h4>
                  )}
                  <motion.p
                    className="text-sm opacity-90"
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.2 }}
                  >
                    {message}
                  </motion.p>
                </div>

                {/* Close Button */}
                <motion.button
                  onClick={() => {
                    setIsVisible(false);
                    onClose?.();
                  }}
                  className="ml-3 opacity-50 hover:opacity-100 transition-opacity text-lg"
                  whileHover={{ scale: 1.1 }}
                  whileTap={{ scale: 0.9 }}
                >
                  ×
                </motion.button>
              </div>
            </div>

            {/* Metro accent strip */}
            <motion.div
              className={`h-1 ${typeStyle.accent}`}
              initial={{ width: '0%' }}
              animate={{ width: '100%' }}
              transition={{ duration: 0.5, delay: 0.3 }}
            />
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
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
      <AnimatePresence>
        {toasts.map((toast, index) => (
          <motion.div
            key={toast.id}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            style={{
              position: 'absolute',
              [position.includes('top') ? 'top' : 'bottom']: `${(index * 80) + 16}px`,
              [position.includes('right') ? 'right' : position.includes('left') ? 'left' : 'left']: position.includes('center') ? '50%' : '16px',
              transform: position.includes('center') ? 'translateX(-50%)' : undefined,
              pointerEvents: 'auto'
            }}
          >
            <MetroNotification
              {...toast}
              visible={true}
              position="top-right" // Override since we handle positioning manually
              onClose={() => removeToast(toast.id)}
            />
          </motion.div>
        ))}
      </AnimatePresence>
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