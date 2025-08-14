/**
 * User Permissions Server Component - Server-side first architecture
 * Fetches data and renders permission management interface
 */

import { Shield, Users, Key } from 'lucide-react'
import { auth } from '@/lib/auth'
import { getUnifiedUserData } from '@/lib/actions/user-profile-actions'
import { getPermissionHistory } from '@/lib/actions/user-permissions-actions'
import type { Session } from 'next-auth'
import { UserPermissionsClient } from './UserPermissionsClient'
import { PermissionStatsCards } from './PermissionStatsCards'
import { PermissionHistoryCard } from './PermissionHistoryCard'

interface UserPermissionsServerProps {
  userId: string
}

export async function UserPermissionsServer({ userId }: UserPermissionsServerProps) {
  // Fetch data in parallel on the server
  const [session, userResult, historyResult] = await Promise.all([
    auth(),
    getUnifiedUserData(userId),
    getPermissionHistory(userId, 50)
  ])

  // Handle authentication
  if (!session?.user) {
    return (
      <div className="pancake-card p-6 text-center">
        <p className="text-muted-foreground">Authentication required</p>
      </div>
    )
  }

  // Handle user data fetch error
  if (!userResult.success) {
    return (
      <div className="pancake-card p-6 text-center">
        <div className="text-red-500 mb-2">Failed to load user data</div>
        <p className="text-sm text-muted-foreground">{userResult.error.message}</p>
      </div>
    )
  }

  const currentUser = currentUserResult
  const user = userResult.data
  const permissionHistory = historyResult.success ? historyResult.data : []

  // Server-side permission calculations
  const activeRoles = user.roles.filter(r => r.isActive !== false).length
  const totalPermissions = user.customPermissions.length
  const activeProfiles = user.permissionProfiles.filter(p => p.isActive !== false).length

  // Check permissions on server
  const canManagePermissions = currentUser.admin && 
    (currentUser.admin_modules.includes('permission_admin') || 
     currentUser.admin_modules.includes('system_admin'))

  return (
    <div className="space-y-6">
      {/* Permission Summary Stats - Server Rendered */}
      <PermissionStatsCards 
        activeRoles={activeRoles}
        totalPermissions={totalPermissions}
        activeProfiles={activeProfiles}
      />

      {/* Main Permission Management - Client Component for Interactions */}
      <UserPermissionsClient
        user={user}
        currentUser={currentUser}
        canManagePermissions={canManagePermissions}
      />

      {/* Permission History - Server Component with Server Data */}
      {historyResult.success && (
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Permission History
          </h3>
          <PermissionHistoryDisplay history={permissionHistory} />
        </div>
      )}
    </div>
  )
}

/**
 * Permission Stats Cards - Pure Server Component
 */
function PermissionHistoryDisplay({ 
  history 
}: { 
  history: Array<{
    id: string
    action: string
    resource: string
    granted: boolean
    timestamp: Date
    reason?: string
    grantedBy: string
    expires?: Date
  }>
}) {
  if (history.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        <Shield className="h-12 w-12 mx-auto mb-2 opacity-50" />
        <p>No permission history available</p>
      </div>
    )
  }

  return (
    <div className="space-y-3 max-h-96 overflow-y-auto">
      {history.map((entry) => (
        <div 
          key={entry.id}
          className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
        >
          <div className="flex-1">
            <div className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full ${
                entry.granted ? 'bg-green-500' : 'bg-red-500'
              }`} />
              <span className="font-medium">
                {entry.granted ? 'Granted' : 'Revoked'} {entry.resource}:{entry.action}
              </span>
            </div>
            <div className="text-sm text-muted-foreground mt-1">
              By {entry.grantedBy} • {entry.timestamp.toLocaleDateString()}
              {entry.expires && (
                <span className="ml-2">
                  • Expires {entry.expires.toLocaleDateString()}
                </span>
              )}
            </div>
            {entry.reason && (
              <div className="text-sm text-gray-600 mt-1 italic">
                "{entry.reason}"
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}