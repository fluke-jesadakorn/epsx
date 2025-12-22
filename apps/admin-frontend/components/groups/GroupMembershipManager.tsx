/**
 * Group Membership Manager Component
 * Interface for managing user assignments to permission groups
 * 
 * Features:
 * - Assign users to groups with optional expiry
 * - Remove users from groups
 * - View group memberships
 * - Bulk operations for multiple users
 * - Expiry management and notifications
 */

'use client'

import { format } from 'date-fns'
import {
  Badge as BadgeIcon, CheckCircle,
  Clock,
  Info,
  MoreHorizontal,
  Search,
  UserMinus,
  UserPlus,
  Users,
  XCircle
} from 'lucide-react'
import React, { useCallback, useMemo, useState } from 'react'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog, DialogContent,
  DialogFooter,
  DialogHeader, DialogTitle
} from '@/components/ui/dialog'
import {
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { Checkbox } from '@/components/ui/FormComponents'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { useToast } from '@/components/ui/use-toast'
import { adminButtonVariants, adminCardVariants } from '@/design-system'
import {
  useGroupAssignmentHistory,
  useGroups,
  useUserGroupMemberships
} from '@/hooks/useGroupPermissions'
import {
  AssignUserToGroupRequest,
  Group,
  UserGroupMembership
} from '@/lib/api/group-management-client'
import { cn } from '@/lib/shared'

interface GroupMembershipManagerProps {
  userId?: string
  groupId?: string
  className?: string
}

/**
 *
 * @param root0
 * @param root0.userId
 * @param root0.groupId
 * @param root0.className
 */
export function GroupMembershipManager({
  userId,
  groupId: _groupId,
  className
}: GroupMembershipManagerProps) {
  const { toast } = useToast()
  const [searchTerm, setSearchTerm] = useState('')
  const [showAssignDialog, setShowAssignDialog] = useState(false)
  const [selectedMemberships, setSelectedMemberships] = useState<string[]>([])
  const [filterStatus, setFilterStatus] = useState<string>('all')

  // Hooks
  const { groups } = useGroups()
  const {
    memberships,
    activeMemberships,
    expiringMemberships,
    isLoading,
    assignUserToGroup,
    removeUserFromGroup,
    refreshMemberships
  } = useUserGroupMemberships(userId || null)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { history: _history } = useGroupAssignmentHistory()

  // Filter memberships
  const filteredMemberships = useMemo(() => {
    let filtered = memberships

    // Filter by status
    if (filterStatus === 'active') {
      filtered = filtered.filter(m => m.is_active)
    } else if (filterStatus === 'expired') {
      filtered = filtered.filter(m => !m.is_active)
    } else if (filterStatus === 'expiring') {
      const now = Date.now()
      const sevenDaysFromNow = now + (7 * 24 * 60 * 60 * 1000)
      filtered = filtered.filter(m =>
        m.is_active && m.expires_at && new Date(m.expires_at).getTime() <= sevenDaysFromNow
      )
    }

    // Filter by search term
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase()
      filtered = filtered.filter(m =>
        m.group?.name.toLowerCase().includes(searchLower) ||
        m.group?.description?.toLowerCase().includes(searchLower)
      )
    }

    return filtered.sort((a, b) =>
      new Date(b.granted_at).getTime() - new Date(a.granted_at).getTime()
    )
  }, [memberships, filterStatus, searchTerm])

  // Event handlers
  const handleAssignGroup = useCallback(async (request: AssignUserToGroupRequest) => {
    try {
      await assignUserToGroup(request)
      await refreshMemberships()
      setShowAssignDialog(false)
      toast({
        title: 'User Assigned',
        description: 'User has been successfully assigned to the group.'
      })
    } catch (_error) {
      toast({
        title: 'Assignment Failed',
        description: _error instanceof Error ? _error.message : 'Failed to assign user to group',
        variant: 'destructive'
      })
    }
  }, [assignUserToGroup, refreshMemberships, toast])

  const handleRemoveFromGroup = useCallback(async (membership: UserGroupMembership) => {
    try {
      await removeUserFromGroup(membership.group_id)
      toast({
        title: 'User Removed',
        description: 'User has been removed from the group.'
      })
    } catch (_error) {
      toast({
        title: 'Removal Failed',
        description: _error instanceof Error ? _error.message : 'Failed to remove user from group',
        variant: 'destructive'
      })
    }
  }, [removeUserFromGroup, toast])

  const handleBulkRemove = useCallback(async () => {
    if (selectedMemberships.length === 0) { return }

    const promises = selectedMemberships.map(membershipId => {
      const membership = memberships.find(m => m.id === membershipId)
      return membership ? removeUserFromGroup(membership.group_id) : Promise.resolve()
    })

    try {
      await Promise.all(promises)
      setSelectedMemberships([])
      toast({
        title: 'Bulk Removal Complete',
        description: `Removed ${selectedMemberships.length} group memberships.`
      })
    } catch (_error) {
      toast({
        title: 'Bulk Removal Failed',
        description: 'Some memberships could not be removed.',
        variant: 'destructive'
      })
    }
  }, [selectedMemberships, memberships, removeUserFromGroup, toast])

  const toggleMembershipSelection = useCallback((membershipId: string) => {
    setSelectedMemberships(prev =>
      prev.includes(membershipId)
        ? prev.filter(id => id !== membershipId)
        : [...prev, membershipId]
    )
  }, [])

  const selectAllMemberships = useCallback(() => {
    setSelectedMemberships(
      selectedMemberships.length === filteredMemberships.length
        ? []
        : filteredMemberships.map(m => m.id)
    )
  }, [selectedMemberships.length, filteredMemberships])

  const getMembershipStatusBadge = (membership: UserGroupMembership) => {
    if (!membership.is_active) {
      return <Badge variant="destructive">Expired</Badge>
    }

    if (membership.expires_at) {
      const expiresAt = new Date(membership.expires_at)
      const now = new Date()
      const daysUntilExpiry = Math.ceil((expiresAt.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))

      if (daysUntilExpiry <= 7) {
        return <Badge variant="secondary">Expires in {daysUntilExpiry} days</Badge>
      }
    }

    return <Badge variant="default">Active</Badge>
  }

  if (isLoading) {
    return (
      <div className={cn('space-y-4', className)}>
        <div className="animate-pulse space-y-4">
          {[...Array(3)].map((_, i) => (
            <Card key={i} className="p-4">
              <div className="flex items-center space-x-4">
                <div className="h-4 w-4 bg-gray-200 rounded"></div>
                <div className="flex-1 space-y-2">
                  <div className="h-4 bg-gray-200 rounded w-1/4"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                </div>
              </div>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h2 className="text-xl font-semibold">Group Memberships</h2>
          <p className="text-sm text-gray-600">
            Manage user assignments to permission groups
          </p>
        </div>
        {userId && (
          <Button
            onClick={() => setShowAssignDialog(true)}
            className={adminButtonVariants({ variant: 'primary', size: 'sm' })}
          >
            <UserPlus className="h-4 w-4 mr-2" />
            Assign to Group
          </Button>
        )}
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Total Memberships</p>
              <p className="text-2xl font-bold">{memberships.length}</p>
            </div>
            <Users className="h-6 w-6 text-blue-600" />
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Active</p>
              <p className="text-2xl font-bold text-green-600">{activeMemberships.length}</p>
            </div>
            <CheckCircle className="h-6 w-6 text-green-600" />
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Expiring Soon</p>
              <p className="text-2xl font-bold text-orange-600">{expiringMemberships.length}</p>
            </div>
            <Clock className="h-6 w-6 text-orange-600" />
          </div>
        </Card>

        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Expired</p>
              <p className="text-2xl font-bold text-red-600">
                {memberships.filter(m => !m.is_active).length}
              </p>
            </div>
            <XCircle className="h-6 w-6 text-red-600" />
          </div>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
          <Input
            placeholder="Search groups..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-10"
          />
        </div>
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-full sm:w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Statuses</SelectItem>
            <SelectItem value="active">Active</SelectItem>
            <SelectItem value="expiring">Expiring Soon</SelectItem>
            <SelectItem value="expired">Expired</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Bulk Actions */}
      {selectedMemberships.length > 0 && (
        <Card className="p-4 bg-blue-50 border-blue-200">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-blue-900">
              {selectedMemberships.length} membership(s) selected
            </span>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={handleBulkRemove}>
                <UserMinus className="h-4 w-4 mr-2" />
                Remove Selected
              </Button>
              <Button variant="outline" size="sm" onClick={() => setSelectedMemberships([])}>
                Clear Selection
              </Button>
            </div>
          </div>
        </Card>
      )}

      {/* Memberships List */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <BadgeIcon className="h-5 w-5" />
              Group Memberships ({filteredMemberships.length})
            </CardTitle>
            {filteredMemberships.length > 0 && (
              <Checkbox
                checked={selectedMemberships.length === filteredMemberships.length}
                onChange={selectAllMemberships}
                className="mr-2"
              />
            )}
          </div>
        </CardHeader>
        <CardContent>
          {filteredMemberships.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No memberships found</h3>
              <p className="text-gray-600">
                {searchTerm || filterStatus !== 'all'
                  ? 'No memberships match your search criteria.'
                  : 'This user is not assigned to any groups yet.'
                }
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {filteredMemberships.map((membership) => (
                <div
                  key={membership.id}
                  className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50"
                >
                  <Checkbox
                    checked={selectedMemberships.includes(membership.id)}
                    onChange={() => toggleMembershipSelection(membership.id)}
                    className="mr-4"
                  />

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <h4 className="font-medium text-gray-900">
                        {membership.group?.name}
                      </h4>
                      {getMembershipStatusBadge(membership)}
                      {(membership.group?.group_type === 'system' || membership.group?.group_type === 'admin') && (
                        <Badge variant="outline">System</Badge>
                      )}
                    </div>

                    {membership.group?.description && (
                      <p className="text-sm text-gray-600 mb-2">
                        {membership.group.description}
                      </p>
                    )}

                    <div className="flex items-center gap-4 text-xs text-gray-500">
                      <span>
                        Granted: {format(new Date(membership.granted_at), 'MMM d, yyyy')}
                      </span>
                      {membership.expires_at && (
                        <span>
                          Expires: {format(new Date(membership.expires_at), 'MMM d, yyyy')}
                        </span>
                      )}
                      <span>
                        Permissions: {membership.group?.permissions.length || 0}
                      </span>
                    </div>
                  </div>

                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => handleRemoveFromGroup(membership)}>
                        <UserMinus className="h-4 w-4 mr-2" />
                        Remove from Group
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Assign Group Dialog */}
      <Dialog open={showAssignDialog} onOpenChange={setShowAssignDialog}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Assign User to Group</DialogTitle>
          </DialogHeader>
          <AssignGroupForm
            userId={userId!}
            availableGroups={groups.filter(g =>
              !memberships.some(m => m.group_id === g.id && m.is_active)
            )}
            onAssign={handleAssignGroup}
            onCancel={() => setShowAssignDialog(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

// Assign Group Form Component
interface AssignGroupFormProps {
  userId: string
  availableGroups: Group[]
  onAssign: (request: AssignUserToGroupRequest) => void
  onCancel: () => void
}

function AssignGroupForm({ userId, availableGroups, onAssign, onCancel }: AssignGroupFormProps) {
  const [selectedGroupId, setSelectedGroupId] = useState('')
  const [expiryDays, setExpiryDays] = useState<string>('')
  const [reason, setReason] = useState('')

  const selectedGroup = availableGroups.find(g => g.id === selectedGroupId)

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault()
    if (!selectedGroupId) { return }

    const expiresAt = expiryDays
      ? new Date(Date.now() + parseInt(expiryDays) * 24 * 60 * 60 * 1000).toISOString()
      : null

    onAssign({
      user_id: userId,
      group_id: selectedGroupId,
      expires_at: expiresAt,
      reason: reason.trim() || undefined
    })
  }, [selectedGroupId, expiryDays, reason, userId, onAssign])

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <Label htmlFor="group">Select Group *</Label>
        <Select value={selectedGroupId} onValueChange={setSelectedGroupId}>
          <SelectTrigger>
            <SelectValue placeholder="Choose a group" />
          </SelectTrigger>
          <SelectContent>
            {availableGroups.map((group) => (
              <SelectItem key={group.id} value={group.id}>
                <div className="flex items-center gap-2">
                  {group.group_type === 'system' || group.group_type === 'admin' ? (
                    <BadgeIcon className="h-3 w-3 text-yellow-600" />
                  ) : (
                    <Users className="h-3 w-3 text-blue-600" />
                  )}
                  {group.name}
                </div>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {selectedGroup && (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>
            This group includes {selectedGroup.permissions.length} permissions.
            {selectedGroup.group_metadata?.['default_expiry_days'] && (
              ` Default expiry: ${selectedGroup.group_metadata['default_expiry_days']} days.`
            )}
          </AlertDescription>
        </Alert>
      )}

      <div>
        <Label htmlFor="expiry">Expiry (days)</Label>
        <Input
          id="expiry"
          type="number"
          value={expiryDays}
          onChange={(e) => setExpiryDays(e.target.value)}
          placeholder={selectedGroup?.group_metadata?.['default_expiry_days']
            ? `Default: ${selectedGroup.group_metadata['default_expiry_days']} days`
            : 'Leave empty for no expiry'
          }
          min="1"
        />
      </div>

      <div>
        <Label htmlFor="reason">Reason (optional)</Label>
        <Textarea
          id="reason"
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          placeholder="Optional reason for this assignment"
          rows={3}
        />
      </div>

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button
          type="submit"
          disabled={!selectedGroupId}
          className={adminButtonVariants({ variant: 'primary' })}
        >
          Assign to Group
        </Button>
      </DialogFooter>
    </form>
  )
}

export default GroupMembershipManager