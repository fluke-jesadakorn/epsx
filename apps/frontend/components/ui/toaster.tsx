'use client';

import * as React from 'react';
import { useState } from 'react';
import { CheckCircle, XCircle, AlertCircle, X } from 'lucide-react';
import { cn } from '@/lib/utils';

interface Toast {
  id: string;
  title: string;
  description?: string;
  type: 'success' | 'error' | 'warning';
}

const ToastContext = React.createContext<{
  toast: (props: Omit<Toast, 'id'>) => void;
}>({ toast: () => {} });

export function useToast() {
  return React.useContext(ToastContext);
}

export function Toaster() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = (props: Omit<Toast, 'id'>) => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts(prev => [...prev, { ...props, id }]);
    
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, 5000);
  };

  const removeToast = (id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  };

  const icons = {
    success: CheckCircle,
    error: XCircle,
    warning: AlertCircle,
  };

  const colors = {
    success: 'border-green-500 bg-green-50 text-green-900',
    error: 'border-red-500 bg-red-50 text-red-900',
    warning: 'border-yellow-500 bg-yellow-50 text-yellow-900',
  };

  return (
    <ToastContext.Provider value={{ toast }}>
      <div className="fixed top-0 right-0 z-50 p-4 space-y-2">
        {toasts.map(toast => {
          const Icon = icons[toast.type];
          return (
            <div
              key={toast.id}
              className={cn(
                'flex items-start space-x-2 rounded-lg border p-4 shadow-lg max-w-sm',
                colors[toast.type]
              )}
            >
              <Icon className="h-5 w-5 mt-0.5" />
              <div className="flex-1">
                <h3 className="font-semibold">{toast.title}</h3>
                {toast.description && (
                  <p className="text-sm opacity-90">{toast.description}</p>
                )}
              </div>
              <button
                onClick={() => removeToast(toast.id)}
                className="opacity-70 hover:opacity-100"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          );
        })}
      </div>
    </ToastContext.Provider>
  );
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  return (
    <>
      {children}
      <Toaster />
    </>
  );
}
