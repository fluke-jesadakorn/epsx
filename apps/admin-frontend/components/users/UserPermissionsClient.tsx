/**
 * User Permissions Client Component - Interactive parts only
 * Handles user interactions and form submissions using Server Actions
 */

'use client'

import { useState, useTransition } from 'react'
import { Users, Key, Shield, Plus, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useRouter } from 'next/navigation'
import { toast } from 'react-hot-toast'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth'
import { assignUserRole, removeUserRole, addCustomPermission, removeCustomPermission } from '@/lib/actions/user-permissions-actions'
import { PermissionAssignmentCard } from './PermissionAssignmentCard'
import { RoleAssignmentCard } from './RoleAssignmentCard'

interface UserPermissionsClientProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
  canManagePermissions: boolean
}

export function UserPermissionsClient({ 
  user, 
  currentUser, 
  canManagePermissions 
}: UserPermissionsClientProps) {
  const router = useRouter()
  const [isPending, startTransition] = useTransition()
  const [showRoleForm, setShowRoleForm] = useState(false)
  const [showPermissionForm, setShowPermissionForm] = useState(false)

  const handleRefreshData = () => {
    startTransition(() => {
      router.refresh()
    })
  }

  const handleRoleAssignment = async (roleId: string, reason?: string) => {
    startTransition(async () => {
      const result = await assignUserRole({
        userId: user.id,
        roleId,
        reason
      })

      if (result.success) {
        toast.success(`Role assigned successfully`)
        setShowRoleForm(false)
        router.refresh()
      } else {
        toast.error(result.error.message || 'Failed to assign role')
      }
    })
  }

  const handleRoleRemoval = async (roleId: string, reason?: string) => {
    startTransition(async () => {
      const result = await removeUserRole({
        userId: user.id,
        roleId,
        reason
      })

      if (result.success) {
        toast.success(`Role removed successfully`)
        router.refresh()
      } else {
        toast.error(result.error.message || 'Failed to remove role')
      }
    })
  }

  const handlePermissionAdd = async (resource: string, action: string, reason?: string) => {
    startTransition(async () => {
      const result = await addCustomPermission({
        userId: user.id,
        resource,
        action,
        reason
      })

      if (result.success) {
        toast.success(`Permission added successfully`)
        setShowPermissionForm(false)
        router.refresh()
      } else {
        toast.error(result.error.message || 'Failed to add permission')
      }
    })
  }

  const handlePermissionRemoval = async (permissionId: string, reason?: string) => {
    startTransition(async () => {
      const result = await removeCustomPermission({
        userId: user.id,
        permissionId,
        reason
      })

      if (result.success) {
        toast.success(`Permission removed successfully`)
        router.refresh()
      } else {
        toast.error(result.error.message || 'Failed to remove permission')
      }
    })
  }

  return (
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
              variant="outline"
              size="sm"
              onClick={() => setShowRoleForm(!showRoleForm)}
              disabled={isPending}
            >
              <Plus className="h-4 w-4 mr-1" />
              Add Role
            </Button>
          )}
        </div>
        
        <div className="space-y-4">
          {user.roles.length > 0 ? (
            <div className="space-y-3">
              {user.roles.map((role) => (
                <div key={role.id} className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">{role.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {role.description}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">
                        Added: {role.createdAt.toLocaleDateString()}
                      </div>
                    </div>
                    {canManagePermissions && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleRoleRemoval(role.id)}
                        disabled={isPending}
                      >
                        Remove
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-6 text-muted-foreground">
              <Users className="h-12 w-12 mx-auto mb-2 opacity-50" />
              <p>No roles assigned</p>
            </div>
          )}
          
          {showRoleForm && canManagePermissions && (
            <RoleAssignmentForm
              onAssign={handleRoleAssignment}
              onCancel={() => setShowRoleForm(false)}
              existingRoles={user.roles.map(r => r.id)}
              isSubmitting={isPending}
            />
          )}
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
              variant="outline"
              size="sm"
              onClick={() => setShowPermissionForm(!showPermissionForm)}
              disabled={isPending}
            >
              <Plus className="h-4 w-4 mr-1" />
              Add Permission
            </Button>
          )}
        </div>
        
        <div className="space-y-4">
          {user.customPermissions.length > 0 ? (
            <div className="space-y-2">
              {user.customPermissions.map((permission) => (
                <div key={permission.id} className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">{permission.permission}</div>
                      <div className="text-sm text-muted-foreground">
                        Resource: {permission.resource || 'General'}
                      </div>
                      {permission.expires && (
                        <div className="text-xs text-orange-600">
                          Expires: {permission.expires.toLocaleDateString()}
                        </div>
                      )}
                    </div>
                    {canManagePermissions && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handlePermissionRemoval(permission.id)}
                        disabled={isPending}
                      >
                        Remove
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-6 text-muted-foreground">
              <Key className="h-12 w-12 mx-auto mb-2 opacity-50" />
              <p>No custom permissions assigned</p>
            </div>
          )}
          
          {showPermissionForm && canManagePermissions && (
            <CustomPermissionForm
              onAdd={handlePermissionAdd}
              onCancel={() => setShowPermissionForm(false)}
              isSubmitting={isPending}
            />
          )}
        </div>
      </div>
    </div>
  )
}

/**
 * Role Assignment Form - Client Component
 */
function RoleAssignmentForm({
  onAssign,
  onCancel,
  existingRoles,
  isSubmitting
}: {
  onAssign: (roleId: string, reason?: string) => void
  onCancel: () => void
  existingRoles: string[]
  isSubmitting: boolean
}) {
  const [selectedRole, setSelectedRole] = useState('')
  const [reason, setReason] = useState('')

  const availableRoles = [
    { id: 'admin', name: 'Administrator', description: 'Full system access' },
    { id: 'moderator', name: 'Moderator', description: 'Content moderation' },
    { id: 'analyst', name: 'Analyst', description: 'Analytics access' },
    { id: 'viewer', name: 'Viewer', description: 'Read-only access' }
  ].filter(role => !existingRoles.includes(role.id))

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (selectedRole) {
      onAssign(selectedRole, reason)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="p-4 border rounded-lg bg-white">
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-2">Role</label>
          <select
            value={selectedRole}
            onChange={(e) => setSelectedRole(e.target.value)}
            className="w-full p-2 border rounded"
            required
          >
            <option value="">Select a role...</option>
            {availableRoles.map(role => (
              <option key={role.id} value={role.id}>
                {role.name} - {role.description}
              </option>
            ))}
          </select>
        </div>
        
        <div>
          <label className="block text-sm font-medium mb-2">Reason (Optional)</label>
          <input
            type="text"
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            className="w-full p-2 border rounded"
            placeholder="Why is this role being assigned?"
          />
        </div>
        
        <div className="flex gap-2">
          <Button type="submit" disabled={!selectedRole || isSubmitting}>
            {isSubmitting ? 'Assigning...' : 'Assign Role'}
          </Button>
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
        </div>
      </div>
    </form>
  )
}

/**
 * Custom Permission Form - Client Component
 */
function CustomPermissionForm({
  onAdd,
  onCancel,
  isSubmitting
}: {
  onAdd: (resource: string, action: string, reason?: string) => void
  onCancel: () => void
  isSubmitting: boolean
}) {
  const [resource, setResource] = useState('')
  const [action, setAction] = useState('')
  const [reason, setReason] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (resource && action) {
      onAdd(resource, action, reason)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="p-4 border rounded-lg bg-white">
      <div className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-2">Resource</label>
            <input
              type="text"
              value={resource}
              onChange={(e) => setResource(e.target.value)}
              className="w-full p-2 border rounded"
              placeholder="e.g., users, dashboard"
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium mb-2">Action</label>
            <select
              value={action}
              onChange={(e) => setAction(e.target.value)}
              className="w-full p-2 border rounded"
              required
            >
              <option value="">Select action...</option>
              <option value="read">Read</option>
              <option value="write">Write</option>
              <option value="delete">Delete</option>
              <option value="admin">Admin</option>
            </select>
          </div>
        </div>
        
        <div>
          <label className="block text-sm font-medium mb-2">Reason (Optional)</label>
          <input
            type="text"
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            className="w-full p-2 border rounded"
            placeholder="Why is this permission being granted?"
          />
        </div>
        
        <div className="flex gap-2">
          <Button type="submit" disabled={!resource || !action || isSubmitting}>
            {isSubmitting ? 'Adding...' : 'Add Permission'}
          </Button>
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
        </div>
      </div>
    </form>
  )
}