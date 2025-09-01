/**
 * Loading UI for bulk operations page
 * Windows Phone + PancakeSwap styled loading state
 */

import { Users, Loader2, Settings } from 'lucide-react'

export default function BulkOperationsLoading() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-4xl mx-auto px-4 py-8">
        {/* Header skeleton */}
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-gradient-to-br from-blue-600 to-blue-800 rounded-full flex items-center justify-center">
              <Users className="h-6 w-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                Bulk Operations
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                Preparing bulk user management...
              </p>
            </div>
          </div>
        </div>

        {/* Loading bulk operations interface */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <div className="p-8">
            <div className="flex items-center justify-center py-16">
              <div className="text-center">
                {/* Windows Phone style loading indicator */}
                <div className="w-20 h-20 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-lg flex items-center justify-center mx-auto mb-6 relative overflow-hidden">
                  {/* PancakeSwap accent */}
                  <div className="absolute top-0 right-0 w-6 h-6 bg-gradient-to-bl from-white to-transparent opacity-30"></div>
                  <Loader2 className="h-10 w-10 text-black animate-spin" />
                </div>
                
                <h3 className="text-2xl font-extralight tracking-wide text-gray-900 dark:text-gray-100 mb-3">
                  loading bulk operations
                </h3>
                
                <p className="text-sm text-gray-600 dark:text-gray-400 font-light mb-8">
                  setting up user management tools
                </p>
                
                {/* Loading progress steps */}
                <div className="space-y-4 max-w-sm mx-auto">
                  {[
                    'Loading selected users',
                    'Preparing permissions interface',
                    'Setting up bulk actions',
                    'Initializing confirmation dialogs'
                  ].map((step, i) => (
                    <div key={i} className="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400">
                      <div 
                        className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"
                        style={{
                          animationDelay: `${i * 0.4}s`,
                          animationDuration: '2s'
                        }}
                      />
                      <span className="font-light text-left">{step}</span>
                    </div>
                  ))}
                </div>
                
                {/* Windows Phone accent dots */}
                <div className="flex items-center justify-center gap-2 mt-8">
                  {Array.from({ length: 4 }).map((_, i) => (
                    <div
                      key={i}
                      className="w-2 h-2 bg-yellow-400 rounded-full animate-bounce"
                      style={{
                        animationDelay: `${i * 0.1}s`,
                        animationDuration: '1s'
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