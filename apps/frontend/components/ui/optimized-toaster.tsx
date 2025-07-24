'use client';

import { useEffect, useState, useCallback, useMemo } from 'react';
import { CheckCircle, XCircle, AlertCircle, X } from 'lucide-react';
import { cn } from '@/lib/utils';

interface Toast {
  id: string;
  title: string;
  description?: string;
  type: 'success' | 'error' | 'warning' | 'info';
  duration?: number;
}

let globalToastManager: ((toast: Omit<Toast, 'id'>) => void) | null = null;

/**
 * Optimized toast system with minimal re-renders
 */
export function OptimizedToaster() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((props: Omit<Toast, 'id'>) => {
    const id = Math.random().toString(36).substring(2, 9);
    const duration = props.duration || (props.type === 'error' ? 8000 : 5000);
    
    setToasts(prev => [...prev, { ...props, id, duration }]);
    
    // Auto-remove toast
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, duration);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  // Set global toast manager
  useEffect(() => {
    globalToastManager = addToast;
    return () => {
      globalToastManager = null;
    };
  }, [addToast]);

  const iconMap = useMemo(() => ({
    success: CheckCircle,
    error: XCircle,
    warning: AlertCircle,
    info: AlertCircle,
  }), []);

  const colorMap = useMemo(() => ({
    success: 'border-green-500 bg-green-50 text-green-900 dark:bg-green-900/20 dark:text-green-100',
    error: 'border-red-500 bg-red-50 text-red-900 dark:bg-red-900/20 dark:text-red-100',
    warning: 'border-yellow-500 bg-yellow-50 text-yellow-900 dark:bg-yellow-900/20 dark:text-yellow-100',
    info: 'border-blue-500 bg-blue-50 text-blue-900 dark:bg-blue-900/20 dark:text-blue-100',
  }), []);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2 max-w-sm">
      {toasts.map(toast => {
        const Icon = iconMap[toast.type];
        return (
          <div
            key={toast.id}
            className={cn(
              'flex items-start space-x-3 rounded-lg border p-4 shadow-lg backdrop-blur-sm transition-all duration-300 animate-in slide-in-from-right',
              colorMap[toast.type]
            )}
          >
            <Icon className="h-5 w-5 mt-0.5 flex-shrink-0" />
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold text-sm truncate">{toast.title}</h3>
              {toast.description && (
                <p className="text-xs opacity-90 mt-1">{toast.description}</p>
              )}
            </div>
            <button
              onClick={() => removeToast(toast.id)}
              className="opacity-70 hover:opacity-100 transition-opacity p-1 -m-1"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
}

// Global toast functions
export const toast = {
  success: (title: string, description?: string) => {
    globalToastManager?.({ type: 'success', title, description });
  },
  error: (title: string, description?: string) => {
    globalToastManager?.({ type: 'error', title, description });
  },
  warning: (title: string, description?: string) => {
    globalToastManager?.({ type: 'warning', title, description });
  },
  info: (title: string, description?: string) => {
    globalToastManager?.({ type: 'info', title, description });
  },
};