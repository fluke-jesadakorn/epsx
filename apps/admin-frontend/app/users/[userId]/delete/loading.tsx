/**
 * Loading UI for user deletion page
 * Windows Phone + PancakeSwap styled loading state
 */

import { Trash2, Loader2 } from 'lucide-react'

export default function DeleteUserLoading() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-md mx-auto px-4 py-8">
        {/* Loading deletion confirmation */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <div className="p-8 text-center">
            {/* Windows Phone style loading indicator */}
            <div className="w-16 h-16 bg-gradient-to-br from-red-600 to-red-800 rounded-lg flex items-center justify-center mx-auto mb-6 relative overflow-hidden">
              {/* PancakeSwap accent */}
              <div className="absolute top-0 right-0 w-4 h-4 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
              <Loader2 className="h-8 w-8 text-white animate-spin" />
            </div>
            
            <h2 className="text-2xl font-extralight tracking-wide text-gray-900 dark:text-gray-100 mb-2">
              loading
            </h2>
            
            <p className="text-sm text-gray-600 dark:text-gray-400 font-light mb-6">
              preparing user deletion
            </p>
            
            {/* Windows Phone progress animation */}
            <div className="space-y-3">
              {['Loading user data', 'Preparing confirmation', 'Security check'].map((text, i) => (
                <div key={i} className="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400">
                  <div 
                    className="w-2 h-2 bg-red-400 rounded-full animate-pulse"
                    style={{
                      animationDelay: `${i * 0.3}s`,
                      animationDuration: '2s'
                    }}
                  />
                  <span className="font-light">{text}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}