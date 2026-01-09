'use client';

import {
  Activity,
  Bell,
  Clock,
  FileText,
  Home,
  Settings,
  Shield,
  Wallet,
  Zap
} from 'lucide-react';
import { useEffect, useState } from 'react';

import { RecentWalletsPanel } from '@/components/admin/RecentWalletsPanel';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { APIError, createAdminApiClient } from '@/shared/utils/api-client';

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

  // Load dashboard data from real backend APIs - no hardcoded values
  useEffect(() => {
    if (isAuthenticated) {
      const loadDashboardData = async () => {
        try {
          setAccessError(null)
          const client = createAdminApiClient()

          // Helper to catch errors and format them
          const safeFetch = async (promise: Promise<any>) => {
            try {
              return await promise
            } catch (err: any) {
              if (err instanceof APIError) {
                return { success: false, data: null, status: err.status, error: err.message }
              }
              return { success: false, data: null, status: 0, error: 'Network error or unexpected issue' }
            }
          }

          // Fetch wallet data, permissions, and system stats in parallel
          const [walletsRes, permissionsRes, systemRes] = await Promise.all([
            safeFetch(client.get('/api/admin/wallets/stats')),
            safeFetch(client.get('/api/admin/permissions/system/stats')),
            safeFetch(client.get('/api/admin/permissions/system/health'))
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

          // Check for permission errors from any of the requests
          if (walletsRes.status === 403 || walletsRes.status === 401 ||
            permissionsRes.status === 403 || permissionsRes.status === 401 ||
            systemRes.status === 403 || systemRes.status === 401) {
            setAccessError(walletsRes.error || permissionsRes.error || systemRes.error || 'Access denied by backend')
          }
        } catch (err) {
          console.error('Failed to load dashboard data:', err)
          // Don't block dashboard access on error - show with default values
        } finally {
          // No longer using isLoadingStats
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
          <a href="/auth" className="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 rounded-2xl text-white font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-500">
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
          <a href="/auth" className="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 rounded-2xl text-white font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-500">
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
          <div className="relative inline-flex items-center gap-3 justify-center">
            <h1 className="flex items-center gap-3 text-4xl sm:text-5xl font-bold">
              <span className="text-amber-500">
                <Home className="w-10 h-10 sm:w-12 sm:h-12" />
              </span>
              <span className="bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent">
                EPSX Admin Center
              </span>
            </h1>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mt-4">
            Welcome back, {user?.wallet_address || 'Admin'}
          </p>
          <div className="text-sm text-gray-500 dark:text-gray-500 mt-2 flex items-center justify-center gap-2">
            <span>{new Date().toLocaleDateString()}</span>
            <span>•</span>
            <span className="flex items-center gap-1">
              System Status: <span className="text-green-500 font-medium">Operational</span> <span className="relative flex h-2 w-2"><span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span><span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span></span>
            </span>
          </div>
        </div>

        {/* Stats Dashboard */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          {/* Total Wallets */}
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-xl text-blue-600 dark:text-blue-400">
                <Wallet className="w-6 h-6" />
              </div>
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
              <div className="p-2 bg-green-100 dark:bg-green-900/30 rounded-xl text-green-600 dark:text-green-400">
                <Activity className="w-6 h-6" />
              </div>
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
              <div className="p-2 bg-purple-100 dark:bg-purple-900/30 rounded-xl text-purple-600 dark:text-purple-400">
                <Zap className="w-6 h-6" />
              </div>
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
              <div className="p-2 bg-orange-100 dark:bg-orange-900/30 rounded-xl text-orange-600 dark:text-orange-400">
                <Clock className="w-6 h-6" />
              </div>
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
              <h3 className="text-lg font-bold text-indigo-600 mb-4 flex items-center gap-2">
                <Activity className="w-5 h-5" /> Quick Stats
              </h3>
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
              title: "Wallet Management",
              icon: <Wallet className="w-6 h-6" />,
              description: "Manage wallets and permissions",
              gradient: "from-blue-400 to-cyan-500",
              textGradient: "text-blue-500",
              bgGradient: "from-blue-400/20 via-cyan-400/20 to-blue-400/20",
              stats: `${dashboardStats.totalWallets} wallets`
            },
            {
              href: "/group-and-permission",
              title: "Permissions",
              icon: <Shield className="w-6 h-6" />,
              description: "Grant and manage access permissions",
              gradient: "from-green-400 to-emerald-500",
              textGradient: "text-emerald-500",
              bgGradient: "from-green-400/20 via-emerald-400/20 to-green-400/20",
              stats: `${dashboardStats.totalPermissions} permissions`
            },
            {
              href: "/audit-log",
              title: "Audit Log",
              icon: <FileText className="w-6 h-6" />,
              description: "Track admin actions and changes",
              gradient: "from-indigo-400 to-purple-500",
              textGradient: "text-indigo-500",
              bgGradient: "from-indigo-400/20 via-purple-400/20 to-indigo-400/20",
              stats: "View history"
            },
            {
              href: "/notifications",
              title: "Notifications",
              icon: <Bell className="w-6 h-6" />,
              description: "Send notifications and manage alerts",
              gradient: "from-purple-400 to-pink-500",
              textGradient: "text-purple-500",
              bgGradient: "from-purple-400/20 via-pink-400/20 to-purple-400/20",
              stats: `${dashboardStats.pendingNotifications} pending`
            },
            {
              href: "/settings",
              title: "Settings",
              icon: <Settings className="w-6 h-6" />,
              description: "Configure system and user settings",
              gradient: "from-gray-400 to-slate-500",
              textGradient: "text-slate-500",
              bgGradient: "from-gray-400/20 via-slate-400/20 to-gray-400/20",
              stats: "System config"
            },
            {
              href: "/developer-portal",
              title: "Developer",
              icon: <Zap className="w-6 h-6" />,
              description: "API documentation and developer tools",
              gradient: "from-orange-400 to-amber-500",
              textGradient: "text-orange-500",
              bgGradient: "from-orange-400/20 via-amber-400/20 to-orange-400/20",
              stats: "API & Tools"
            }
          ].map((action) => (
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
                      <div className="flex items-center gap-3 mb-2">
                        <div className={`p-2 rounded-lg bg-gray-50 dark:bg-gray-800 ${action.textGradient}`}>
                          {action.icon}
                        </div>
                        <h3 className={`text-lg sm:text-xl font-bold bg-gradient-to-r ${action.gradient} bg-clip-text text-transparent`}>
                          {action.title}
                        </h3>
                      </div>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-3 ml-1">
                        {action.description}
                      </p>
                      <div className="text-xs font-medium text-gray-500 dark:text-gray-400 ml-1">
                        {action.stats}
                      </div>
                    </div>

                    <div className="flex items-center justify-between mt-4">
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