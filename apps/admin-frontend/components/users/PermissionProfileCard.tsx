/**
 * Permission Profile Card Component
 * Shows permission profile information with management actions
 */

import { MoreHorizontal, Shield, Users } from 'lucide-react'
import type { PermissionProfile } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'

interface PermissionProfileCardProps {
  profile: PermissionProfile
  canManage: boolean
}

export function PermissionProfileCard({ profile, canManage }: PermissionProfileCardProps) {
  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  return (
    <div className="flex items-center justify-between p-3 border border-muted rounded-lg hover:bg-muted/30 transition-colors">
      <div className="flex items-center gap-3">
        <Shield className={`h-5 w-5 ${
          profile.isActive ? 'text-purple-500' : 'text-gray-400'
        }`} />
        <div>
          <div className="flex items-center gap-2">
            <span className="font-medium">{profile.name}</span>
            <UserStatusBadge 
              status={profile.isActive ? 'active' : 'disabled'} 
              size="sm" 
            />
          </div>
          <div className="text-xs text-muted-foreground flex items-center gap-2">
            <span>{profile.description}</span>
            {profile.assignedAt && (
              <>
                <span>•</span>
                <span>Assigned {formatDate(profile.assignedAt)}</span>
              </>
            )}
          </div>
          {profile.permissions && profile.permissions.length > 0 && (
            <div className="text-xs text-muted-foreground mt-1 flex items-center gap-1">
              <Users className="h-3 w-3" />
              <span>{profile.permissions.length} permissions included</span>
            </div>
          )}
        </div>
      </div>

      {canManage && (
        <button className="p-2 hover:bg-muted rounded-md transition-colors">
          <MoreHorizontal className="h-4 w-4" />
          <span className="sr-only">Profile actions</span>
        </button>
      )}
    </div>
  )
}