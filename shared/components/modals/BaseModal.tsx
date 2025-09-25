/**
 * BASE MODAL COMPONENT
 * Unified modal component to replace duplicate modal implementations
 * Consolidates SubscriptionDetailsModal, WalletConnectionModal, etc.
 */

"use client"

import React, { useEffect, useRef } from 'react'
import { cn } from '../../utils'
import { BaseButton } from '../buttons/BaseButton'

// ============================================================================
// MODAL TYPES
// ============================================================================

export type ModalSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full'

export type ModalPosition = 'center' | 'top' | 'bottom'

// ============================================================================
// MODAL COMPONENT PROPS
// ============================================================================

export interface BaseModalProps {
  // Visibility
  isOpen: boolean
  onClose: () => void
  
  // Content
  title?: React.ReactNode
  description?: React.ReactNode
  children: React.ReactNode
  
  // Footer
  footer?: React.ReactNode
  primaryAction?: {
    label: string
    onClick: () => void | Promise<void>
    loading?: boolean
    disabled?: boolean
    variant?: 'primary' | 'destructive'
  }
  secondaryAction?: {
    label: string
    onClick: () => void
    disabled?: boolean
  }
  
  // Behavior
  closeOnEscape?: boolean
  closeOnOverlayClick?: boolean
  preventClose?: boolean
  
  // Layout
  size?: ModalSize
  position?: ModalPosition
  
  // Styling
  className?: string
  overlayClassName?: string
  contentClassName?: string
  
  // Accessibility
  'aria-label'?: string
  'aria-describedby'?: string
  
  // Events
  onAfterOpen?: () => void
  onAfterClose?: () => void
}

// ============================================================================
// MODAL SIZE VARIANTS
// ============================================================================

const sizeVariants = {
  xs: 'max-w-xs',
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  '2xl': 'max-w-2xl',
  full: 'max-w-full m-4'
}

const positionVariants = {
  center: 'items-center',
  top: 'items-start pt-20',
  bottom: 'items-end pb-20'
}

// ============================================================================
// PORTAL COMPONENT
// ============================================================================

interface PortalProps {
  children: React.ReactNode
}

const Portal: React.FC<PortalProps> = ({ children }) => {
  const [mounted, setMounted] = React.useState(false)

  React.useEffect(() => {
    setMounted(true)
    return () => setMounted(false)
  }, [])

  if (!mounted) return null

  return React.createPortal(
    children,
    document.body
  )
}

// ============================================================================
// BASE MODAL COMPONENT
// ============================================================================

export const BaseModal: React.FC<BaseModalProps> = ({
  isOpen,
  onClose,
  title,
  description,
  children,
  footer,
  primaryAction,
  secondaryAction,
  closeOnEscape = true,
  closeOnOverlayClick = true,
  preventClose = false,
  size = 'md',
  position = 'center',
  className,
  overlayClassName,
  contentClassName,
  'aria-label': ariaLabel,
  'aria-describedby': ariaDescribedBy,
  onAfterOpen,
  onAfterClose
}) => {
  const modalRef = useRef<HTMLDivElement>(null)
  const [isAnimating, setIsAnimating] = React.useState(false)
  const [primaryActionLoading, setPrimaryActionLoading] = React.useState(false)

  // Handle escape key
  useEffect(() => {
    if (!isOpen || !closeOnEscape) return

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !preventClose) {
        onClose()
      }
    }

    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [isOpen, closeOnEscape, preventClose, onClose])

  // Handle body scroll lock
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden'
      onAfterOpen?.()
    } else {
      document.body.style.overflow = 'unset'
      onAfterClose?.()
    }

    return () => {
      document.body.style.overflow = 'unset'
    }
  }, [isOpen, onAfterOpen, onAfterClose])

  // Focus management
  useEffect(() => {
    if (isOpen && modalRef.current) {
      modalRef.current.focus()
    }
  }, [isOpen])

  // Handle overlay click
  const handleOverlayClick = (event: React.MouseEvent) => {
    if (
      closeOnOverlayClick &&
      !preventClose &&
      event.target === event.currentTarget
    ) {
      onClose()
    }
  }

  // Handle primary action
  const handlePrimaryAction = async () => {
    if (!primaryAction || primaryActionLoading) return

    try {
      setPrimaryActionLoading(true)
      await primaryAction.onClick()
    } catch (error) {
      console.error('Primary action failed:', error)
    } finally {
      setPrimaryActionLoading(false)
    }
  }

  if (!isOpen) return null

  return (
    <Portal>
      <div
        className={cn(
          // Base overlay styling
          'fixed inset-0 z-50',
          'flex justify-center',
          positionVariants[position],
          'bg-black bg-opacity-50',
          'backdrop-blur-sm',
          'transition-opacity duration-300',
          isAnimating ? 'opacity-0' : 'opacity-100',
          overlayClassName
        )}
        onClick={handleOverlayClick}
        role="dialog"
        aria-modal="true"
        aria-label={ariaLabel || (typeof title === 'string' ? title : 'Modal')}
        aria-describedby={ariaDescribedBy}
      >
        <div
          ref={modalRef}
          className={cn(
            // Base modal styling
            'relative bg-white dark:bg-gray-800',
            'rounded-lg shadow-xl',
            'w-full mx-4',
            sizeVariants[size],
            'max-h-[90vh] overflow-hidden',
            'transition-all duration-300',
            'focus:outline-none',
            // Animation
            isAnimating 
              ? 'opacity-0 scale-95 translate-y-4' 
              : 'opacity-100 scale-100 translate-y-0',
            className
          )}
          tabIndex={-1}
        >
          {/* Header */}
          {(title || description) && (
            <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  {title && (
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                      {title}
                    </h2>
                  )}
                  {description && (
                    <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
                      {description}
                    </p>
                  )}
                </div>
                {!preventClose && (
                  <button
                    onClick={onClose}
                    className={cn(
                      'flex-shrink-0 ml-4',
                      'p-1 rounded-md',
                      'text-gray-400 hover:text-gray-500',
                      'hover:bg-gray-100 dark:hover:bg-gray-700',
                      'transition-colors duration-200',
                      'focus:outline-none focus:ring-2 focus:ring-blue-500'
                    )}
                    aria-label="Close modal"
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                )}
              </div>
            </div>
          )}

          {/* Content */}
          <div className={cn(
            'px-6 py-4',
            'overflow-y-auto',
            'max-h-[calc(90vh-200px)]', // Account for header and footer
            contentClassName
          )}>
            {children}
          </div>

          {/* Footer */}
          {(footer || primaryAction || secondaryAction) && (
            <div className="px-6 py-4 bg-gray-50 dark:bg-gray-700 border-t border-gray-200 dark:border-gray-600">
              {footer || (
                <div className="flex justify-end space-x-3">
                  {secondaryAction && (
                    <BaseButton
                      variant="ghost"
                      onClick={secondaryAction.onClick}
                      disabled={secondaryAction.disabled || primaryActionLoading}
                    >
                      {secondaryAction.label}
                    </BaseButton>
                  )}
                  {primaryAction && (
                    <BaseButton
                      variant={primaryAction.variant || 'primary'}
                      onClick={handlePrimaryAction}
                      loading={primaryActionLoading}
                      disabled={primaryAction.disabled}
                    >
                      {primaryAction.label}
                    </BaseButton>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </Portal>
  )
}

// ============================================================================
// SPECIALIZED MODAL COMPONENTS
// ============================================================================

/**
 * Confirmation Modal - for confirmation dialogs
 */
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
    } catch (error) {
      // Keep modal open on error
      console.error('Confirmation action failed:', error)
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

/**
 * Form Modal - for forms in modals
 */
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
    } catch (error) {
      console.error('Form submission failed:', error)
    }
  }

  return (
    <BaseModal
      onClose={onClose}
      primaryAction={onSubmit ? {
        label: submitLabel,
        onClick: () => handleSubmit(new Event('submit') as any),
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
      <form onSubmit={handleSubmit}>
        {children}
      </form>
    </BaseModal>
  )
}

export default BaseModal