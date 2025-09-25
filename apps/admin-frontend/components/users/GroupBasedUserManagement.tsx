/**
 * Group-Based User Management Component
 * Updated user management interface that uses the new Web3 group-based permission system
 * 
 * Features:
 * - View users with their group memberships instead of individual permissions
 * - Assign/remove users to/from permission groups
 * - Bulk group operations for multiple users
 * - Group-based filtering and search
 * - Integration with Web3 wallet auto-assignment
 * - Real-time group analytics and insights
 */

'use client'

import React, { useState, useCallback, useMemo, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { 
  Search, Filter, Plus, Download, Users, UserCheck, UserX,
  MoreHorizontal, Edit, Trash2, Shield, Calendar, Activity,
  RefreshCw, Eye, UserPlus, Settings, Zap, Globe,
  Badge as BadgeIcon, Clock, AlertTriangle, CheckCircle
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter 
} from '@/components/ui/dialog'
import { 
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { useToast } from '@/components/ui/use-toast'

import { 
  usePermissionGroups,
  useGroupAnalytics,
  useAdminGroupPermissions,
  useWeb3AssignmentRules
} from '@/hooks/useGroupPermissions'
import { GroupMembershipManager } from '@/components/groups/GroupMembershipManager'
import { groupManagementClient } from '@/lib/api/group-management-client'
import type { User, UserStats, UserFilters } from '@/types/core'
import { adminCardVariants, adminButtonVariants } from '@/design-system'
import { cn } from '@/lib/shared'

interface GroupBasedUserManagementProps {
  initialUsers?: User[]
  onUserCreate?: (user: User) => void
  onUserUpdate?: (user: User) => void
  onUserDelete?: (userId: string) => void
  onBulkAction?: (action: string, userIds: string[]) => void
  className?: string
}

interface UserWithGroups extends User {
  groups?: Array<{
    id: string
    name: string
    is_system_group: boolean
    expires_at?: string
    is_active: boolean
  }>
  group_count?: number
  effective_permissions?: string[]
}

export function GroupBasedUserManagement({
  initialUsers = [],
  onUserCreate,
  onUserUpdate,
  onUserDelete,
  onBulkAction,
  className
}: GroupBasedUserManagementProps) {
  const router = useRouter()
  const { toast } = useToast()
  
  // State
  const [users, setUsers] = useState<UserWithGroups[]>([])
  const [loading, setLoading] = useState(false)
  const [searchTerm, setSearchTerm] = useState('')
  const [filterGroup, setFilterGroup] = useState<string>('all')
  const [filterStatus, setFilterStatus] = useState<string>('all')
  const [selectedUsers, setSelectedUsers] = useState<string[]>([])
  const [selectedUser, setSelectedUser] = useState<UserWithGroups | null>(null)
  const [showUserDetails, setShowUserDetails] = useState(false)
  const [showBulkAssign, setShowBulkAssign] = useState(false)

  // Hooks
  const { groups, systemGroups } = usePermissionGroups()
  const { stats } = useGroupAnalytics()
  const { canManageUsers, canManageGroups } = useAdminGroupPermissions()
  const { processWallet } = useWeb3AssignmentRules()

  // Load users with group information
  const loadUsersWithGroups = useCallback(async () => {
    setLoading(true)
    try {
      // Enhance users with group information
      const usersWithGroups = await Promise.all(
        initialUsers.map(async (user) => {
          try {
            const userGroups = await groupManagementClient.getUserGroups(user.id)
            const userPermissions = await groupManagementClient.getUserPermissions(user.id)
            
            return {
              ...user,
              groups: userGroups.map(membership => ({
                id: membership.group_id,
                name: membership.group?.name || 'Unknown Group',
                is_system_group: membership.group?.is_system_group || false,
                expires_at: membership.expires_at,
                is_active: membership.is_active
              })),
              group_count: userGroups.filter(m => m.is_active).length,
              effective_permissions: userPermissions
            } as UserWithGroups
          } catch (error) {
            console.warn(`Failed to load groups for user ${user.id}:`, error)
            return {
              ...user,
              groups: [],
              group_count: 0,
              effective_permissions: []
            } as UserWithGroups
          }
        })
      )
      
      setUsers(usersWithGroups)
    } catch (error) {
      toast({
        title: 'Load Failed',
        description: 'Failed to load user group information',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [initialUsers, toast])

  // Load users on mount and when initialUsers changes
  useEffect(() => {
    loadUsersWithGroups()
  }, [loadUsersWithGroups])

  // Filter users
  const filteredUsers = useMemo(() => {
    let filtered = users

    // Filter by search term
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase()
      filtered = filtered.filter(user => 
        user.email?.toLowerCase().includes(searchLower) ||
        user.displayName?.toLowerCase().includes(searchLower) ||
        user.groups?.some(group => group.name.toLowerCase().includes(searchLower))
      )
    }

    // Filter by group
    if (filterGroup !== 'all') {
      if (filterGroup === 'no_groups') {
        filtered = filtered.filter(user => !user.group_count || user.group_count === 0)
      } else {
        filtered = filtered.filter(user => 
          user.groups?.some(group => group.id === filterGroup)
        )
      }
    }

    // Filter by status
    if (filterStatus === 'active') {
      filtered = filtered.filter(user => user.status === 'active')
    } else if (filterStatus === 'inactive') {
      filtered = filtered.filter(user => user.status !== 'active')
    } else if (filterStatus === 'expiring') {
      const now = Date.now()
      const sevenDaysFromNow = now + (7 * 24 * 60 * 60 * 1000)
      filtered = filtered.filter(user => 
        user.groups?.some(group => 
          group.expires_at && new Date(group.expires_at).getTime() <= sevenDaysFromNow
        )
      )
    }

    return filtered.sort((a, b) => b.group_count! - a.group_count!)
  }, [users, searchTerm, filterGroup, filterStatus])

  // User stats
  const userStats = useMemo(() => {
    const totalUsers = users.length
    const activeUsers = users.filter(u => u.status === 'active').length
    const usersWithGroups = users.filter(u => u.group_count! > 0).length
    const adminUsers = users.filter(u => 
      u.groups?.some(g => g.name.toLowerCase().includes('admin'))
    ).length

    return {
      totalUsers,
      activeUsers,
      usersWithGroups,
      adminUsers,
      usersWithoutGroups: totalUsers - usersWithGroups
    }
  }, [users])

  // Event handlers
  const handleUserSelect = useCallback((userId: string) => {
    setSelectedUsers(prev => 
      prev.includes(userId) 
        ? prev.filter(id => id !== userId)
        : [...prev, userId]
    )
  }, [])

  const handleSelectAll = useCallback(() => {
    setSelectedUsers(
      selectedUsers.length === filteredUsers.length 
        ? [] 
        : filteredUsers.map(u => u.id)
    )
  }, [selectedUsers.length, filteredUsers])

  const handleViewUser = useCallback((user: UserWithGroups) => {
    setSelectedUser(user)
    setShowUserDetails(true)
  }, [])

  const handleProcessWeb3Wallet = useCallback(async (user: UserWithGroups) => {
    if (!user.wallet_address) {
      toast({
        title: 'No Wallet Address',
        description: 'User does not have a connected wallet address',
        variant: 'destructive'
      })
      return
    }

    try {
      const assignedGroups = await processWallet(user.wallet_address)
      await loadUsersWithGroups() // Refresh user data
      toast({
        title: 'Web3 Processing Complete',
        description: `Assigned to ${assignedGroups.length} group(s) based on blockchain assets`
      })
    } catch (error) {
      toast({
        title: 'Web3 Processing Failed',
        description: error instanceof Error ? error.message : 'Failed to process wallet',
        variant: 'destructive'
      })
    }
  }, [processWallet, loadUsersWithGroups, toast])

  const handleBulkGroupAssignment = useCallback(async (groupId: string, action: 'assign' | 'remove') => {
    if (selectedUsers.length === 0) return

    try {
      const promises = selectedUsers.map(userId => {
        return action === 'assign'
          ? groupManagementClient.assignUserToGroup({ user_id: userId, group_id: groupId })
          : groupManagementClient.removeUserFromGroup(userId, groupId)
      })

      await Promise.all(promises)
      await loadUsersWithGroups() // Refresh user data
      setSelectedUsers([])
      setShowBulkAssign(false)

      toast({
        title: 'Bulk Operation Complete',
        description: `${action === 'assign' ? 'Assigned' : 'Removed'} ${selectedUsers.length} user(s) ${action === 'assign' ? 'to' : 'from'} group`
      })
    } catch (error) {
      toast({
        title: 'Bulk Operation Failed',
        description: error instanceof Error ? error.message : `Failed to ${action} users`,
        variant: 'destructive'
      })
    }
  }, [selectedUsers, loadUsersWithGroups, toast])

  const getGroupBadgeVariant = (group: { id: string; name: string; is_system_group: boolean; expires_at?: string; is_active: boolean; }) => {
    if (!group.is_active) return 'destructive'
    if (group.is_system_group) return 'default'
    return 'secondary'
  }

  const getUserStatusIcon = (user: UserWithGroups) => {
    if (user.status === 'active') return <CheckCircle className="h-4 w-4 text-green-600" />
    if (user.status === 'suspended') return <UserX className="h-4 w-4 text-red-600" />
    return <UserCheck className="h-4 w-4 text-gray-600" />
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">User Management</h1>
          <p className="text-sm text-gray-600">
            Manage users and their permission group memberships
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={loadUsersWithGroups} disabled={loading}>
            <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
            Refresh
          </Button>
          {canManageUsers && (
            <Button onClick={() => router.push('/users/create')}>
              <UserPlus className="h-4 w-4 mr-2" />
              Add User
            </Button>
          )}
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Users</p>
                <p className="text-2xl font-bold">{userStats.totalUsers}</p>
              </div>
              <Users className="h-6 w-6 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active</p>
                <p className="text-2xl font-bold text-green-600">{userStats.activeUsers}</p>
              </div>
              <CheckCircle className="h-6 w-6 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">With Groups</p>
                <p className="text-2xl font-bold text-purple-600">{userStats.usersWithGroups}</p>
              </div>
              <BadgeIcon className="h-6 w-6 text-purple-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">No Groups</p>
                <p className="text-2xl font-bold text-orange-600">{userStats.usersWithoutGroups}</p>
              </div>
              <AlertTriangle className="h-6 w-6 text-orange-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Admins</p>
                <p className="text-2xl font-bold text-red-600">{userStats.adminUsers}</p>
              </div>
              <Shield className="h-6 w-6 text-red-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters and Search */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
          <Input
            placeholder="Search users, emails, or groups..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-10"
          />
        </div>
        <Select value={filterGroup} onValueChange={setFilterGroup}>
          <SelectTrigger className="w-full sm:w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Groups</SelectItem>
            <SelectItem value="no_groups">No Groups</SelectItem>
            {groups.map((group) => (
              <SelectItem key={group.id} value={group.id}>
                {group.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-full sm:w-48">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="active">Active</SelectItem>
            <SelectItem value="inactive">Inactive</SelectItem>
            <SelectItem value="expiring">Expiring Groups</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Bulk Actions */}
      {selectedUsers.length > 0 && (
        <Card className="p-4 bg-blue-50 border-blue-200">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-blue-900">
              {selectedUsers.length} user(s) selected
            </span>
            <div className="flex gap-2">
              {canManageGroups && (
                <Button variant="outline" size="sm" onClick={() => setShowBulkAssign(true)}>
                  <BadgeIcon className="h-4 w-4 mr-2" />
                  Bulk Assign Groups
                </Button>
              )}
              <Button variant="outline" size="sm" onClick={() => setSelectedUsers([])}>
                Clear Selection
              </Button>
            </div>
          </div>
        </Card>
      )}

      {/* Users Table */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Users className="h-5 w-5" />
              Users ({filteredUsers.length})
            </CardTitle>
            <Checkbox
              checked={selectedUsers.length === filteredUsers.length && filteredUsers.length > 0}
              onCheckedChange={handleSelectAll}
            />
          </div>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="space-y-4">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="animate-pulse flex items-center space-x-4 p-4">
                  <div className="w-4 h-4 bg-gray-200 rounded"></div>
                  <div className="flex-1 space-y-2">
                    <div className="h-4 bg-gray-200 rounded w-1/4"></div>
                    <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                  </div>
                </div>
              ))}
            </div>
          ) : filteredUsers.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No users found</h3>
              <p className="text-gray-600">
                {searchTerm || filterGroup !== 'all' || filterStatus !== 'all'
                  ? 'No users match your search criteria.'
                  : 'No users have been created yet.'
                }
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {filteredUsers.map((user) => (
                <div
                  key={user.id}
                  className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50"
                >
                  <Checkbox
                    checked={selectedUsers.includes(user.id)}
                    onCheckedChange={() => handleUserSelect(user.id)}
                    className="mr-4"
                  />
                  
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      {getUserStatusIcon(user)}
                      <div className="min-w-0">
                        <h4 className="font-medium text-gray-900 truncate">
                          {user.displayName || user.email}
                        </h4>
                        {user.displayName && (
                          <p className="text-sm text-gray-600 truncate">{user.email}</p>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2 flex-wrap">
                      <Badge variant="outline" className="text-xs">
                        {user.group_count} groups
                      </Badge>
                      {user.groups?.slice(0, 3).map((group) => (
                        <Badge 
                          key={group.id} 
                          variant={getGroupBadgeVariant(group)}
                          className="text-xs"
                        >
                          {group.name}
                        </Badge>
                      ))}
                      {user.groups && user.groups.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{user.groups.length - 3} more
                        </Badge>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    {user.wallet_address && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleProcessWeb3Wallet(user)}
                        title="Process Web3 Wallet"
                      >
                        <Zap className="h-4 w-4" />
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleViewUser(user)}
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => router.push(`/users/${user.id}/edit`)}>
                          <Edit className="h-4 w-4 mr-2" />
                          Edit User
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleViewUser(user)}>
                          <BadgeIcon className="h-4 w-4 mr-2" />
                          Manage Groups
                        </DropdownMenuItem>
                        {user.wallet_address && (
                          <DropdownMenuItem onClick={() => handleProcessWeb3Wallet(user)}>
                            <Globe className="h-4 w-4 mr-2" />
                            Process Web3 Wallet
                          </DropdownMenuItem>
                        )}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* User Details Dialog */}
      <Dialog open={showUserDetails} onOpenChange={setShowUserDetails}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {selectedUser?.displayName || selectedUser?.email} - Group Management
            </DialogTitle>
          </DialogHeader>
          {selectedUser && (
            <GroupMembershipManager
              userId={selectedUser.id}
              className="mt-4"
            />
          )}
        </DialogContent>
      </Dialog>

      {/* Bulk Group Assignment Dialog */}
      <Dialog open={showBulkAssign} onOpenChange={setShowBulkAssign}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Bulk Group Assignment</DialogTitle>
          </DialogHeader>
          <BulkGroupAssignmentForm
            userCount={selectedUsers.length}
            groups={groups}
            onAssign={(groupId) => handleBulkGroupAssignment(groupId, 'assign')}
            onRemove={(groupId) => handleBulkGroupAssignment(groupId, 'remove')}
            onCancel={() => setShowBulkAssign(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

// Bulk Group Assignment Form Component
interface BulkGroupAssignmentFormProps {
  userCount: number
  groups: any[]
  onAssign: (groupId: string) => void
  onRemove: (groupId: string) => void
  onCancel: () => void
}

function BulkGroupAssignmentForm({ 
  userCount, 
  groups, 
  onAssign, 
  onRemove, 
  onCancel 
}: BulkGroupAssignmentFormProps) {
  const [selectedGroup, setSelectedGroup] = useState('')
  const [action, setAction] = useState<'assign' | 'remove'>('assign')

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault()
    if (!selectedGroup) return

    if (action === 'assign') {
      onAssign(selectedGroup)
    } else {
      onRemove(selectedGroup)
    }
  }, [selectedGroup, action, onAssign, onRemove])

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Alert>
        <Users className="h-4 w-4" />
        <AlertDescription>
          This action will affect {userCount} selected user(s).
        </AlertDescription>
      </Alert>

      <div>
        <label className="block text-sm font-medium mb-2">Action</label>
        <Select value={action} onValueChange={(value: 'assign' | 'remove') => setAction(value)}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="assign">Assign to Group</SelectItem>
            <SelectItem value="remove">Remove from Group</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Permission Group</label>
        <Select value={selectedGroup} onValueChange={setSelectedGroup}>
          <SelectTrigger>
            <SelectValue placeholder="Select a group" />
          </SelectTrigger>
          <SelectContent>
            {groups.map((group) => (
              <SelectItem key={group.id} value={group.id}>
                <div className="flex items-center gap-2">
                  {group.is_system_group ? (
                    <Shield className="h-3 w-3 text-yellow-600" />
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

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button 
          type="submit" 
          disabled={!selectedGroup}
          variant={action === 'remove' ? 'destructive' : 'default'}
        >
          {action === 'assign' ? 'Assign to Group' : 'Remove from Group'}
        </Button>
      </DialogFooter>
    </form>
  )
}

export default GroupBasedUserManagement