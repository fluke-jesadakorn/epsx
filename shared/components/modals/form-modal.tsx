
import React from 'react'
import { BaseModal, type BaseModalProps } from './base-modal'

export interface FormModalProps extends Omit<BaseModalProps, 'children'> {
    children: React.ReactNode
    onSubmit?: (event: React.FormEvent) => void | Promise<void>
    submitLabel?: string
    cancelLabel?: string
    isSubmitting?: boolean
}

export const FormModal: React.FC<FormModalProps> = ({
    children,
    onSubmit,
    submitLabel = 'Submit',
    cancelLabel = 'Cancel',
    isSubmitting = false,
    onClose,
    ...props
}) => {
    const handleSubmit = async (event: React.FormEvent) => {
        event.preventDefault()
        try {
            await onSubmit?.(event)
        } catch (_error) {
            // console.error('Form submission failed:', _error)
        }
    }

    return (
        <BaseModal
            onClose={onClose}
            primaryAction={onSubmit ? {
                label: submitLabel,
                onClick: () => handleSubmit({ preventDefault: () => { } } as React.FormEvent),
                loading: isSubmitting
            } : undefined}
            secondaryAction={{
                label: cancelLabel,
                onClick: onClose,
                disabled: isSubmitting
            }}
            preventClose={isSubmitting}
            {...props}
        >
            <form onSubmit={(e) => { void handleSubmit(e); }}>
                {children}
            </form>
        </BaseModal>
    )
}
