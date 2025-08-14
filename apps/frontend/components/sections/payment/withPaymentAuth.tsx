'use client'

import { ComponentType } from 'react'
import { useRouter } from 'next/navigation'
import { useAuth, usePackageTier } from '@/lib/auth'

/**
 * Higher-order component for payment authentication
 * Ensures user is authenticated and has payment access
 */
export function withPaymentAuth<P extends object>(
  WrappedComponent: ComponentType<P>
) {
  const PaymentAuthComponent = (props: P) => {
    const router = useRouter()
    const { user, isAuthenticated, isLoading } = useAuth()
    const { hasRequiredTier, currentTier } = usePackageTier('BRONZE')
    
    // Payment access means user has at least BRONZE tier
    const hasPaymentAccess = hasRequiredTier

    // Show loading state
    if (isLoading) {
      return (
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center">
            <div className="relative">
              <div className="w-16 h-16 border-4 border-gray-200 dark:border-gray-700 rounded-full animate-spin"></div>
              <div className="absolute top-0 left-0 w-16 h-16 border-4 border-transparent border-t-blue-500 rounded-full animate-spin"></div>
            </div>
            <p className="mt-4 text-gray-600 dark:text-gray-400">
              Verifying payment access...
            </p>
          </div>
        </div>
      )
    }

    // Show authentication required message
    if (!isAuthenticated) {
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
              You need to be logged in to access payment features.
            </p>
            <button
              onClick={() => router.push('/login')}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Sign In
            </button>
          </div>
        </div>
      )
    }

    // Show payment access required message
    if (!hasPaymentAccess) {
      return (
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center max-w-md mx-auto p-6">
            <div className="w-16 h-16 bg-yellow-100 dark:bg-yellow-900 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-yellow-600" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
              </svg>
            </div>
            <h3 className="text-lg font-semibold mb-2">Premium Access Required</h3>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              You need at least a Bronze subscription to access payment features. Current tier: {currentTier}
            </p>
            <button
              onClick={() => router.push('/payment')}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Upgrade Plan
            </button>
          </div>
        </div>
      )
    }

    // Render the wrapped component if authenticated and has payment access
    return <WrappedComponent {...props} />
  }

  PaymentAuthComponent.displayName = `withPaymentAuth(${WrappedComponent.displayName || WrappedComponent.name})`

  return PaymentAuthComponent
}