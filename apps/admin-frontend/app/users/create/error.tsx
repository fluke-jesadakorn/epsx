/**
 * Error boundary for user creation page
 * Windows Phone + PancakeSwap styled error handling
 */

'use client'

import { User, AlertTriangle, RefreshCw, ArrowLeft } from 'lucide-react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'

export default function CreateUserError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  const router = useRouter()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-2xl mx-auto px-4 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-gradient-to-br from-red-600 to-red-800 rounded-full flex items-center justify-center">
              <User className="h-6 w-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                Create New User
              </h1>
              <p className="text-red-600 dark:text-red-400">
                Failed to load creation form
              </p>
            </div>
          </div>
        </div>

        {/* Error tile */}
        <div className="bg-gradient-to-br from-red-600 to-red-800 text-white rounded-lg shadow-2xl relative overflow-hidden">
          {/* PancakeSwap corner accent */}
          <div className="absolute top-0 right-0 w-8 h-8 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
          
          <div className="p-6">
            {/* Error icon */}
            <div className="flex items-center gap-3 mb-6">
              <div className="w-12 h-12 bg-white/20 rounded-lg flex items-center justify-center">
                <AlertTriangle className="h-6 w-6 text-yellow-300" />
              </div>
              <div>
                <h2 className="text-xl font-extralight tracking-wide">creation error</h2>
                <p className="text-red-200 text-sm font-light">unable to load user form</p>
              </div>
            </div>

            {/* Error details */}
            <div className="mb-6 bg-black/20 p-4 rounded-lg">
              <p className="text-sm font-mono text-red-100 break-words">
                {error.message || 'Failed to initialize user creation form'}
              </p>
              {error.digest && (
                <p className="text-xs text-red-300 mt-2 opacity-75">
                  Error ID: {error.digest}
                </p>
              )}
            </div>

            {/* Windows Phone style action buttons */}
            <div className="space-y-3">
              <Button
                onClick={reset}
                variant="secondary"
                size="default"
                className="w-full group"
              >
                <RefreshCw className="h-4 w-4 group-hover:rotate-180 transition-transform duration-500" />
                retry creation form
              </Button>
              
              <Button
                onClick={() => router.push('/users')}
                variant="pancake"
                size="default"
                className="w-full"
              >
                <ArrowLeft className="h-4 w-4" />
                back to users list
              </Button>
            </div>

            {/* Windows Phone accent */}
            <div className="absolute bottom-2 right-2 w-1 h-1 bg-yellow-400 rounded-full opacity-60"></div>
          </div>
        </div>
      </div>
    </div>
  )
}