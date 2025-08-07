/**
 * Role Assignment Modal Component
 * Allows assigning new roles to users
 */

'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { toast } from '@/components/ui/toast'
import { assignUserRole } from '@/lib/actions/unified-user-actions'

interface RoleAssignmentModalProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  userId: string
  userEmail: string
  existingRoles: string[]
  onRoleAssigned?: () => void
}

const AVAILABLE_ROLES = [
  { value: 'user', label: 'User', description: 'Basic user permissions' },
  { value: 'moderator', label: 'Moderator', description: 'Can manage other users' },
  { value: 'admin', label: 'Administrator', description: 'Full system access' },
  { value: 'premium', label: 'Premium User', description: 'Premium features access' },
  { value: 'developer', label: 'Developer', description: 'API access and development features' }
]

export function RoleAssignmentModal({
  isOpen,
  onOpenChange,
  userId,
  userEmail,
  existingRoles,
  onRoleAssigned
}: RoleAssignmentModalProps) {
  const [selectedRole, setSelectedRole] = useState('')
  const [reason, setReason] = useState('')
  const [isAssigning, setIsAssigning] = useState(false)

  // Get available roles (exclude existing ones)
  const availableRoles = AVAILABLE_ROLES.filter(role => 
    !existingRoles.includes(role.value)
  )

  // Reset form when modal opens/closes
  useEffect(() => {
    if (!isOpen) {
      setSelectedRole('')
      setReason('')
    }
  }, [isOpen])

  const handleAssignRole = async () => {
    if (!selectedRole) {
      toast.error('Please select a role to assign')
      return
    }

    setIsAssigning(true)
    try {
      const result = await assignUserRole({
        userId,
        role: selectedRole,
        reason: reason || `Role assigned to ${userEmail}`
      })
      
      if (result.success) {
        const roleLabel = AVAILABLE_ROLES.find(r => r.value === selectedRole)?.label
        toast.success(`Role "${roleLabel}" assigned successfully`)
        onRoleAssigned?.()
        onOpenChange(false)
      } else {
        toast.error(result.error?.message || 'Failed to assign role')
      }
    } catch (error) {
      toast.error('Failed to assign role')
    } finally {
      setIsAssigning(false)
    }
  }

  if (availableRoles.length === 0) {
    return (
      <Dialog open={isOpen} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Assign Role</DialogTitle>
            <DialogDescription>
              All available roles have already been assigned to {userEmail}.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    )
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Assign Role</DialogTitle>
          <DialogDescription>
            Assign a new role to {userEmail}. This will grant the user the permissions associated with the selected role.
          </DialogDescription>
        </DialogHeader>
        
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="role">Role</Label>
            <Select value={selectedRole} onValueChange={setSelectedRole}>
              <SelectTrigger>
                <SelectValue placeholder="Select a role to assign" />
              </SelectTrigger>
              <SelectContent>
                {availableRoles.map((role) => (
                  <SelectItem key={role.value} value={role.value}>
                    <div className="flex flex-col">
                      <span className="font-medium">{role.label}</span>
                      <span className="text-xs text-muted-foreground">{role.description}</span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          
          <div className="grid gap-2">
            <Label htmlFor="reason">Reason (Optional)</Label>
            <Textarea
              id="reason"
              placeholder="Enter reason for role assignment..."
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="min-h-[80px]"
            />
          </div>

          {existingRoles.length > 0 && (
            <div className="rounded-md bg-muted p-3">
              <h4 className="text-sm font-medium mb-2">Current Roles:</h4>
              <div className="flex flex-wrap gap-1">
                {existingRoles.map((role) => {
                  const roleInfo = AVAILABLE_ROLES.find(r => r.value === role)
                  return (
                    <span 
                      key={role}
                      className="inline-block px-2 py-1 text-xs bg-background border rounded"
                    >
                      {roleInfo?.label || role}
                    </span>
                  )
                })}
              </div>
            </div>
          )}
        </div>
        
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button 
            onClick={handleAssignRole}
            disabled={!selectedRole || isAssigning}
            className="min-w-[100px]"
          >
            {isAssigning ? 'Assigning...' : 'Assign Role'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}