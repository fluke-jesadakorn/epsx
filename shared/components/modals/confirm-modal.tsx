
import React from 'react'
import { BaseModal, type BaseModalProps } from './base-modal'

export interface ConfirmModalProps extends Omit<BaseModalProps, 'children' | 'primaryAction' | 'secondaryAction'> {
    message: React.ReactNode
    confirmLabel?: string
    cancelLabel?: string
    onConfirm: () => void | Promise<void>
    onCancel?: () => void
    variant?: 'default' | 'destructive'
}

export const ConfirmModal: React.FC<ConfirmModalProps> = ({
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    onConfirm,
    onCancel,
    variant = 'default',
    onClose,
    ...props
}) => {
    const handleCancel = () => {
        onCancel?.()
        onClose()
    }

    const handleConfirm = async () => {
        try {
            await onConfirm()
            onClose()
        } catch (_error) {
            // Keep modal open on error
            // console.error('Confirmation action failed:', _error)
        }
    }

    return (
        <BaseModal
            size="sm"
            onClose={onClose}
            primaryAction={{
                label: confirmLabel,
                onClick: handleConfirm,
                variant: variant === 'destructive' ? 'destructive' : 'primary'
            }}
            secondaryAction={{
                label: cancelLabel,
                onClick: handleCancel
            }}
            {...props}
        >
            <div className="text-center">
                {variant === 'destructive' && (
                    <div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100 dark:bg-red-900/20 mb-4">
                        <svg className="h-6 w-6 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                        </svg>
                    </div>
                )}
                <div className="text-sm text-gray-700 dark:text-gray-300">
                    {message}
                </div>
            </div>
        </BaseModal>
    )
}
