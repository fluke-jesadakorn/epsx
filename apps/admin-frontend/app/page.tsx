import { Shield, Users, Activity, Database, TrendingUp, Settings } from 'lucide-react'
import Link from 'next/link'
import { Suspense } from 'react'
import { StatsCard } from '@/components/ui/StatsCard'
import { DashboardSkeleton } from '@/components/admin/DashboardSkeleton'
import { getDashboardStats, getSystemMetrics } from '@/lib/data/dashboard'

// This page uses authentication/cookies and should be dynamic
export const dynamic = 'force-dynamic'

async function DashboardContent() {
  // Fetch real data from backend
  const [dashboardStats, systemMetrics] = await Promise.allSettled([
    getDashboardStats(),
    getSystemMetrics()
  ])

  const stats = dashboardStats.status === 'fulfilled' ? dashboardStats.value : null
  const metrics = systemMetrics.status === 'fulfilled' ? systemMetrics.value : null

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
      <div className="flex items-center gap-4 mb-6">
        <div className="p-3 rounded-full bg-gradient-to-r from-blue-500 to-purple-500">
          <Shield className="h-8 w-8 text-white" />
        </div>
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Admin Dashboard
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Welcome to the EPSX Admin Panel
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatsCard
          title="Total Users"
          value={stats?.totalUsers?.toLocaleString() || '0'}
          description={`${stats?.verifiedUsers || 0} verified`}
          icon={Users}
          gradient="from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20"
          textColor="text-blue-600 dark:text-blue-400"
          className="bg-gradient-to-r from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20"
        />
        
        <StatsCard
          title="Active Sessions"
          value={stats?.totalSessions?.toLocaleString() || '0'}
          description={`${stats?.activeUsers || 0} users online`}
          icon={Activity}
          gradient="from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20"
          textColor="text-green-600 dark:text-green-400"
          className="bg-gradient-to-r from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20"
        />
        
        <StatsCard
          title="System Health"
          value={stats?.systemHealth === 'good' ? 'Good' : stats?.systemHealth === 'warning' ? 'Warning' : 'Critical'}
          description={`${metrics?.uptime?.toFixed(1) || '99.9'}% uptime`}
          icon={Database}
          gradient="from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20"
          textColor="text-purple-600 dark:text-purple-400"
          className="bg-gradient-to-r from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20"
        />
        
        <StatsCard
          title="Performance"
          value={`${metrics?.errorRate ? (100 - metrics.errorRate).toFixed(1) : '99.8'}%`}
          description={`${metrics?.serverLoad || 45}% CPU usage`}
          icon={TrendingUp}
          gradient="from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20"
          textColor="text-orange-600 dark:text-orange-400"
          className="bg-gradient-to-r from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20"
        />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-gray-50 dark:bg-gray-700/50 p-6 rounded-lg">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Actions</h3>
          <div className="space-y-3">
            <Link href="/users" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white dark:hover:bg-gray-700 transition-colors">
              <Users className="h-5 w-5 text-blue-500" />
              <span className="text-gray-900 dark:text-white">Manage Users</span>
            </Link>
            <Link href="/permissions" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white dark:hover:bg-gray-700 transition-colors">
              <Shield className="h-5 w-5 text-blue-500" />
              <span className="text-gray-900 dark:text-white">Manage Permissions</span>
            </Link>
            <Link href="/analytics" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white dark:hover:bg-gray-700 transition-colors">
              <TrendingUp className="h-5 w-5 text-green-500" />
              <span className="text-gray-900 dark:text-white">View Analytics</span>
            </Link>
            <Link href="/settings" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white dark:hover:bg-gray-700 transition-colors">
              <Settings className="h-5 w-5 text-purple-500" />
              <span className="text-gray-900 dark:text-white">System Settings</span>
            </Link>
          </div>
        </div>

        <div className="bg-gray-50 dark:bg-gray-700/50 p-6 rounded-lg">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">System Status</h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">API Server</span>
              <span className={`flex items-center gap-2 ${stats ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>
                <div className={`w-2 h-2 ${stats ? 'bg-green-500' : 'bg-red-500'} rounded-full`}></div>
                {stats ? 'Online' : 'Offline'}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Database</span>
              <span className={`flex items-center gap-2 ${metrics?.databaseConnections ? 'text-green-600 dark:text-green-400' : 'text-orange-600 dark:text-orange-400'}`}>
                <div className={`w-2 h-2 ${metrics?.databaseConnections ? 'bg-green-500' : 'bg-orange-500'} rounded-full`}></div>
                {metrics?.databaseConnections ? `Connected (${metrics.databaseConnections})` : 'Limited'}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-600 dark:text-gray-400">Memory Usage</span>
              <span className={`flex items-center gap-2 ${(metrics?.memoryUsage || 0) < 80 ? 'text-green-600 dark:text-green-400' : 'text-orange-600 dark:text-orange-400'}`}>
                <div className={`w-2 h-2 ${(metrics?.memoryUsage || 0) < 80 ? 'bg-green-500' : 'bg-orange-500'} rounded-full`}></div>
                {metrics?.memoryUsage ? `${metrics.memoryUsage.toFixed(1)}%` : 'N/A'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <Suspense fallback={<DashboardSkeleton />}>
        <DashboardContent />
      </Suspense>
    </div>
  )
}