/**
 * Global error UI for the admin frontend
 * Windows Phone + PancakeSwap styled error boundary
 */

'use client'

import { AlertTriangle, ArrowLeft, Home, RefreshCw } from 'lucide-react'
import { useRouter } from 'next/navigation'

/**
 *
 * @param root0
 * @param root0.error
 * @param root0.reset
 */
export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  const router = useRouter()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black flex items-center justify-center p-6">
      {/* Windows Phone style error tile */}
      <div className="bg-background text-foreground p-8 max-w-lg w-full shadow-2xl relative overflow-hidden">
        {/* PancakeSwap corner accent */}
        <div className="absolute top-0 right-0 w-8 h-8 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>

        {/* Error icon */}
        <div className="flex items-center gap-3 mb-6">
          <div className="w-12 h-12 bg-white/20 rounded-lg flex items-center justify-center">
            <AlertTriangle className="h-6 w-6 text-yellow-300" />
          </div>
          <div>
            <h2 className="text-2xl font-extralight tracking-wide">error</h2>
            <p className="text-red-200 text-sm font-light">something went wrong</p>
          </div>
        </div>

        {/* Error message */}
        <div className="mb-8 bg-black/20 p-4 rounded-lg">
          <p className="text-sm font-mono text-red-100 break-words">
            {error.message}
          </p>
          {error.digest && (
            <p className="text-xs text-red-300 mt-2 opacity-75">
              Error ID: {error.digest}
            </p>
          )}
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
            <Home className="h-4 w-4" />
            back to users
          </button>

          <button
            onClick={() => router.back()}
            className="w-full flex items-center justify-center gap-2 py-2 px-4 text-white/70 hover:text-white transition-colors font-light text-sm"
          >
            <ArrowLeft className="h-4 w-4" />
            go back
          </button>
        </div>

        {/* Windows Phone accent */}
        <div className="absolute bottom-2 right-2 w-1 h-1 bg-yellow-400 rounded-full opacity-60"></div>
      </div>
    </div>
  )
}