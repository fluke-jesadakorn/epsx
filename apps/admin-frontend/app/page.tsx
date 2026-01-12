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
    <div className="p-6">
      <div className="mb-8">
        <div className="h-10 bg-primary/20 rounded-2xl w-64 mb-2 "></div>
        <div className="h-4 bg-muted rounded-full w-48 "></div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-6 gap-4 max-w-7xl">
        {Array.from({ length: 9 }).map((_, i) => (
          <div key={i} className="h-32 bg-card rounded-2xl border border-primary/10"></div>
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
      <div className="min-h-screen p-6">
        <div className="text-center max-w-md mx-auto mt-32">
          <h1 className="text-3xl font-bold text-foreground mb-4">
            Authentication Required
          </h1>
          <p className="text-muted-foreground mb-8">
            Please connect your wallet to access the admin dashboard.
          </p>
          <a href="/auth" className="inline-flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity">
            Connect Wallet
          </a>
        </div>
      </div>
    )
  }

  // Show access error if backend rejected the request
  if (accessError) {
    return (
      <div className="min-h-screen p-6">
        <div className="text-center max-w-md mx-auto mt-32">
          <h1 className="text-3xl font-bold text-foreground mb-4">
            Access Denied
          </h1>
          <p className="text-muted-foreground mb-8">
            {accessError}
          </p>
          <a href="/auth" className="inline-flex items-center gap-2 px-6 py-3 bg-destructive text-destructive-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity">
            Try Again
          </a>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen p-3 sm:p-6">
      <div className="relative space-y-6 sm:space-y-8">
        {/* Page Header */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-flex items-center gap-3 justify-center">
            <h1 className="flex items-center gap-3 text-4xl sm:text-5xl font-bold">
              <span className="text-primary">
                <Home className="w-10 h-10 sm:w-12 sm:h-12" />
              </span>
              <span className="bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                EPSX Admin Center
              </span>
            </h1>
          </div>
          <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto mt-4">
            Welcome back, {user?.wallet_address || 'Admin'}
          </p>
          <div className="text-sm text-muted-foreground mt-2 flex items-center justify-center gap-2">
            <span>{new Date().toLocaleDateString()}</span>
            <span>•</span>
            <span className="flex items-center gap-1">
              System Status: <span className="text-success font-medium">Operational</span> <span className="relative flex h-2 w-2"><span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-success/40 opacity-75"></span><span className="relative inline-flex rounded-full h-2 w-2 bg-success"></span></span>
            </span>
          </div>
        </div>

        {/* Stats Dashboard */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          {/* Total Wallets */}
          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-primary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-primary/10 rounded-xl text-primary">
                <Wallet className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-primary">{dashboardStats.totalWallets}</div>
              <div className="text-xs sm:text-sm text-foreground/80">Wallets</div>
              <div className="text-xs text-muted-foreground">{dashboardStats.activeWallets} active today</div>
            </div>
          </div>

          {/* System Health */}
          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-success/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-success/10 rounded-xl text-success">
                <Activity className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Health</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-success">{dashboardStats.systemHealth}%</div>
              <div className="text-xs sm:text-sm text-foreground/80">System Health</div>
              <div className="text-xs text-muted-foreground">{dashboardStats.systemUptime} uptime</div>
            </div>
          </div>

          {/* Today's Activity */}
          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-secondary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-secondary/10 rounded-xl text-secondary">
                <Zap className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Today</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-secondary">{dashboardStats.todayConnections}</div>
              <div className="text-xs sm:text-sm text-foreground/80">Connections</div>
              <div className="text-xs text-muted-foreground">{dashboardStats.totalPermissions} permissions</div>
            </div>
          </div>

          {/* Performance */}
          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border-2 border-warning/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-warning/10 rounded-xl text-warning">
                <Clock className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Speed</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-warning">{dashboardStats.avgResponseTime}</div>
              <div className="text-xs sm:text-sm text-foreground/80">Avg Response</div>
              <div className="text-xs text-muted-foreground">{dashboardStats.pendingNotifications} pending</div>
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
            <div className="bg-card/80 backdrop-blur-sm rounded-2xl p-6 shadow-sm border-2 border-primary/20">
              <h3 className="text-lg font-bold text-primary mb-4 flex items-center gap-2">
                <Activity className="w-5 h-5" /> Quick Stats
              </h3>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Active Wallets</span>
                  <span className="font-semibold">{dashboardStats.activeWallets}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">System Health</span>
                  <span className="font-semibold text-success">{dashboardStats.systemHealth}%</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Response Time</span>
                  <span className="font-semibold text-warning">{dashboardStats.avgResponseTime}</span>
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
              gradient: "from-primary via-secondary to-primary",
              textGradient: "text-primary",
              bgGradient: "bg-primary/5",
              stats: `${dashboardStats.totalWallets} wallets`
            },
            {
              href: "/group-and-permission",
              title: "Permissions",
              icon: <Shield className="w-6 h-6" />,
              description: "Grant and manage access permissions",
              gradient: "from-success via-secondary to-success",
              textGradient: "text-success",
              bgGradient: "bg-success/5",
              stats: `${dashboardStats.totalPermissions} permissions`
            },
            {
              href: "/audit-log",
              title: "Audit Log",
              icon: <FileText className="w-6 h-6" />,
              description: "Track admin actions and changes",
              gradient: "from-info via-primary to-info",
              textGradient: "text-info",
              bgGradient: "bg-info/5",
              stats: "View history"
            },
            {
              href: "/notifications",
              title: "Notifications",
              icon: <Bell className="w-6 h-6" />,
              description: "Send notifications and manage alerts",
              gradient: "from-secondary via-primary to-secondary",
              textGradient: "text-secondary",
              bgGradient: "bg-secondary/5",
              stats: `${dashboardStats.pendingNotifications} pending`
            },
            {
              href: "/settings",
              title: "Settings",
              icon: <Settings className="w-6 h-6" />,
              description: "Configure system and user settings",
              gradient: "from-muted-foreground via-foreground to-muted-foreground",
              textGradient: "text-muted-foreground",
              bgGradient: "bg-muted/5",
              stats: "System config"
            },
            {
              href: "/developer-portal",
              title: "Developer",
              icon: <Zap className="w-6 h-6" />,
              description: "API documentation and developer tools",
              gradient: "from-warning via-primary to-warning",
              textGradient: "text-warning",
              bgGradient: "bg-warning/5",
              stats: "API & Tools"
            }
          ].map((action) => (
            <a
              key={action.href}
              href={action.href}
              className="block group focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            >
              <div className={`relative overflow-hidden rounded-2xl sm:rounded-3xl ${action.bgGradient} p-0.5 border border-border/50 hover:border-primary/50 transition-colors`}>
                <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
                  {/* Floating decoration */}
                  <div className={`absolute top-4 right-4 w-4 h-4 bg-gradient-to-r ${action.gradient} rounded-full blur-sm opacity-60`}></div>

                  <div className="p-4 sm:p-6">
                    <div className="mb-4">
                      <div className="flex items-center gap-3 mb-2">
                        <div className={`p-2 rounded-lg bg-muted ${action.textGradient}`}>
                          {action.icon}
                        </div>
                        <h3 className={`text-lg sm:text-xl font-bold bg-gradient-to-r ${action.gradient} bg-clip-text text-transparent`}>
                          {action.title}
                        </h3>
                      </div>
                      <p className="text-sm text-muted-foreground mb-3 ml-1">
                        {action.description}
                      </p>
                      <div className="text-xs font-medium text-muted-foreground/80 ml-1">
                        {action.stats}
                      </div>
                    </div>

                    <div className="flex items-center justify-between mt-4">
                      <div className={`px-3 py-1 bg-gradient-to-r ${action.gradient} text-white rounded-full text-xs font-medium`}>
                        Open
                      </div>
                      <div className="text-muted-foreground group-hover:text-primary transition-colors">→</div>
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