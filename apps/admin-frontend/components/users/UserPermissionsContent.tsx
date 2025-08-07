/**
 * User Permissions Content Component
 * Comprehensive permissions management consolidating IAM features
 */

'use client'

import { useState } from 'react'
import { Shield, Users, Key, Settings, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth-enhanced'
import { PermissionAssignmentCard } from './PermissionAssignmentCard'
import { RoleAssignmentCard } from './RoleAssignmentCard'
import { PermissionProfileCard } from './PermissionProfileCard'
import { RoleAssignmentModal } from './RoleAssignmentModal'
import { CustomPermissionModal } from './CustomPermissionModal'
import { PermissionProfileModal } from './PermissionProfileModal'

interface UserPermissionsContentProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
  onUserUpdated?: () => void
}

export function UserPermissionsContent({ 
  user, 
  currentUser, 
  onUserUpdated 
}: UserPermissionsContentProps) {
  const canManagePermissions = currentUser.isSuperAdmin || currentUser.canManageUsers
  
  // Modal states
  const [showRoleModal, setShowRoleModal] = useState(false)
  const [showPermissionModal, setShowPermissionModal] = useState(false)
  const [showProfileModal, setShowProfileModal] = useState(false)

  // Calculate permission stats
  const activeRoles = user.roles.filter(r => r.isActive).length
  const totalPermissions = user.customPermissions.length
  const activeProfiles = user.permissionProfiles.filter(p => p.isActive).length

  // Get existing role names and profile IDs for the modals
  const existingRoles = user.roles.map(r => r.name)
  const existingProfileIds = user.permissionProfiles.map(p => p.id)

  const handleRefreshData = () => {
    onUserUpdated?.()
  }

  return (
    <div className="space-y-6">
      {/* Permission Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{activeRoles}</p>
              <p className="text-sm text-muted-foreground">Active Roles</p>
            </div>
            <Users className="h-8 w-8 text-blue-500" />
          </div>
        </div>
        
        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{totalPermissions}</p>
              <p className="text-sm text-muted-foreground">Custom Permissions</p>
            </div>
            <Key className="h-8 w-8 text-green-500" />
          </div>
        </div>

        <div className="pancake-card p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{activeProfiles}</p>
              <p className="text-sm text-muted-foreground">Permission Profiles</p>
            </div>
            <Shield className="h-8 w-8 text-purple-500" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Role Assignments */}
        <div className="pancake-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <Users className="h-5 w-5" />
              Role Assignments
            </h3>
            {canManagePermissions && (
              <Button
                onClick={() => setShowRoleModal(true)}
                size="sm"
                className="bg-blue-600 hover:bg-blue-700"
              >
                <Plus className="h-4 w-4 mr-1" />
                Assign Role
              </Button>
            )}
          </div>
          
          <div className="space-y-3">
            {user.roles.length > 0 ? (
              user.roles.map((role) => (
                <RoleAssignmentCard 
                  key={role.id}
                  role={role}
                  userId={user.id}
                  canManage={canManagePermissions}
                  onRoleRemoved={handleRefreshData}
                />
              ))
            ) : (
              <div className="text-center text-muted-foreground py-8">
                <Users className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>No roles assigned</p>
                {canManagePermissions && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowRoleModal(true)}
                    className="mt-2"
                  >
                    Assign First Role
                  </Button>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Permission Profiles */}
        <div className="pancake-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Permission Profiles
            </h3>
            {canManagePermissions && (
              <Button
                onClick={() => setShowProfileModal(true)}
                size="sm"
                className="bg-purple-600 hover:bg-purple-700"
              >
                <Plus className="h-4 w-4 mr-1" />
                Assign Profile
              </Button>
            )}
          </div>
          
          <div className="space-y-3">
            {user.permissionProfiles.length > 0 ? (
              user.permissionProfiles.map((profile) => (
                <PermissionProfileCard 
                  key={profile.id}
                  profile={profile}
                  canManage={canManagePermissions}
                />
              ))
            ) : (
              <div className="text-center text-muted-foreground py-8">
                <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>No permission profiles assigned</p>
                {canManagePermissions && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowProfileModal(true)}
                    className="mt-2"
                  >
                    Assign Profile
                  </Button>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Custom Permissions */}
      <div className="pancake-card p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Key className="h-5 w-5" />
            Custom Permissions
          </h3>
          {canManagePermissions && (
            <Button
              onClick={() => setShowPermissionModal(true)}
              size="sm"
              className="bg-green-600 hover:bg-green-700"
            >
              <Plus className="h-4 w-4 mr-1" />
              Add Permission
            </Button>
          )}
        </div>
        
        <div className="space-y-2">
          {user.customPermissions.length > 0 ? (
            user.customPermissions.map((permission) => (
              <PermissionAssignmentCard 
                key={permission.id}
                permission={permission}
                userId={user.id}
                canManage={canManagePermissions}
                onPermissionRemoved={handleRefreshData}
              />
            ))
          ) : (
            <div className="text-center text-muted-foreground py-8">
              <Key className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No custom permissions assigned</p>
              {canManagePermissions && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setShowPermissionModal(true)}
                  className="mt-2"
                >
                  Add First Permission
                </Button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Permission History */}
      <div className="pancake-card p-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Settings className="h-5 w-5" />
          Permission History
        </h3>
        
        <div className="space-y-3">
          {/* This would show recent permission changes */}
          <div className="text-center text-muted-foreground py-8">
            <Settings className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p>Permission history will be shown here</p>
            <p className="text-xs">Track role assignments, profile changes, and custom permissions</p>
          </div>
        </div>
      </div>

      {/* Modals */}
      <RoleAssignmentModal
        isOpen={showRoleModal}
        onOpenChange={setShowRoleModal}
        userId={user.id}
        userEmail={user.email}
        existingRoles={existingRoles}
        onRoleAssigned={handleRefreshData}
      />

      <CustomPermissionModal
        isOpen={showPermissionModal}
        onOpenChange={setShowPermissionModal}
        userId={user.id}
        userEmail={user.email}
        onPermissionAdded={handleRefreshData}
      />

      <PermissionProfileModal
        isOpen={showProfileModal}
        onOpenChange={setShowProfileModal}
        userId={user.id}
        userEmail={user.email}
        existingProfiles={existingProfileIds}
        onProfileAssigned={handleRefreshData}
      />
    </div>
  )
}