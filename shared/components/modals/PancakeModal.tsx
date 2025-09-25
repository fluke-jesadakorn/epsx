/**
 * PANCAKESWAP THEMED MODAL COMPONENTS
 * PancakeSwap-themed wrapper around BaseModal for admin-frontend
 * Replaces SubscriptionDetailsModal, PlanAnalyticsModal, etc.
 */

"use client"

import * as React from "react"
import {
  BaseModal,
  ConfirmModal as BaseConfirmModal,
  FormModal as BaseFormModal,
  type BaseModalProps,
  type ConfirmModalProps as BaseConfirmModalProps,
  type FormModalProps as BaseFormModalProps
} from './BaseModal'
import { PancakeCard } from '../cards/PancakeCard'
import { PancakeButton } from '../buttons/PancakeButton'
import { cn } from '../../utils'

// ============================================================================
// PANCAKESWAP MODAL VARIANTS
// ============================================================================

interface PancakeModalProps extends Omit<BaseModalProps, 'className'> {
  variant?: 'default' | 'elevated' | 'success' | 'warning' | 'error'
  className?: string
}

interface PancakeConfirmModalProps extends Omit<BaseConfirmModalProps, 'variant'> {
  variant?: 'default' | 'success' | 'warning' | 'destructive'
}

interface PancakeFormModalProps extends BaseFormModalProps {
  variant?: 'default' | 'elevated'
}

// ============================================================================
// PANCAKESWAP MODAL COMPONENT
// ============================================================================

const PancakeModal = React.forwardRef<HTMLDivElement, PancakeModalProps>(({
  variant = 'default',
  className,
  children,
  ...props
}, ref) => {
  const variantStyles = {
    default: 'bg-gradient-to-br from-orange-50/50 to-yellow-50/50 dark:from-orange-950/30 dark:to-yellow-950/30 border-orange-200 dark:border-orange-800',
    elevated: 'bg-gradient-to-br from-orange-100/70 to-yellow-100/70 dark:from-orange-900/50 dark:to-yellow-900/50 border-orange-300 dark:border-orange-700 shadow-2xl',
    success: 'bg-gradient-to-br from-green-50/50 to-emerald-50/50 dark:from-green-950/30 dark:to-emerald-950/30 border-green-200 dark:border-green-800',
    warning: 'bg-gradient-to-br from-yellow-50/50 to-amber-50/50 dark:from-yellow-950/30 dark:to-amber-950/30 border-yellow-200 dark:border-yellow-800',
    error: 'bg-gradient-to-br from-red-50/50 to-pink-50/50 dark:from-red-950/30 dark:to-pink-950/30 border-red-200 dark:border-red-800'
  }

  return (
    <BaseModal
      ref={ref}
      className={cn(
        'rounded-2xl backdrop-blur-sm border-2',
        variantStyles[variant],
        className
      )}
      overlayClassName="bg-black/60 backdrop-blur-md"
      {...props}
    >
      {children}
    </BaseModal>
  )
})
PancakeModal.displayName = "PancakeModal"

// ============================================================================
// PANCAKESWAP MODAL WITH CARD WRAPPER
// ============================================================================

interface PancakeCardModalProps extends PancakeModalProps {
  cardClassName?: string
  cardVariant?: 'pancake' | 'pancakeElevated' | 'pancakeOutlined'
}

const PancakeCardModal = React.forwardRef<HTMLDivElement, PancakeCardModalProps>(({
  cardClassName,
  cardVariant = 'pancakeElevated',
  children,
  ...props
}, ref) => {
  return (
    <PancakeModal ref={ref} {...props}>
      <PancakeCard 
        variant={cardVariant}
        className={cn('border-0 shadow-none bg-transparent', cardClassName)}
      >
        {children}
      </PancakeCard>
    </PancakeModal>
  )
})
PancakeCardModal.displayName = "PancakeCardModal"

// ============================================================================
// PANCAKESWAP CONFIRM MODAL
// ============================================================================

const PancakeConfirmModal: React.FC<PancakeConfirmModalProps> = ({
  variant = 'default',
  message,
  ...props
}) => {
  const getVariantIcon = (variant: string) => {
    switch (variant) {
      case 'success':
        return '✅'
      case 'warning':
        return '⚠️'
      case 'destructive':
        return '🚨'
      default:
        return '❓'
    }
  }

  const getVariantColor = (variant: string) => {
    switch (variant) {
      case 'success':
        return 'from-green-600 to-emerald-600'
      case 'warning':
        return 'from-yellow-600 to-amber-600'
      case 'destructive':
        return 'from-red-600 to-pink-600'
      default:
        return 'from-orange-600 to-yellow-600'
    }
  }

  return (
    <BaseConfirmModal
      {...props}
      variant={variant === 'destructive' ? 'destructive' : 'default'}
      overlayClassName="bg-black/60 backdrop-blur-md"
      className={cn(
        'rounded-2xl backdrop-blur-sm border-2',
        variant === 'success' && 'bg-gradient-to-br from-green-50/50 to-emerald-50/50 dark:from-green-950/30 dark:to-emerald-950/30 border-green-200 dark:border-green-800',
        variant === 'warning' && 'bg-gradient-to-br from-yellow-50/50 to-amber-50/50 dark:from-yellow-950/30 dark:to-amber-950/30 border-yellow-200 dark:border-yellow-800',
        variant === 'destructive' && 'bg-gradient-to-br from-red-50/50 to-pink-50/50 dark:from-red-950/30 dark:to-pink-950/30 border-red-200 dark:border-red-800',
        variant === 'default' && 'bg-gradient-to-br from-orange-50/50 to-yellow-50/50 dark:from-orange-950/30 dark:to-yellow-950/30 border-orange-200 dark:border-orange-800'
      )}
    >
      <div className="text-center space-y-4">
        <div className="text-4xl">{getVariantIcon(variant)}</div>
        <div className={cn(
          'text-lg font-semibold bg-gradient-to-r bg-clip-text text-transparent',
          getVariantColor(variant)
        )}>
          {typeof props.title === 'string' ? props.title : 'Confirm Action'}
        </div>
        <div className="text-gray-700 dark:text-gray-300 text-sm">
          {message}
        </div>
      </div>
    </BaseConfirmModal>
  )
}

// ============================================================================
// PANCAKESWAP FORM MODAL
// ============================================================================

const PancakeFormModal: React.FC<PancakeFormModalProps> = ({
  variant = 'default',
  title,
  className,
  ...props
}) => {
  return (
    <BaseFormModal
      {...props}
      title={
        <div className={cn(
          'text-xl font-bold bg-gradient-to-r bg-clip-text text-transparent',
          variant === 'elevated' 
            ? 'from-orange-600 to-yellow-600' 
            : 'from-orange-500 to-yellow-500'
        )}>
          {title}
        </div>
      }
      overlayClassName="bg-black/60 backdrop-blur-md"
      className={cn(
        'rounded-2xl backdrop-blur-sm border-2',
        variant === 'elevated' 
          ? 'bg-gradient-to-br from-orange-100/70 to-yellow-100/70 dark:from-orange-900/50 dark:to-yellow-900/50 border-orange-300 dark:border-orange-700 shadow-2xl'
          : 'bg-gradient-to-br from-orange-50/50 to-yellow-50/50 dark:from-orange-950/30 dark:to-yellow-950/30 border-orange-200 dark:border-orange-800',
        className
      )}
    />
  )
}

// ============================================================================
// SPECIALIZED PANCAKESWAP MODALS
// ============================================================================

/**
 * Subscription Details Modal - Enhanced version for admin-frontend
 */
interface PancakeSubscriptionModalProps {
  isOpen: boolean
  onClose: () => void
  subscription: {
    id: string
    plan_name: string
    status: string
    user_id: string
    [key: string]: any
  }
  onUpdate?: () => void
  onCancel?: () => void
}

const PancakeSubscriptionModal: React.FC<PancakeSubscriptionModalProps> = ({
  isOpen,
  onClose,
  subscription,
  onUpdate,
  onCancel
}) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'from-green-500 to-emerald-500 text-white'
      case 'expired':
        return 'from-yellow-500 to-amber-500 text-white'
      case 'cancelled':
        return 'from-red-500 to-pink-500 text-white'
      default:
        return 'from-gray-500 to-gray-600 text-white'
    }
  }

  return (
    <PancakeModal
      isOpen={isOpen}
      onClose={onClose}
      size="lg"
      variant="elevated"
      title={
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent">
              Subscription Details
            </h2>
            <div className="flex items-center gap-3 mt-2">
              <span className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                {subscription.plan_name}
              </span>
              <span className={cn(
                'px-3 py-1 text-sm rounded-full font-semibold bg-gradient-to-r',
                getStatusColor(subscription.status)
              )}>
                {subscription.status.toUpperCase()}
              </span>
            </div>
          </div>
        </div>
      }
      footer={
        <div className="flex gap-4">
          <PancakeButton variant="outline" onClick={onClose}>
            Close
          </PancakeButton>
          {subscription.status === 'active' && onCancel && (
            <PancakeButton variant="destructive" onClick={onCancel}>
              Cancel Subscription
            </PancakeButton>
          )}
          {onUpdate && (
            <PancakeButton variant="default" onClick={onUpdate}>
              Update
            </PancakeButton>
          )}
        </div>
      }
    >
      <div className="space-y-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <PancakeCard variant="pancakeOutlined" className="p-4">
            <h3 className="font-bold text-gray-900 dark:text-white mb-3">Basic Information</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">ID:</span>
                <span className="font-mono">{subscription.id}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">User:</span>
                <span className="font-mono">{subscription.user_id}</span>
              </div>
            </div>
          </PancakeCard>

          <PancakeCard variant="pancakeOutlined" className="p-4">
            <h3 className="font-bold text-gray-900 dark:text-white mb-3">Status</h3>
            <div className="text-center">
              <div className={cn(
                'inline-flex items-center px-4 py-2 rounded-full text-lg font-bold bg-gradient-to-r',
                getStatusColor(subscription.status)
              )}>
                {subscription.status.toUpperCase()}
              </div>
            </div>
          </PancakeCard>
        </div>
      </div>
    </PancakeModal>
  )
}

// ============================================================================
// EXPORTS
// ============================================================================

export {
  // Main components
  PancakeModal,
  PancakeCardModal,
  PancakeConfirmModal,
  PancakeFormModal,
  PancakeSubscriptionModal,
  
  // Types
  type PancakeModalProps,
  type PancakeConfirmModalProps,
  type PancakeFormModalProps,
  type PancakeCardModalProps,
  type PancakeSubscriptionModalProps
}

// Default export
export default PancakeModal