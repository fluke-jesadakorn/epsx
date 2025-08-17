/**
 * Global error UI for the admin frontend
 * Shows when pages fail to load
 */

'use client'

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  return (
    <div className="min-h-[400px] flex items-center justify-center p-6">
      <div className="p-8 max-w-md w-full text-center border rounded-lg">
        <h2 className="text-xl font-semibold mb-2">Something went wrong!</h2>
        
        <p className="text-gray-600 mb-6">
          Error: {error.message}
        </p>
        
        <button
          onClick={reset}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Try again
        </button>
      </div>
    </div>
  )
}