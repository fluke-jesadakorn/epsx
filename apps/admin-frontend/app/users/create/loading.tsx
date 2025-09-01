/**
 * Loading UI for user creation page
 * Windows Phone + PancakeSwap styled loading state
 */

import { User, Loader2 } from 'lucide-react'

export default function CreateUserLoading() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-2xl mx-auto px-4 py-8">
        {/* Header skeleton */}
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-gradient-to-br from-green-500 to-green-700 rounded-full flex items-center justify-center">
              <User className="h-6 w-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                Create New User
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                Setting up the form...
              </p>
            </div>
          </div>
        </div>

        {/* Loading form skeleton */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <div className="p-6 space-y-6">
            <div className="flex items-center justify-center py-12">
              <div className="text-center">
                {/* Windows Phone style loading indicator */}
                <div className="w-16 h-16 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-lg flex items-center justify-center mb-4">
                  <Loader2 className="h-8 w-8 text-black animate-spin" />
                </div>
                
                <h3 className="text-lg font-extralight tracking-wide text-gray-900 dark:text-gray-100 mb-2">
                  loading
                </h3>
                
                <p className="text-sm text-gray-600 dark:text-gray-400 font-light">
                  preparing user creation form
                </p>
                
                {/* Windows Phone progress dots */}
                <div className="flex items-center justify-center gap-1 mt-4">
                  {Array.from({ length: 3 }).map((_, i) => (
                    <div
                      key={i}
                      className="w-2 h-2 bg-yellow-400 rounded-full animate-pulse"
                      style={{
                        animationDelay: `${i * 0.2}s`,
                        animationDuration: '1.5s'
                      }}
                    />
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}