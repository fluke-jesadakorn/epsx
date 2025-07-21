import { useState, useCallback } from 'react';

export interface ConfirmationDialog {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel: () => void;
  type?: 'danger' | 'warning' | 'info';
}

export const useConfirmation = () => {
  const [dialog, setDialog] = useState<ConfirmationDialog | null>(null);

  const confirm = useCallback((options: {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    type?: 'danger' | 'warning' | 'info';
  }): Promise<boolean> => {
    return new Promise((resolve) => {
      setDialog({
        isOpen: true,
        title: options.title,
        message: options.message,
        confirmText: options.confirmText || 'Confirm',
        cancelText: options.cancelText || 'Cancel',
        type: options.type || 'info',
        onConfirm: () => {
          setDialog(null);
          resolve(true);
        },
        onCancel: () => {
          setDialog(null);
          resolve(false);
        },
      });
    });
  }, []);

  const close = useCallback(() => {
    setDialog(null);
  }, []);

  return {
    dialog,
    confirm,
    close,
  };
};
