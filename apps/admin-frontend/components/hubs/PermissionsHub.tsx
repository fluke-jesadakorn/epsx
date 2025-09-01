import React from 'react'
import { Clock, Shield, Users, TrendingUp, Plus, RefreshCw, AlertTriangle } from 'lucide-react'
import { getUsersList } from '@/lib/actions/users'
import PermissionActions from '@/components/ui/PermissionActions'
import { CleanupButton } from './CleanupButton'

/**
 * Windows Phone-style Permissions Hub
 * Advanced permission management with embedded timestamps
 */

interface PermissionRowProps {
  userEmail: string
  permission: string
  expiresAt?: number
  onExtend: () => void
  onRevoke: () => void
}

function PermissionRow({ userEmail, permission, expiresAt, onExtend, onRevoke }: PermissionRowProps) {
  const isEmbedded = permission.includes(':') && /:\d+$/.test(permission)
  const [basePerm, timestamp] = isEmbedded ? permission.split(/(:)(\d+)$/).filter(Boolean) : [permission, '']
  
  let status = 'never'
  let statusColor = 'text-green-600'
  let timeLeft = ''
  
  if (expiresAt || timestamp) {
    const expiry = expiresAt || (timestamp ? parseInt(timestamp) * 1000 : 0)
    const now = Date.now()
    const diff = expiry - now
    
    if (diff <= 0) {
      status = 'expired'
      statusColor = 'text-red-600'
      timeLeft = '🔴 Expired'
    } else if (diff < 2 * 60 * 60 * 1000) { // < 2 hours
      status = 'critical'
      statusColor = 'text-red-600'
      timeLeft = `🔴 ${Math.round(diff / (60 * 1000))}min left`
    } else if (diff < 24 * 60 * 60 * 1000) { // < 24 hours
      status = 'warning'
      statusColor = 'text-orange-600'
      timeLeft = `🟡 ${Math.round(diff / (60 * 60 * 1000))}h left`
    } else if (diff < 7 * 24 * 60 * 60 * 1000) { // < 7 days
      status = 'caution'
      statusColor = 'text-yellow-600'
      timeLeft = `🟡 ${Math.round(diff / (24 * 60 * 60 * 1000))} days left`
    } else {
      status = 'active'
      statusColor = 'text-green-600'
      timeLeft = `🟢 ${Math.round(diff / (24 * 60 * 60 * 1000))} days left`
    }
  } else {
    timeLeft = '♾️ Never expires'
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 transition-colors">
      {/* User */}
      <div>
        <h3 className="font-medium text-gray-900 dark:text-white">
          {userEmail}
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          User Account
        </p>
      </div>

      {/* Permission */}
      <div>
        <div className="text-sm font-medium text-gray-900 dark:text-white">
          {basePerm || permission}
        </div>
        {isEmbedded && (
          <div className="text-xs text-blue-600 dark:text-blue-400">
            🎯 Embedded: :{timestamp}
          </div>
        )}
      </div>

      {/* Status */}
      <div>
        <div className={`text-sm font-medium ${statusColor}`}>
          {timeLeft}
        </div>
        {expiresAt && (
          <div className="text-xs text-gray-500">
            Expires: {new Date(expiresAt).toLocaleDateString()}
          </div>
        )}
      </div>

      {/* Actions */}
      <PermissionActions 
        userEmail={userEmail}
        permission={permission}
        status={status}
        onExtend={(email, perm) => onExtend()}
        onRevoke={(email, perm) => onRevoke()}
      />
    </div>
  )
}

function PermissionStatsCards({ analytics }: { analytics: any }) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
      <div className="bg-blue-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">🔐 Total Permissions</h3>
        <p className="text-2xl font-bold">{analytics.total_permissions?.toLocaleString() || 0}</p>
        <p className="text-xs opacity-75">{analytics.users_with_permissions} users affected</p>
      </div>
      
      <div className="bg-green-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">👥 Active Users</h3>
        <p className="text-2xl font-bold">{analytics.users_with_permissions?.toLocaleString() || 0}</p>
        <p className="text-xs opacity-75">with permissions</p>
      </div>
      
      <div className="bg-orange-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">⏰ Expiring Soon</h3>
        <p className="text-2xl font-bold">{analytics.expiring_soon || 0}</p>
        <p className="text-xs opacity-75">next 24 hours</p>
      </div>
      
      <div className="bg-red-500 text-white p-4 rounded-lg">
        <h3 className="text-sm font-medium opacity-90">💥 Expired</h3>
        <p className="text-2xl font-bold">{analytics.expired || 0}</p>
        <p className="text-xs opacity-75">needs cleanup</p>
      </div>
    </div>
  )
}

export default async function PermissionsHub() {
  // Fetch data on server side
  const usersResult = await getUsersList({ page: 1, limit: 50 })
  
  // Mock analytics data for now (replace with actual server-side API when available)
  const analytics = {
    total_permissions: 42,
    users_with_permissions: 15,
    expiring_soon: 3,
    expired: 1,
    health_score: 85,
    recent_activity: 7
  }

  const users = usersResult.success ? usersResult.data?.users || [] : []

  // Create mock permission data with embedded timestamps for demonstration
  const mockPermissions = users.flatMap((user: any) => {
    // Get permissions from the new structured system
    const userPermissions = user.customPermissions || []
    if (userPermissions.length === 0) {
      // If no permissions, add some mock ones for demonstration
      return [
        {
          userEmail: user.email,
          permission: 'epsx:analytics:view',
          expiresAt: null
        },
        {
          userEmail: user.email,  
          permission: 'admin:users:view',
          expiresAt: null
        }
      ]
    }
    
    return userPermissions.map((permission: string) => ({
      userEmail: user.email,
      permission,
      // Simulate some expiring permissions
      expiresAt: permission.includes(':1735689600') ? 1735689600 * 1000 : 
                 permission.includes(':1703980800') ? 1703980800 * 1000 : null
    }))
  })

  // Placeholder handlers for now - these will be replaced with proper server actions
  const handleExtend = async (userEmail: string, permission: string) => {
    console.log('Extend permission:', userEmail, permission)
    // This would trigger the extend permission action
  }

  const handleRevoke = async (userEmail: string, permission: string) => {
    console.log('Revoke permission:', userEmail, permission)
    // This would trigger the revoke permission action
  }


  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-4xl font-light text-gray-900 dark:text-white mb-2">
          🔐 PERMISSIONS HUB
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Advanced permission management with embedded timestamps
        </p>
      </div>

      {/* Statistics Cards */}
      <PermissionStatsCards analytics={analytics} />

      {/* Health Score Banner */}
      <div className="mb-6 p-4 bg-gradient-to-r from-green-500 to-blue-500 text-white rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">🎯 Permission Health Score</h3>
            <p className="text-sm opacity-90">
              {analytics.health_score}% excellent | 🔄 Auto-cleanup enabled | ⚡ {analytics.recent_activity} recent activities
            </p>
          </div>
          <div className="text-3xl font-bold">
            {analytics.health_score}%
          </div>
        </div>
      </div>

      {/* Pivot Navigation */}
      <div className="mb-6">
        <div className="flex overflow-x-auto gap-1 border-b border-gray-200 dark:border-gray-700">
          <button className="px-4 py-3 font-medium text-blue-600 border-b-2 border-blue-600 whitespace-nowrap">
            ◄ OVERVIEW ►
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            EMBEDDED
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            EXPIRING
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            ANALYTICS
          </button>
          <button className="px-4 py-3 font-medium text-gray-600 hover:text-gray-900 whitespace-nowrap">
            PROFILES
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="flex flex-col sm:flex-row gap-4 mb-6">
        <div className="flex gap-2">
          <button className="flex items-center gap-2 px-4 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
            <Plus size={20} />
            Grant Permission
          </button>
          <button className="flex items-center gap-2 px-4 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors">
            <Clock size={20} />
            Bulk Extend
          </button>
          <CleanupButton />
          <button className="flex items-center gap-2 px-4 py-3 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors">
            <TrendingUp size={20} />
            📊 Report
          </button>
        </div>
      </div>

      {/* Permissions List */}
      <div className="space-y-3">
        <div className="flex items-center gap-2 mb-4">
          <Shield className="text-blue-600" size={20} />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Active Permissions ({mockPermissions.length})
          </h2>
        </div>
        
        {mockPermissions.map((perm: any, index: number) => (
          <PermissionRow 
            key={index}
            userEmail={perm.userEmail}
            permission={perm.permission}
            expiresAt={perm.expiresAt}
            onExtend={() => handleExtend(perm.userEmail, perm.permission)}
            onRevoke={() => handleRevoke(perm.userEmail, perm.permission)}
          />
        ))}
      </div>

      {/* Summary Statistics */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            📈 Permission Trends
          </h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Never Expiring:</span>
              <span className="font-medium text-green-600">{mockPermissions.filter(p => !p.expiresAt).length}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Time-limited:</span>
              <span className="font-medium text-blue-600">{mockPermissions.filter(p => p.expiresAt).length}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Admin Permissions:</span>
              <span className="font-medium text-red-600">{mockPermissions.filter(p => p.permission.includes('admin:')).length}</span>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            🎯 Quick Actions
          </h3>
          <div className="space-y-3">
            <button className="w-full text-left p-3 bg-blue-50 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-800 transition-colors">
              🔍 Audit All Permissions
            </button>
            <button className="w-full text-left p-3 bg-green-50 dark:bg-green-900 text-green-800 dark:text-green-200 rounded-lg hover:bg-green-100 dark:hover:bg-green-800 transition-colors">
              📊 Generate Report
            </button>
            <button className="w-full text-left p-3 bg-orange-50 dark:bg-orange-900 text-orange-800 dark:text-orange-200 rounded-lg hover:bg-orange-100 dark:hover:bg-orange-800 transition-colors">
              ⚠️ Review Expiring Soon
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}