import React from 'react';
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-react';
import type { Toast } from './useToast';

interface ToastContainerProps {
  toasts: Toast[];
  onRemove: (id: string) => void;
}

export const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onRemove }) => {
  const getToastIcon = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return <CheckCircle className="h-5 w-5 text-green-400" />;
      case 'error':
        return <AlertCircle className="h-5 w-5 text-red-400" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-yellow-400" />;
      case 'info':
        return <Info className="h-5 w-5 text-blue-400" />;
    }
  };

  const getToastColors = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800';
      case 'error':
        return 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800';
      case 'warning':
        return 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800';
      case 'info':
        return 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800';
    }
  };

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`max-w-sm w-full shadow-lg rounded-lg pointer-events-auto border transform transition-all duration-300 ease-out ${getToastColors(toast.type)}`}
          style={{
            animation: 'slideIn 0.3s ease-out'
          }}
        >
          <div className="p-4">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                {getToastIcon(toast.type)}
              </div>
              <div className="ml-3 w-0 flex-1">
                <p className="text-sm font-medium text-gray-900">
                  {toast.message}
                </p>
              </div>
              <div className="ml-4 flex-shrink-0 flex">
                <button
                  className="inline-flex text-gray-400 hover:text-gray-500 focus:outline-none"
                  onClick={() => onRemove(toast.id)}
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};
