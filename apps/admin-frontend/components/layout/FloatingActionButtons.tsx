'use client'

import { useRouter } from 'next/navigation'
import { useState } from 'react'

/**
 *
 */
export function FloatingActionButtons() {
  const router = useRouter()
  const [isOpen, setIsOpen] = useState(false)
  const [showQuickStats, setShowQuickStats] = useState(false)

  const quickActions = [
    {
      id: 'add-user',
      label: 'Add User',
      icon: '👤',
      gradient: 'from-blue-400 to-purple-500',
      action: () => router.push('/users/create')
    },
    {
      id: 'grant-permission',
      label: 'Grant Permission',
      icon: '🔑',
      gradient: 'from-green-400 to-teal-500',
      action: () => router.push('/permissions/grant')
    },
    {
      id: 'send-notification',
      label: 'Send Notification',
      icon: '📢',
      gradient: 'from-orange-400 to-red-500',
      action: () => router.push('/notifications/create')
    },
    {
      id: 'view-analytics',
      label: 'View Analytics',
      icon: '📊',
      gradient: 'from-purple-400 to-pink-500',
      action: () => router.push('/analytics')
    }
  ]

  return (
    <>
      {/* Main FAB */}
      <div className="fixed bottom-8 right-8 z-50">
        <div className="relative">
          {/* Quick Actions Menu */}
          {isOpen && (
            <div className="absolute bottom-16 right-0 space-y-3 mb-4">
              {quickActions.map((action, index) => (
                <div
                  key={action.id}
                  className="flex items-center gap-3 animate-fade-in-up"
                  style={{ animationDelay: `${index * 50}ms` }}
                >
                  <div className="bg-white dark:bg-gray-800 px-3 py-2 rounded-2xl shadow-lg border border-yellow-200 dark:border-purple-500/30 backdrop-blur-sm max-w-[150px]">
                    <span className="text-xs sm:text-sm font-semibold text-gray-700 dark:text-gray-300 whitespace-nowrap truncate block">
                      {action.label}
                    </span>
                  </div>
                  <button
                    onClick={action.action}
                    className={`h-14 w-14 bg-gradient-to-r ${action.gradient} text-white rounded-2xl shadow-xl hover:shadow-2xl hover:scale-110 transition-all duration-200 flex items-center justify-center text-xl`}
                  >
                    {action.icon}
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Main Button */}
          <button
            onClick={() => setIsOpen(!isOpen)}
            className={`h-16 w-16 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 text-white rounded-3xl shadow-2xl hover:shadow-3xl hover:scale-110 transition-all duration-300 flex items-center justify-center text-2xl relative overflow-hidden group ${
              isOpen ? 'rotate-45' : ''
            }`}
          >
            {/* Background Animation */}
            <div className="absolute inset-0 bg-gradient-to-r from-pink-500 via-purple-500 to-blue-500 opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>
            
            {/* Icon */}
            <span className="relative z-10 transition-transform duration-200">
              {isOpen ? '✕' : '🚀'}
            </span>

            {/* Pulsing Ring */}
            <div className="absolute inset-0 rounded-3xl border-4 border-yellow-400/30 animate-ping"></div>
          </button>
        </div>
      </div>

      {/* Quick Stats Floating Panel */}
      <div className={`fixed bottom-8 left-4 sm:left-8 z-40 transition-all duration-300 ${showQuickStats ? 'translate-y-0 opacity-100' : 'translate-y-4 opacity-0'}`}>
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-yellow-200 dark:border-purple-500/30 p-4 sm:p-6 w-[260px] sm:min-w-[280px] max-w-[calc(100vw-2rem)]">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-bold text-base sm:text-lg bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-transparent truncate">
              🎯 Quick Stats
            </h3>
            <button
              onClick={() => setShowQuickStats(false)}
              className="text-gray-400 hover:text-gray-600 transition-colors flex-shrink-0 ml-2"
            >
              ✕
            </button>
          </div>
          
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 bg-gradient-to-r from-green-50 to-teal-50 dark:from-green-900/20 dark:to-teal-900/20 rounded-2xl">
              <div className="flex items-center gap-2">
                <span className="text-green-500">👥</span>
                <span className="text-sm font-medium">Active Users</span>
              </div>
              <span className="font-bold text-green-600">1,247</span>
            </div>

            <div className="flex items-center justify-between p-3 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 rounded-2xl">
              <div className="flex items-center gap-2">
                <span className="text-blue-500">🔑</span>
                <span className="text-sm font-medium">Permissions</span>
              </div>
              <span className="font-bold text-blue-600">3,891</span>
            </div>

            <div className="flex items-center justify-between p-3 bg-gradient-to-r from-orange-50 to-red-50 dark:from-orange-900/20 dark:to-red-900/20 rounded-2xl">
              <div className="flex items-center gap-2">
                <span className="text-orange-500">⏳</span>
                <span className="text-sm font-medium">Pending</span>
              </div>
              <span className="font-bold text-orange-600">23</span>
            </div>
          </div>

          <button className="w-full mt-4 bg-gradient-to-r from-yellow-400 to-orange-500 text-white py-2 rounded-2xl font-semibold hover:from-yellow-500 hover:to-orange-600 transition-all duration-200 text-sm">
            View Full Dashboard
          </button>
        </div>
      </div>

      {/* Stats Toggle Button */}
      <button
        onClick={() => setShowQuickStats(!showQuickStats)}
        className="fixed bottom-28 right-8 z-40 h-12 w-12 bg-gradient-to-r from-purple-400 to-pink-500 text-white rounded-2xl shadow-xl hover:shadow-2xl hover:scale-110 transition-all duration-200 flex items-center justify-center"
      >
        📊
      </button>

      {/* Help Button */}
      <button className="fixed bottom-44 right-8 z-40 h-12 w-12 bg-gradient-to-r from-blue-400 to-teal-500 text-white rounded-2xl shadow-xl hover:shadow-2xl hover:scale-110 transition-all duration-200 flex items-center justify-center">
        ❓
      </button>
    </>
  )
}

// Add these animations to your globals.css
const animationCSS = `
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.3s ease-out forwards;
}
`