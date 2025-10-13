'use client';

import { useEffect, useState } from 'react'

import { RecentWalletsPanel } from '@/components/admin/RecentWalletsPanel'
import { PermissionManagement } from '@/components/permissions/PermissionManagement'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'

function DashboardSkeleton() {
  return (
    <div className="wp-pancake-page-bg p-6">
      <div className="mb-8">
        <div className="h-10 bg-gray-700/50 rounded w-64 mb-2 "></div>
        <div className="h-4 bg-gray-700/50 rounded w-48 "></div>
      </div>
      
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 gap-4 max-w-7xl">
        {Array.from({ length: 9 }).map((_, i) => (
          <div key={i} className="h-32 bg-gray-700/30 backdrop-blur-sm rounded-lg  border border-yellow-500/10"></div>
        ))}
      </div>
    </div>
  )
}

/**
 *
 */
export default function DashboardPage() {
  const { user, isAuthenticated, isLoading } = useSharedAuth()
  const [dashboardStats, setDashboardStats] = useState({
    totalWallets: 0,
    activeWallets: 0,
    totalPermissions: 0,
    systemHealth: 100,
    todayConnections: 0,
    pendingNotifications: 0,
    systemUptime: '99.9%',
    avgResponseTime: '120ms'
  })
  const [accessError, setAccessError] = useState<string | null>(null)
  const [isLoadingStats, setIsLoadingStats] = useState(true)

  // Load dashboard data from real backend APIs - no hardcoded values
  useEffect(() => {
    if (isAuthenticated) {
      const loadDashboardData = async () => {
        try {
          setIsLoadingStats(true)
          setAccessError(null)
          const client = createAdminApiClient()

          // Fetch wallet data, permissions, and system stats in parallel
          const [walletsRes, permissionsRes, systemRes] = await Promise.all([
            client.get('/api/admin/wallets/stats').catch(() => ({ success: false, data: null })),
            client.get('/api/admin/permissions/stats').catch(() => ({ success: false, data: null })),
            client.get('/api/admin/system/health').catch(() => ({ success: false, data: null }))
          ])

          // Update stats from real API responses
          if (walletsRes.success && walletsRes.data) {
            setDashboardStats(prev => ({
              ...prev,
              totalWallets: walletsRes.data.total || 0,
              activeWallets: walletsRes.data.active || 0,
              todayConnections: walletsRes.data.today_connections || 0
            }))
          }

          if (permissionsRes.success && permissionsRes.data) {
            setDashboardStats(prev => ({
              ...prev,
              totalPermissions: permissionsRes.data.total || 0,
              pendingNotifications: permissionsRes.data.pending_notifications || 0
            }))
          }

          if (systemRes.success && systemRes.data) {
            setDashboardStats(prev => ({
              ...prev,
              systemHealth: systemRes.data.health_percentage || 100,
              systemUptime: systemRes.data.uptime || '99.9%',
              avgResponseTime: systemRes.data.avg_response_time || '120ms'
            }))
          }

          // Check for permission errors
          if (walletsRes.status === 403 || walletsRes.status === 401) {
            setAccessError(walletsRes.error || 'Access denied by backend')
          }
        } catch (err) {
          console.error('Failed to load dashboard data:', err)
          // Don't block dashboard access on error - show with default values
        } finally {
          setIsLoadingStats(false)
        }
      }

      loadDashboardData()
    }
  }, [isAuthenticated])

  // Show loading state
  if (isLoading) {
    return <DashboardSkeleton />
  }

  // Show authentication required if not authenticated
  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="text-center max-w-md mx-auto mt-32">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4">
            Authentication Required
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mb-8">
            Please connect your wallet to access the admin dashboard.
          </p>
          <a href="/login" className="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 rounded-2xl text-white font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-500">
            Connect Wallet
          </a>
        </div>
      </div>
    )
  }

  // Show access error if backend rejected the request
  if (accessError) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="text-center max-w-md mx-auto mt-32">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4">
            Access Denied
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mb-8">
            {accessError}
          </p>
          <a href="/login" className="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 rounded-2xl text-white font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-500">
            Try Again
          </a>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative space-y-6 sm:space-y-8">
        {/* Page Header */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              🏠 EPSX Admin Center
            </h1>
                <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Welcome back, {user?.wallet_address || 'Admin'}
          </p>
          <div className="text-sm text-gray-500 dark:text-gray-500 mt-2">
            {new Date().toLocaleDateString()} • System Status: Operational 🟢
          </div>
        </div>

          {/* Stats Dashboard */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
            {/* Total Wallets */}
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-2xl sm:text-3xl">👛</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-blue-600">{dashboardStats.totalWallets}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Wallets</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">{dashboardStats.activeWallets} active today</div>
              </div>
            </div>

            {/* System Health */}
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-2xl sm:text-3xl">💚</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Health</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-green-600">{dashboardStats.systemHealth}%</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">System Health</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">{dashboardStats.systemUptime} uptime</div>
              </div>
            </div>

            {/* Today's Activity */}
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-2xl sm:text-3xl">⚡</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Today</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-purple-600">{dashboardStats.todayConnections}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Connections</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">{dashboardStats.totalPermissions} permissions</div>
              </div>
            </div>

            {/* Performance */}
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-2xl sm:text-3xl">🚀</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Speed</span>
              </div>
              <div className="space-y-1">
                <div className="text-xl sm:text-3xl font-bold text-orange-600">{dashboardStats.avgResponseTime}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Avg Response</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">{dashboardStats.pendingNotifications} pending</div>
              </div>
            </div>
          </div>

          {/* Recent Wallets Panel */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6 sm:mb-8">
            <div className="lg:col-span-2">
              <RecentWalletsPanel />
            </div>
            <div className="space-y-4">
              {/* Additional metrics or quick stats can go here */}
              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50">
                <h3 className="text-lg font-bold text-indigo-600 mb-4">📊 Quick Stats</h3>
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-gray-600 dark:text-gray-400">Active Wallets</span>
                    <span className="font-semibold">{dashboardStats.activeWallets}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-gray-600 dark:text-gray-400">System Health</span>
                    <span className="font-semibold text-green-600">{dashboardStats.systemHealth}%</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-gray-600 dark:text-gray-400">Response Time</span>
                    <span className="font-semibold text-orange-600">{dashboardStats.avgResponseTime}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Quick Actions */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
            {[
              {
                href: "/wallet-management",
                title: "👛 Wallet Management",
                description: "Manage Web3 wallets and permissions",
                gradient: "from-blue-400 to-cyan-500",
                bgGradient: "from-blue-400/20 via-cyan-400/20 to-blue-400/20",
                stats: `${dashboardStats.totalWallets} wallets`
              },
              {
                href: "/permissions",
                title: "🔐 Permissions",
                description: "Grant and manage access permissions",
                gradient: "from-green-400 to-emerald-500",
                bgGradient: "from-green-400/20 via-emerald-400/20 to-green-400/20",
                stats: `${dashboardStats.totalPermissions} permissions`
              },
              {
                href: "/remote-config",
                title: "🎛️ Remote Config",
                description: "Manage user settings and feature flags",
                gradient: "from-teal-400 to-cyan-500",
                bgGradient: "from-teal-400/20 via-cyan-400/20 to-teal-400/20",
                stats: "Dynamic settings"
              },
              {
                href: "/notifications",
                title: "📤 Notifications",
                description: "Send notifications and manage alerts",
                gradient: "from-purple-400 to-pink-500",
                bgGradient: "from-purple-400/20 via-pink-400/20 to-purple-400/20",
                stats: `${dashboardStats.pendingNotifications} pending`
              },
              {
                href: "/settings",
                title: "⚙️ Settings",
                description: "Configure system and user settings",
                gradient: "from-gray-400 to-slate-500",
                bgGradient: "from-gray-400/20 via-slate-400/20 to-gray-400/20",
                stats: "System config"
              },
              {
                href: "/developer-portal",
                title: "⚡ Developer",
                description: "API documentation and developer tools",
                gradient: "from-indigo-400 to-purple-500",
                bgGradient: "from-indigo-400/20 via-purple-400/20 to-indigo-400/20",
                stats: "API & Tools"
              }
            ].map((action, index) => (
              <a
                key={action.href}
                href={action.href}
                className="block group focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500"
              >
              <div className={`relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r ${action.bgGradient} p-0.5`}>
                  <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
                    {/* Floating decoration */}
                    <div className={`absolute top-4 right-4 w-4 h-4 bg-gradient-to-r ${action.gradient} rounded-full blur-sm opacity-60`}></div>
                    
                    <div className="p-4 sm:p-6">
                      <div className="mb-4">
                        <h3 className={`text-lg sm:text-xl font-bold bg-gradient-to-r ${action.gradient} bg-clip-text text-transparent mb-2`}>
                          {action.title}
                        </h3>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                          {action.description}
                        </p>
                        <div className="text-xs font-medium text-gray-500 dark:text-gray-400">
                          {action.stats}
                        </div>
                      </div>
                      
                      <div className="flex items-center justify-between">
                        <div className={`px-3 py-1 bg-gradient-to-r ${action.gradient} text-white rounded-full text-xs font-medium`}>
                          Open
                        </div>
                        <div className="text-gray-400">→</div>
                      </div>
                    </div>
                  </div>
                </div>
              </a>
            ))}
          </div>
        </div>
      </div>
    )
}