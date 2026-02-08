/**
 * BASE MODAL COMPONENT
 * Unified modal component to replace duplicate modal implementations
 * Consolidates SubscriptionDetailsModal, WalletConnectionModal, etc.
 */

"use client"

import React, { useEffect, useRef } from 'react'
import { createPortal } from 'react-dom'
import { cn } from '../../utils'
import { BaseButton } from '../buttons/base-button'

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

  if (!mounted) { return null }

  return createPortal(
    children,
    document.body
  )
}

// ============================================================================
// BASE MODAL COMPONENT
// ============================================================================

/**
 * Modal Sub-components to keep BaseModal clean
 */

interface ModalHeaderProps {
  title?: React.ReactNode;
  description?: React.ReactNode;
  onClose: () => void;
  preventClose?: boolean;
}

const ModalHeader: React.FC<ModalHeaderProps> = ({ title, description, onClose, preventClose }) => {
  if (!title && !description) { return null; }

  return (
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
              'flex-shrink-0 ml-4 p-1 rounded-md',
              'text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700',
              'transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500'
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
  );
};

interface ModalFooterProps {
  footer?: React.ReactNode;
  primaryAction?: BaseModalProps['primaryAction'];
  secondaryAction?: BaseModalProps['secondaryAction'];
  primaryActionLoading: boolean;
  onPrimaryClick: () => void;
}

const ModalFooter: React.FC<ModalFooterProps> = ({
  footer,
  primaryAction,
  secondaryAction,
  primaryActionLoading,
  onPrimaryClick
}) => {
  if (!footer && !primaryAction && !secondaryAction) { return null; }

  return (
    <div className="px-6 py-4 bg-gray-50 dark:bg-gray-700 border-t border-gray-200 dark:border-gray-600">
      {footer ?? (
        <div className="flex justify-end space-x-3">
          {secondaryAction !== undefined && (
            <BaseButton
              variant="ghost"
              onClick={secondaryAction.onClick}
              disabled={(secondaryAction.disabled ?? false) || primaryActionLoading}
            >
              {secondaryAction.label}
            </BaseButton>
          )}
          {primaryAction !== undefined && (
            <BaseButton
              variant={primaryAction.variant ?? 'primary'}
              onClick={onPrimaryClick}
              loading={primaryActionLoading}
              disabled={primaryAction.disabled}
            >
              {primaryAction.label}
            </BaseButton>
          )}
        </div>
      )}
    </div>
  );
};

// ============================================================================
// BASE MODAL COMPONENT
// ============================================================================

export const BaseModal = React.forwardRef<HTMLDivElement, BaseModalProps>((props, ref) => {
  const {
    isOpen, onClose, title, description, children, footer,
    primaryAction, secondaryAction, closeOnEscape = true,
    closeOnOverlayClick = true, preventClose = false,
    size = 'md', position = 'center', className, overlayClassName,
    contentClassName, 'aria-label': ariaLabel, 'aria-describedby': ariaDescribedBy,
    onAfterOpen, onAfterClose
  } = props;

  const internalRef = useRef<HTMLDivElement>(null);
  const [primaryActionLoading, setPrimaryActionLoading] = React.useState(false);

  // Re-sync with incoming ref if it exists
  useEffect(() => {
    if (!ref) { return; }
    if (typeof ref === 'function') {
      ref(internalRef.current);
    } else {
      (ref as React.MutableRefObject<HTMLDivElement | null>).current = internalRef.current;
    }
  }, [ref]);

  // Handle escape key
  useEffect(() => {
    if (!isOpen || !closeOnEscape) { return; }
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !preventClose) { onClose(); }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, closeOnEscape, preventClose, onClose]);

  // Handle body scroll lock & events
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
      onAfterOpen?.();
      internalRef.current?.focus();
    } else {
      document.body.style.overflow = 'unset';
      onAfterClose?.();
    }
    return () => { document.body.style.overflow = 'unset'; };
  }, [isOpen, onAfterOpen, onAfterClose]);

  if (!isOpen) { return null; }

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (closeOnOverlayClick && !preventClose && e.target === e.currentTarget) {
      onClose();
    }
  };

  const handlePrimaryClick = async () => {
    if (!primaryAction || primaryActionLoading) { return; }
    setPrimaryActionLoading(true);
    try {
      await primaryAction.onClick();
    } catch (_error) {
      // Failed silently
    } finally {
      setPrimaryActionLoading(false);
    }
  };

  return (
    <Portal>
      <div
        className={cn(
          'fixed inset-0 z-50 flex justify-center bg-black/50 backdrop-blur-sm transition-opacity duration-300 opacity-100',
          positionVariants[position],
          overlayClassName
        )}
        onClick={handleOverlayClick}
        role="dialog"
        aria-modal="true"
        aria-label={ariaLabel ?? (typeof title === 'string' ? title : 'Modal')}
        aria-describedby={ariaDescribedBy}
      >
        <div
          ref={internalRef}
          className={cn(
            'relative bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full mx-4 max-h-[90vh] overflow-hidden transition-all duration-300 focus:outline-none opacity-100 scale-100 translate-y-0',
            sizeVariants[size],
            className
          )}
          tabIndex={-1}
        >
          <ModalHeader title={title} description={description} onClose={onClose} preventClose={preventClose} />

          <div className={cn('px-6 py-4 overflow-y-auto max-h-[calc(90vh-200px)]', contentClassName)}>
            {children}
          </div>

          <ModalFooter
            footer={footer}
            primaryAction={primaryAction}
            secondaryAction={secondaryAction}
            primaryActionLoading={primaryActionLoading}
            onPrimaryClick={() => { void handlePrimaryClick(); }}
          />
        </div>
      </div>
    </Portal>
  );
});
BaseModal.displayName = 'base-modal';

// (Removed ConfirmModal and FormModal - they are now in their own files)