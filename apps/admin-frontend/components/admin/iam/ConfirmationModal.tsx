import React from 'react';
import { X, AlertTriangle, Info, AlertCircle } from 'lucide-react';
import type { ConfirmationDialog } from './useConfirmation';

interface ConfirmationModalProps {
  dialog: ConfirmationDialog;
}

export const ConfirmationModal: React.FC<ConfirmationModalProps> = ({ dialog }) => {
  if (!dialog.isOpen) return null;

  const getIcon = () => {
    switch (dialog.type) {
      case 'danger':
        return <AlertCircle className="h-6 w-6 text-red-600" />;
      case 'warning':
        return <AlertTriangle className="h-6 w-6 text-yellow-600" />;
      default:
        return <Info className="h-6 w-6 text-blue-600" />;
    }
  };

  const getButtonColor = () => {
    switch (dialog.type) {
      case 'danger':
        return 'bg-red-600 hover:bg-red-700 focus:ring-red-500';
      case 'warning':
        return 'bg-yellow-600 hover:bg-yellow-700 focus:ring-yellow-500';
      default:
        return 'bg-blue-600 hover:bg-blue-700 focus:ring-blue-500';
    }
  };

  return (
    <div className="fixed inset-0 modal-overlay flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full modal-enter">
        <div className="p-6">
          <div className="flex items-center mb-4">
            <div className="flex-shrink-0">
              {getIcon()}
            </div>
            <div className="ml-3">
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                {dialog.title}
              </h3>
            </div>
            <div className="ml-auto">
              <button
                onClick={dialog.onCancel}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                <X className="h-6 w-6" />
              </button>
            </div>
          </div>
          
          <div className="mb-6">
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {dialog.message}
            </p>
          </div>
          
          <div className="flex justify-end gap-3">
            <button
              onClick={dialog.onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
            >
              {dialog.cancelText}
            </button>
            <button
              onClick={dialog.onConfirm}
              className={`px-4 py-2 text-sm font-medium text-white border border-transparent rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 transition-colors ${getButtonColor()}`}
            >
              {dialog.confirmText}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
