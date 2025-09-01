/**
 * Error boundary for user deletion page
 * Windows Phone + PancakeSwap styled error handling
 */

'use client'

import { Trash2, AlertTriangle, RefreshCw, ArrowLeft } from 'lucide-react'
import { useRouter } from 'next/navigation'

export default function DeleteUserError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  const router = useRouter()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-6">
      {/* Windows Phone style error tile */}
      <div className="bg-gradient-to-br from-red-600 to-red-800 text-white p-6 max-w-md w-full shadow-2xl relative overflow-hidden rounded-lg">
        {/* PancakeSwap corner accent */}
        <div className="absolute top-0 right-0 w-8 h-8 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
        
        {/* Error icon */}
        <div className="flex items-center gap-3 mb-6">
          <div className="w-12 h-12 bg-white/20 rounded-lg flex items-center justify-center">
            <AlertTriangle className="h-6 w-6 text-yellow-300" />
          </div>
          <div>
            <h2 className="text-xl font-extralight tracking-wide">deletion error</h2>
            <p className="text-red-200 text-sm font-light">unable to process request</p>
          </div>
        </div>

        {/* Error details */}
        <div className="mb-6 bg-black/20 p-4 rounded-lg">
          <p className="text-sm font-mono text-red-100 break-words">
            {error.message || 'Failed to load user deletion page'}
          </p>
          {error.digest && (
            <p className="text-xs text-red-300 mt-2 opacity-75">
              Error ID: {error.digest}
            </p>
          )}
        </div>

        {/* Possible causes */}
        <div className="mb-6 p-3 bg-white/10 rounded-lg">
          <p className="text-xs text-red-200 font-light mb-2">Possible causes:</p>
          <ul className="text-xs text-red-100 space-y-1 list-disc list-inside opacity-90">
            <li>User not found</li>
            <li>Insufficient permissions</li>
            <li>Network connectivity issues</li>
          </ul>
        </div>

        {/* Windows Phone style action buttons */}
        <div className="space-y-3">
          <button
            onClick={reset}
            className="w-full flex items-center justify-center gap-2 py-3 px-4 bg-white/20 hover:bg-white/30 rounded-lg transition-all font-light text-sm group"
          >
            <RefreshCw className="h-4 w-4 group-hover:rotate-180 transition-transform duration-500" />
            try again
          </button>
          
          <button
            onClick={() => router.push('/users')}
            className="w-full flex items-center justify-center gap-2 py-3 px-4 bg-yellow-400/20 hover:bg-yellow-400/30 rounded-lg transition-all font-light text-sm"
          >
            <ArrowLeft className="h-4 w-4" />
            return to users
          </button>
        </div>

        {/* Windows Phone accent */}
        <div className="absolute bottom-2 right-2 w-1 h-1 bg-yellow-400 rounded-full opacity-60"></div>
      </div>
    </div>
  )
}