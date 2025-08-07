/**
 * Role Assignment Card Component
 * Shows role information with management actions
 */

'use client'

import { useState } from 'react'
import { MoreHorizontal, Shield, Trash2, Edit } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { ConfirmDialog } from '@/components/ui/ConfirmDialog'
import { toast } from '@/components/ui/toast'
import type { UserRole } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'
import { removeUserRole } from '@/lib/actions/unified-user-actions'

interface RoleAssignmentCardProps {
  role: UserRole
  userId: string
  canManage: boolean
  onRoleRemoved?: () => void
}

export function RoleAssignmentCard({ role, userId, canManage, onRoleRemoved }: RoleAssignmentCardProps) {
  const [isRemoving, setIsRemoving] = useState(false)
  const [showConfirmDialog, setShowConfirmDialog] = useState(false)

  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  const handleRemoveRole = async () => {
    setIsRemoving(true)
    try {
      const result = await removeUserRole({
        userId,
        role: role.name,
        reason: `Role removed by admin`
      })
      
      if (result.success) {
        toast.success(`Role "${role.name}" removed successfully`)
        onRoleRemoved?.()
      } else {
        toast.error(result.error?.message || 'Failed to remove role')
      }
    } catch (error) {
      toast.error('Failed to remove role')
    } finally {
      setIsRemoving(false)
      setShowConfirmDialog(false)
    }
  }

  return (
    <>
      <div className="flex items-center justify-between p-3 border border-muted rounded-lg hover:bg-muted/30 transition-colors">
        <div className="flex items-center gap-3">
          <Shield className={`h-5 w-5 ${
            role.isActive ? 'text-green-500' : 'text-gray-400'
          }`} />
          <div>
            <div className="flex items-center gap-2">
              <span className="font-medium">{role.name}</span>
              <UserStatusBadge 
                status={role.isActive ? 'active' : 'disabled'} 
                size="sm" 
              />
            </div>
            <div className="text-xs text-muted-foreground flex items-center gap-2">
              <span>{role.description}</span>
              {role.assignedAt && (
                <>
                  <span>•</span>
                  <span>Assigned {formatDate(role.assignedAt)}</span>
                </>
              )}
            </div>
          </div>
        </div>

        {canManage && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                <MoreHorizontal className="h-4 w-4" />
                <span className="sr-only">Role actions</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem
                onClick={() => setShowConfirmDialog(true)}
                className="text-destructive focus:text-destructive"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Remove Role
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>

      <ConfirmDialog
        isOpen={showConfirmDialog}
        onOpenChange={setShowConfirmDialog}
        title="Remove Role"
        description={`Are you sure you want to remove the "${role.name}" role from this user? This action cannot be undone.`}
        confirmText="Remove Role"
        onConfirm={handleRemoveRole}
        isLoading={isRemoving}
        variant="destructive"
      />
    </>
  )
}