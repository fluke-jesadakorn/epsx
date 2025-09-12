/**
 * Error page for user edit
 * Shows error state with retry functionality
 */

'use client'

import { useEffect } from 'react'
import { AlertCircle, RefreshCw, ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PancakeCard } from '@/components/ui/PancakeCard'

interface UserEditErrorProps {
  error: Error & { digest?: string }
  reset: () => void
}

export default function UserEditError({ error, reset }: UserEditErrorProps) {
  useEffect(() => {
    console.error('User edit error:', error)
  }, [error])

  return (
    <div className="min-h-screen bg-gradient-to-br from-red-50 via-orange-50 to-yellow-50 dark:from-gray-900 dark:via-red-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-red-400/20 to-orange-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-orange-400/20 to-yellow-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-yellow-400/15 to-red-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-4xl mx-auto">
        <div className="text-center mb-8">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-red-600 via-orange-600 to-yellow-600 bg-clip-text text-transparent mb-4">
              ⚠️ Something went wrong
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-red-400 to-orange-500 rounded-full animate-pulse"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            We encountered an error while loading the user edit page
          </p>
        </div>
        
        <PancakeCard className="p-8 text-center">
          <div className="flex flex-col items-center gap-6">
            <div className="w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
              <AlertCircle className="w-8 h-8 text-red-500" />
            </div>
            
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-200">
                Unable to Load User Data
              </h2>
              <p className="text-gray-600 dark:text-gray-400 max-w-md">
                {error.message || 'An unexpected error occurred while trying to load the user information for editing.'}
              </p>
              {error.digest && (
                <p className="text-sm text-gray-500 dark:text-gray-500">
                  Error ID: {error.digest}
                </p>
              )}
            </div>

            <div className="flex gap-3">
              <Button
                variant="outline"
                onClick={() => window.history.back()}
                className="rounded-2xl"
              >
                <ArrowLeft className="w-4 h-4 mr-2" />
                Go Back
              </Button>
              
              <Button
                onClick={reset}
                className="bg-gradient-to-r from-red-500 to-orange-500 hover:from-red-600 hover:to-orange-600 rounded-2xl"
              >
                <RefreshCw className="w-4 h-4 mr-2" />
                Try Again
              </Button>
            </div>

            <div className="text-sm text-gray-500 dark:text-gray-400 border-t border-gray-200 dark:border-gray-700 pt-4 mt-4">
              <p>If this problem persists, please contact your system administrator</p>
            </div>
          </div>
        </PancakeCard>
      </div>
    </div>
  )
}