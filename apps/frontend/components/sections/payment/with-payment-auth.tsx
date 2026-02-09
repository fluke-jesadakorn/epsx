'use client'

import { useAuth } from '@/lib/auth'
import { useRouter } from 'next/navigation'
import type { ComponentType} from 'react';
import { forwardRef } from 'react'

interface PaymentAuthUser {
  permissions?: string[] | Record<string, unknown>;
  [key: string]: unknown;
}

interface PaymentAuthState {
  isLoading: boolean;
  isAuthenticated: boolean;
  hasPaymentAccess: boolean;
  user: PaymentAuthUser | null;
  router: ReturnType<typeof useRouter>;
}

interface PaymentAccessRequiredUIProps {
  user: PaymentAuthUser | null;
  router: ReturnType<typeof useRouter>;
}

// Hook for payment auth logic
function usePaymentAuth(): PaymentAuthState {
  const router = useRouter()
  const { user, isLoading } = useAuth()
  const isAuthenticated = Boolean(user)

  const hasPaymentAccess = user?.permissions ?
    (Array.isArray(user.permissions)
      ? user.permissions.some(p =>
        p.includes('epsx:') || p.includes('premium:') || p.includes('payments:')
      )
      : Object.keys(user.permissions).some(p =>
        p.includes('epsx:') || p.includes('premium:') || p.includes('payments:')
      )
    ) : true

  return {
    isLoading,
    isAuthenticated,
    hasPaymentAccess,
    user,
    router
  }
}

// Loading UI
function LoadingUI() {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="text-center">
        <div className="text-blue-500 text-lg mb-4">
          Verifying payment access...
        </div>
      </div>
    </div>
  )
}

// Authentication required UI
function AuthRequiredUI() {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="text-center max-w-md mx-auto p-6">
        <div className="w-16 h-16 bg-red-100 dark:bg-red-900 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg className="w-8 h-8 text-red-600" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
          </svg>
        </div>
        <h3 className="text-lg font-semibold mb-2">Authentication Required</h3>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          Please connect your wallet to access payment features.
        </p>
        <p className="text-sm text-gray-500">
          Use the wallet button in the navigation menu to connect.
        </p>
      </div>
    </div>
  )
}

// Payment access required UI
function PaymentAccessRequiredUI({ user, router }: PaymentAccessRequiredUIProps) {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="text-center max-w-md mx-auto p-6">
        <div className="w-16 h-16 bg-yellow-100 dark:bg-yellow-900 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg className="w-8 h-8 text-yellow-600" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
          </svg>
        </div>
        <h3 className="text-lg font-semibold mb-2">Payment Access Setup Required</h3>
        <p className="text-gray-600 dark:text-gray-400 mb-2">
          Your account is authenticated but needs payment access setup.
        </p>
        <p className="text-xs text-gray-500 mb-4">
          Debug: Permissions found: {user?.permissions ? JSON.stringify(user.permissions) : 'None'}
        </p>
        <button
          onClick={() => window.location.reload()}
          className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 mr-2"
        >
          Refresh Page
        </button>
        <button
          onClick={() => router.push('/')}
          className="px-6 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700"
        >
          Go Home
        </button>
      </div>
    </div>
  )
}

/**
 * Higher-order component for payment authentication
 * Ensures user is authenticated and has payment access
 */
export function withPaymentAuth<P extends object>(
  WrappedComponent: ComponentType<P>
) {
  const PaymentAuthComponent = forwardRef<unknown, P>((props: P, ref: React.Ref<unknown>) => {
    const { isLoading, isAuthenticated, hasPaymentAccess, user, router } = usePaymentAuth()

    if (isLoading) {return <LoadingUI />}
    if (!isAuthenticated) {return <AuthRequiredUI />}
    if (!hasPaymentAccess) {return <PaymentAccessRequiredUI user={user} router={router} />}

    return <WrappedComponent {...props} ref={ref} />
  })

  PaymentAuthComponent.displayName = `withPaymentAuth(${WrappedComponent.displayName ?? WrappedComponent.name})`

  return PaymentAuthComponent as unknown as ComponentType<P>
}