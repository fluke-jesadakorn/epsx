/**
 * Permission Assignment Card Component
 * Shows individual permission assignments with management functionality
 */

'use client'

import { useState } from 'react'
import { MoreHorizontal, Key, AlertCircle, Trash2 } from 'lucide-react'
import { Button } from '@epsx/ui'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Badge } from '@epsx/ui'
import { ConfirmDialog } from '@/components/ui/ConfirmDialog'
import { toast } from '@/components/ui/toast'
import type { Permission } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'
import { removeCustomPermission } from '@/lib/actions/unified-user-actions'

interface PermissionAssignmentCardProps {
  permission: Permission
  userId: string
  canManage: boolean
  onPermissionRemoved?: () => void
}

export function PermissionAssignmentCard({ 
  permission, 
  userId, 
  canManage, 
  onPermissionRemoved 
}: PermissionAssignmentCardProps) {
  const [isRemoving, setIsRemoving] = useState(false)
  const [showConfirmDialog, setShowConfirmDialog] = useState(false)

  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  // Color coding based on permission type/category
  const getPermissionColor = (permissionName: string) => {
    if (permissionName.includes('admin') || permissionName.includes('super')) {
      return 'text-red-500'
    }
    if (permissionName.includes('write') || permissionName.includes('manage')) {
      return 'text-orange-500'
    }
    return 'text-green-500'
  }

  const getActionColor = (action: string) => {
    switch (action.toLowerCase()) {
      case 'read':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'
      case 'write':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
      case 'delete':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
      case 'admin':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200'
    }
  }

  const isHighRisk = permission.name.includes('admin') || permission.name.includes('delete')

  const handleRemovePermission = async () => {
    setIsRemoving(true)
    try {
      // Parse permission name to extract resource and action
      // Format: "resource:action" or just use the name as resource with 'read' as default action
      const parts = permission.name.split(':')
      const resource = parts[0] || permission.name
      const action = parts[1] || 'read'

      const result = await removeCustomPermission({
        userId,
        resource,
        action,
        reason: `Permission removed by admin`
      })
      
      if (result.success) {
        toast.success(`Permission "${permission.name}" removed successfully`)
        onPermissionRemoved?.()
      } else {
        toast.error(result.error?.message || 'Failed to remove permission')
      }
    } catch (error) {
      toast.error('Failed to remove permission')
    } finally {
      setIsRemoving(false)
      setShowConfirmDialog(false)
    }
  }

  // Extract action from permission name for display
  const getActionFromPermissionName = (name: string): string => {
    if (name.includes(':')) {
      return name.split(':')[1] || 'read'
    }
    // Infer action from permission name patterns
    if (name.includes('delete') || name.includes('remove')) return 'delete'
    if (name.includes('write') || name.includes('update') || name.includes('create')) return 'write'
    if (name.includes('admin')) return 'admin'
    return 'read'
  }

  return (
    <>
      <div className="flex items-center justify-between p-3 border border-muted rounded-lg hover:bg-muted/30 transition-colors">
        <div className="flex items-center gap-3">
          <div className="flex items-center">
            <Key className={`h-4 w-4 ${getPermissionColor(permission.name)}`} />
            {isHighRisk && (
              <AlertCircle className="h-3 w-3 text-red-500 ml-1" />
            )}
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-medium text-sm">{permission.name}</span>
              <UserStatusBadge 
                status={permission.isActive ? 'active' : 'disabled'} 
                size="sm" 
              />
              <Badge className={`text-xs px-2 py-0.5 ${getActionColor(getActionFromPermissionName(permission.name))}`}>
                {getActionFromPermissionName(permission.name).toUpperCase()}
              </Badge>
            </div>
            <div className="text-xs text-muted-foreground flex items-center gap-2">
              <span>{permission.description || 'Custom permission'}</span>
              {permission.assignedAt && (
                <>
                  <span>•</span>
                  <span>Assigned {formatDate(permission.assignedAt)}</span>
                </>
              )}
            </div>
            {permission.expiresAt && (
              <div className="text-xs text-orange-600 mt-1">
                Expires {formatDate(permission.expiresAt)}
              </div>
            )}
          </div>
        </div>

        {canManage && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                <MoreHorizontal className="h-4 w-4" />
                <span className="sr-only">Permission actions</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem
                onClick={() => setShowConfirmDialog(true)}
                className="text-destructive focus:text-destructive"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Remove Permission
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>

      <ConfirmDialog
        isOpen={showConfirmDialog}
        onOpenChange={setShowConfirmDialog}
        title="Remove Permission"
        description={`Are you sure you want to remove the "${permission.name}" permission? This action cannot be undone.`}
        confirmText="Remove Permission"
        onConfirm={handleRemovePermission}
        isLoading={isRemoving}
        variant="destructive"
      />
    </>
  )
}