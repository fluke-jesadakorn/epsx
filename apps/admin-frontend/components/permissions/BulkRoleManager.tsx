'use client'

import { useState, useEffect, useMemo } from 'react'
import { Plus, Search, Users, UserCheck, UserX, Download, Upload, Filter, AlertTriangle, Check, X, Eye } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Checkbox } from '@/components/ui/checkbox'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Progress } from '@/components/ui/progress'
import { Separator } from '@/components/ui/separator'

// Types
interface User {
  id: string
  email: string
  name?: string
  roles: { id: string; name: string; display_name?: string }[]
  permissions: { id: string; name: string; source: 'role' | 'direct' }[]
  last_login?: string
  created_at: string
}

interface Role {
  id: string
  name: string
  display_name?: string
  description?: string
  is_system_role: boolean
  user_count: number
  permissions: { id: string; name: string }[]
}

interface BulkOperation {
  id: string
  type: 'assign_role' | 'revoke_role' | 'assign_permission' | 'revoke_permission'
  target_users: string[]
  target_role?: string
  target_permission?: string
  reason: string
  created_by: string
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  progress: number
  created_at: string
  completed_at?: string
  error_message?: string
  results?: {
    success_count: number
    failure_count: number
    failed_users: { user_id: string; error: string }[]
  }
}

interface BulkRoleManagerProps {
  users: User[]
  roles: Role[]
  onRefreshUsers: () => Promise<void>
  onClose?: () => void
}

export default function BulkRoleManager({ users, roles, onRefreshUsers, onClose }: BulkRoleManagerProps) {
  const [activeTab, setActiveTab] = useState<'assign' | 'revoke' | 'operations'>('assign')
  const [selectedUsers, setSelectedUsers] = useState<string[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [roleFilter, setRoleFilter] = useState('all')
  const [bulkOperations, setBulkOperations] = useState<BulkOperation[]>([])
  const [loading, setLoading] = useState(false)

  // Dialog states
  const [isAssignDialogOpen, setIsAssignDialogOpen] = useState(false)
  const [isRevokeDialogOpen, setIsRevokeDialogOpen] = useState(false)

  // Form states
  const [assignForm, setAssignForm] = useState({
    roleId: '',
    reason: '',
    notifyUsers: false,
    scheduleFor: ''
  })

  const [revokeForm, setRevokeForm] = useState({
    roleId: '',
    reason: '',
    notifyUsers: false,
    scheduleFor: ''
  })

  // Mock API functions - replace with real API calls
  const mockBulkAssignRole = async (userIds: string[], roleId: string, reason: string, options?: any): Promise<BulkOperation> => {
    const operationId = Math.random().toString(36).substring(7)
    return {
      id: operationId,
      type: 'assign_role',
      target_users: userIds,
      target_role: roleId,
      reason,
      created_by: 'current-admin',
      status: 'pending',
      progress: 0,
      created_at: new Date().toISOString()
    }
  }

  const mockBulkRevokeRole = async (userIds: string[], roleId: string, reason: string, options?: any): Promise<BulkOperation> => {
    const operationId = Math.random().toString(36).substring(7)
    return {
      id: operationId,
      type: 'revoke_role',
      target_users: userIds,
      target_role: roleId,
      reason,
      created_by: 'current-admin',
      status: 'pending',
      progress: 0,
      created_at: new Date().toISOString()
    }
  }

  const mockGetBulkOperations = async (): Promise<BulkOperation[]> => {
    return [
      {
        id: 'op1',
        type: 'assign_role',
        target_users: ['user1', 'user2', 'user3'],
        target_role: 'analytics_user',
        reason: 'Quarterly access review - granting analytics access',
        created_by: 'admin@epsx.io',
        status: 'completed',
        progress: 100,
        created_at: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
        completed_at: new Date(Date.now() - 1000 * 60 * 25).toISOString(),
        results: {
          success_count: 3,
          failure_count: 0,
          failed_users: []
        }
      },
      {
        id: 'op2',
        type: 'revoke_role',
        target_users: ['user4', 'user5'],
        target_role: 'premium_user',
        reason: 'Access cleanup - removing unused premium access',
        created_by: 'admin@epsx.io',
        status: 'in_progress',
        progress: 50,
        created_at: new Date(Date.now() - 1000 * 60 * 10).toISOString()
      }
    ]
  }

  useEffect(() => {
    loadBulkOperations()
  }, [])

  const loadBulkOperations = async () => {
    try {
      const operations = await mockGetBulkOperations()
      setBulkOperations(operations)
    } catch (error) {
      console.error('Failed to load bulk operations:', error)
    }
  }

  // Filter users
  const filteredUsers = useMemo(() => {
    return users.filter(user => {
      const matchesSearch = user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           user.name?.toLowerCase().includes(searchTerm.toLowerCase())
      
      const matchesRole = roleFilter === 'all' ||
                         (roleFilter === 'no_roles' && user.roles.length === 0) ||
                         user.roles.some(role => role.id === roleFilter)
                         
      return matchesSearch && matchesRole
    })
  }, [users, searchTerm, roleFilter])

  // Selection helpers
  const isAllSelected = filteredUsers.length > 0 && selectedUsers.length === filteredUsers.length
  const isPartiallySelected = selectedUsers.length > 0 && selectedUsers.length < filteredUsers.length

  const handleSelectAll = () => {
    if (isAllSelected) {
      setSelectedUsers([])
    } else {
      setSelectedUsers(filteredUsers.map(user => user.id))
    }
  }

  const handleUserSelect = (userId: string) => {
    setSelectedUsers(prev => 
      prev.includes(userId) 
        ? prev.filter(id => id !== userId)
        : [...prev, userId]
    )
  }

  const handleBulkAssign = async () => {
    if (!assignForm.roleId || !assignForm.reason || selectedUsers.length === 0) return

    try {
      setLoading(true)
      const operation = await mockBulkAssignRole(
        selectedUsers, 
        assignForm.roleId, 
        assignForm.reason,
        {
          notify_users: assignForm.notifyUsers,
          schedule_for: assignForm.scheduleFor || undefined
        }
      )
      
      setBulkOperations(prev => [operation, ...prev])
      setIsAssignDialogOpen(false)
      setAssignForm({ roleId: '', reason: '', notifyUsers: false, scheduleFor: '' })
      setSelectedUsers([])
      
      // Start polling for operation status (in real implementation)
      console.log('Started bulk assign operation:', operation.id)
      
    } catch (error) {
      console.error('Failed to start bulk assign:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleBulkRevoke = async () => {
    if (!revokeForm.roleId || !revokeForm.reason || selectedUsers.length === 0) return

    try {
      setLoading(true)
      const operation = await mockBulkRevokeRole(
        selectedUsers,
        revokeForm.roleId,
        revokeForm.reason,
        {
          notify_users: revokeForm.notifyUsers,
          schedule_for: revokeForm.scheduleFor || undefined
        }
      )
      
      setBulkOperations(prev => [operation, ...prev])
      setIsRevokeDialogOpen(false)
      setRevokeForm({ roleId: '', reason: '', notifyUsers: false, scheduleFor: '' })
      setSelectedUsers([])
      
      console.log('Started bulk revoke operation:', operation.id)
      
    } catch (error) {
      console.error('Failed to start bulk revoke:', error)
    } finally {
      setLoading(false)
    }
  }

  // Get roles that can be assigned to selected users
  const assignableRoles = useMemo(() => {
    if (selectedUsers.length === 0) return roles

    const selectedUserData = users.filter(user => selectedUsers.includes(user.id))
    return roles.filter(role => {
      // Check if any selected user doesn't already have this role
      return selectedUserData.some(user => !user.roles.some(userRole => userRole.id === role.id))
    })
  }, [selectedUsers, users, roles])

  // Get roles that can be revoked from selected users
  const revokableRoles = useMemo(() => {
    if (selectedUsers.length === 0) return []

    const selectedUserData = users.filter(user => selectedUsers.includes(user.id))
    const commonRoles = roles.filter(role => {
      // Check if all selected users have this role
      return selectedUserData.every(user => user.roles.some(userRole => userRole.id === role.id))
    })
    
    return commonRoles
  }, [selectedUsers, users, roles])

  const getOperationStatusColor = (status: BulkOperation['status']) => {
    switch (status) {
      case 'pending': return 'text-yellow-600 bg-yellow-100'
      case 'in_progress': return 'text-blue-600 bg-blue-100'
      case 'completed': return 'text-green-600 bg-green-100'
      case 'failed': return 'text-red-600 bg-red-100'
      default: return 'text-gray-600 bg-gray-100'
    }
  }

  const getOperationIcon = (type: BulkOperation['type']) => {
    switch (type) {
      case 'assign_role': return <UserCheck className="w-4 h-4" />
      case 'revoke_role': return <UserX className="w-4 h-4" />
      default: return <Users className="w-4 h-4" />
    }
  }

  const exportSelectedUsers = () => {
    const selectedUserData = users.filter(user => selectedUsers.includes(user.id))
    const csvData = selectedUserData.map(user => ({
      email: user.email,
      name: user.name || '',
      roles: user.roles.map(r => r.display_name || r.name).join('; '),
      permissions_count: user.permissions.length,
      last_login: user.last_login || 'Never'
    }))

    // Create CSV content
    const headers = ['Email', 'Name', 'Roles', 'Permissions Count', 'Last Login']
    const csvContent = [
      headers.join(','),
      ...csvData.map(row => [
        `"${row.email}"`,
        `"${row.name}"`,
        `"${row.roles}"`,
        row.permissions_count,
        `"${row.last_login}"`
      ].join(','))
    ].join('\n')

    // Download file
    const blob = new Blob([csvContent], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `selected-users-${new Date().toISOString().split('T')[0]}.csv`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <h2 className="text-2xl font-bold">Bulk Role Management</h2>
          <p className="text-muted-foreground">Assign or revoke roles for multiple users simultaneously</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportSelectedUsers} disabled={selectedUsers.length === 0}>
            <Download className="w-4 h-4 mr-2" />
            Export Selected
          </Button>
          {onClose && (
            <Button variant="ghost" onClick={onClose}>
              Close
            </Button>
          )}
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Users</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{filteredUsers.length}</div>
            <p className="text-xs text-muted-foreground">
              {selectedUsers.length} selected
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Available Roles</CardTitle>
            <UserCheck className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{assignableRoles.length}</div>
            <p className="text-xs text-muted-foreground">
              Can be assigned to selection
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Revokable Roles</CardTitle>
            <UserX className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{revokableRoles.length}</div>
            <p className="text-xs text-muted-foreground">
              Common to all selected
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Operations</CardTitle>
            <Eye className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{bulkOperations.length}</div>
            <p className="text-xs text-muted-foreground">
              {bulkOperations.filter(op => op.status === 'in_progress').length} in progress
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Main Tabs */}
      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as any)}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="assign" className="flex items-center space-x-2">
            <UserCheck className="w-4 h-4" />
            <span>Assign Roles</span>
          </TabsTrigger>
          <TabsTrigger value="revoke" className="flex items-center space-x-2">
            <UserX className="w-4 h-4" />
            <span>Revoke Roles</span>
          </TabsTrigger>
          <TabsTrigger value="operations" className="flex items-center space-x-2">
            <Eye className="w-4 h-4" />
            <span>Operations</span>
          </TabsTrigger>
        </TabsList>

        {/* User Selection Common to Assign/Revoke tabs */}
        {(activeTab === 'assign' || activeTab === 'revoke') && (
          <>
            {/* Search and Filters */}
            <div className="flex gap-4 items-center">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground h-4 w-4" />
                <Input
                  placeholder="Search users..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
              <Select value={roleFilter} onValueChange={setRoleFilter}>
                <SelectTrigger className="w-48">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Users</SelectItem>
                  <SelectItem value="no_roles">No Roles</SelectItem>
                  <Separator />
                  {roles.map(role => (
                    <SelectItem key={role.id} value={role.id}>
                      {role.display_name || role.name} ({role.user_count})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Selection Actions */}
            <div className="flex justify-between items-center p-4 border rounded-lg bg-muted/50">
              <div className="flex items-center space-x-4">
                <Checkbox
                  checked={isAllSelected}
                  ref={(el) => {
                    if (el) el.indeterminate = isPartiallySelected
                  }}
                  onCheckedChange={handleSelectAll}
                />
                <span className="text-sm font-medium">
                  {selectedUsers.length === 0 
                    ? 'Select users to perform bulk operations'
                    : `${selectedUsers.length} user${selectedUsers.length === 1 ? '' : 's'} selected`
                  }
                </span>
              </div>
              
              <div className="flex gap-2">
                {activeTab === 'assign' && (
                  <Dialog open={isAssignDialogOpen} onOpenChange={setIsAssignDialogOpen}>
                    <DialogTrigger asChild>
                      <Button disabled={selectedUsers.length === 0 || assignableRoles.length === 0}>
                        <Plus className="w-4 h-4 mr-2" />
                        Assign Role
                      </Button>
                    </DialogTrigger>
                    <DialogContent>
                      <DialogHeader>
                        <DialogTitle>Bulk Assign Role</DialogTitle>
                        <DialogDescription>
                          Assign a role to {selectedUsers.length} selected user{selectedUsers.length === 1 ? '' : 's'}
                        </DialogDescription>
                      </DialogHeader>
                      <div className="space-y-4">
                        <div>
                          <Label htmlFor="assign-role">Select Role</Label>
                          <Select value={assignForm.roleId} onValueChange={(value) => 
                            setAssignForm(prev => ({ ...prev, roleId: value }))
                          }>
                            <SelectTrigger>
                              <SelectValue placeholder="Choose a role..." />
                            </SelectTrigger>
                            <SelectContent>
                              {assignableRoles.map(role => (
                                <SelectItem key={role.id} value={role.id}>
                                  <div className="space-y-1">
                                    <div className="font-medium">{role.display_name || role.name}</div>
                                    {role.description && (
                                      <div className="text-xs text-muted-foreground">{role.description}</div>
                                    )}
                                  </div>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        <div>
                          <Label htmlFor="assign-reason">Reason</Label>
                          <Textarea
                            id="assign-reason"
                            value={assignForm.reason}
                            onChange={(e) => setAssignForm(prev => ({ ...prev, reason: e.target.value }))}
                            placeholder="Explain why you're assigning this role..."
                            rows={3}
                          />
                        </div>
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id="assign-notify"
                            checked={assignForm.notifyUsers}
                            onCheckedChange={(checked) => 
                              setAssignForm(prev => ({ ...prev, notifyUsers: checked as boolean }))
                            }
                          />
                          <Label htmlFor="assign-notify">Notify users via email</Label>
                        </div>
                        <div>
                          <Label htmlFor="assign-schedule">Schedule For (Optional)</Label>
                          <Input
                            id="assign-schedule"
                            type="datetime-local"
                            value={assignForm.scheduleFor}
                            onChange={(e) => setAssignForm(prev => ({ ...prev, scheduleFor: e.target.value }))}
                          />
                        </div>
                      </div>
                      <DialogFooter>
                        <Button variant="outline" onClick={() => setIsAssignDialogOpen(false)}>
                          Cancel
                        </Button>
                        <Button 
                          onClick={handleBulkAssign}
                          disabled={!assignForm.roleId || !assignForm.reason || loading}
                        >
                          {loading ? 'Processing...' : 'Assign Role'}
                        </Button>
                      </DialogFooter>
                    </DialogContent>
                  </Dialog>
                )}

                {activeTab === 'revoke' && (
                  <Dialog open={isRevokeDialogOpen} onOpenChange={setIsRevokeDialogOpen}>
                    <DialogTrigger asChild>
                      <Button 
                        variant="destructive" 
                        disabled={selectedUsers.length === 0 || revokableRoles.length === 0}
                      >
                        <UserX className="w-4 h-4 mr-2" />
                        Revoke Role
                      </Button>
                    </DialogTrigger>
                    <DialogContent>
                      <DialogHeader>
                        <DialogTitle>Bulk Revoke Role</DialogTitle>
                        <DialogDescription>
                          Revoke a role from {selectedUsers.length} selected user{selectedUsers.length === 1 ? '' : 's'}
                        </DialogDescription>
                      </DialogHeader>
                      <Alert>
                        <AlertTriangle className="h-4 w-4" />
                        <AlertDescription>
                          This will remove the selected role from all selected users. This action cannot be undone.
                        </AlertDescription>
                      </Alert>
                      <div className="space-y-4">
                        <div>
                          <Label htmlFor="revoke-role">Select Role to Revoke</Label>
                          <Select value={revokeForm.roleId} onValueChange={(value) => 
                            setRevokeForm(prev => ({ ...prev, roleId: value }))
                          }>
                            <SelectTrigger>
                              <SelectValue placeholder="Choose a role to revoke..." />
                            </SelectTrigger>
                            <SelectContent>
                              {revokableRoles.map(role => (
                                <SelectItem key={role.id} value={role.id}>
                                  <div className="space-y-1">
                                    <div className="font-medium">{role.display_name || role.name}</div>
                                    {role.description && (
                                      <div className="text-xs text-muted-foreground">{role.description}</div>
                                    )}
                                  </div>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        <div>
                          <Label htmlFor="revoke-reason">Reason</Label>
                          <Textarea
                            id="revoke-reason"
                            value={revokeForm.reason}
                            onChange={(e) => setRevokeForm(prev => ({ ...prev, reason: e.target.value }))}
                            placeholder="Explain why you're revoking this role..."
                            rows={3}
                          />
                        </div>
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id="revoke-notify"
                            checked={revokeForm.notifyUsers}
                            onCheckedChange={(checked) => 
                              setRevokeForm(prev => ({ ...prev, notifyUsers: checked as boolean }))
                            }
                          />
                          <Label htmlFor="revoke-notify">Notify users via email</Label>
                        </div>
                      </div>
                      <DialogFooter>
                        <Button variant="outline" onClick={() => setIsRevokeDialogOpen(false)}>
                          Cancel
                        </Button>
                        <Button 
                          variant="destructive"
                          onClick={handleBulkRevoke}
                          disabled={!revokeForm.roleId || !revokeForm.reason || loading}
                        >
                          {loading ? 'Processing...' : 'Revoke Role'}
                        </Button>
                      </DialogFooter>
                    </DialogContent>
                  </Dialog>
                )}

                <Button variant="ghost" onClick={() => setSelectedUsers([])}>
                  Clear Selection
                </Button>
              </div>
            </div>

            {/* User List */}
            <div className="border rounded-lg max-h-96 overflow-y-auto">
              <div className="space-y-0 divide-y">
                {filteredUsers.map(user => (
                  <div key={user.id} className="flex items-center space-x-4 p-3 hover:bg-muted/50">
                    <Checkbox
                      checked={selectedUsers.includes(user.id)}
                      onCheckedChange={() => handleUserSelect(user.id)}
                    />
                    <div className="flex-1">
                      <div className="flex items-center justify-between">
                        <div>
                          <div className="font-medium">{user.email}</div>
                          {user.name && <div className="text-sm text-muted-foreground">{user.name}</div>}
                        </div>
                        <div className="flex items-center space-x-2">
                          <Badge variant="outline">
                            {user.roles.length} roles
                          </Badge>
                          {user.last_login ? (
                            <Badge variant="secondary" className="text-xs">
                              Active {new Date(user.last_login).toLocaleDateString()}
                            </Badge>
                          ) : (
                            <Badge variant="outline" className="text-xs">
                              Never logged in
                            </Badge>
                          )}
                        </div>
                      </div>
                      {user.roles.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-2">
                          {user.roles.map(role => (
                            <Badge key={role.id} variant="secondary" className="text-xs">
                              {role.display_name || role.name}
                            </Badge>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
              
              {filteredUsers.length === 0 && (
                <div className="text-center py-8 text-muted-foreground">
                  No users found matching your criteria.
                </div>
              )}
            </div>
          </>
        )}

        {/* Operations Tab */}
        <TabsContent value="operations" className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h3 className="text-lg font-semibold">Bulk Operations History</h3>
              <p className="text-sm text-muted-foreground">Track the progress and results of bulk operations</p>
            </div>
            <Button variant="outline" onClick={loadBulkOperations}>
              <Eye className="w-4 h-4 mr-2" />
              Refresh
            </Button>
          </div>

          <div className="space-y-4">
            {bulkOperations.map(operation => (
              <Card key={operation.id}>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-base flex items-center space-x-2">
                      {getOperationIcon(operation.type)}
                      <span className="capitalize">{operation.type.replace('_', ' ')}</span>
                    </CardTitle>
                    <Badge className={getOperationStatusColor(operation.status)}>
                      {operation.status.replace('_', ' ')}
                    </Badge>
                  </div>
                  <CardDescription>
                    {operation.reason}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <div className="text-muted-foreground">Target Users</div>
                      <div className="font-medium">{operation.target_users.length}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Target Role</div>
                      <div className="font-medium">
                        {operation.target_role && roles.find(r => r.id === operation.target_role)?.display_name || operation.target_role}
                      </div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Created By</div>
                      <div className="font-medium">{operation.created_by}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground">Created</div>
                      <div className="font-medium">{new Date(operation.created_at).toLocaleString()}</div>
                    </div>
                  </div>

                  {operation.status === 'in_progress' && (
                    <div className="space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <span>Progress</span>
                        <span>{operation.progress}%</span>
                      </div>
                      <Progress value={operation.progress} className="h-2" />
                    </div>
                  )}

                  {operation.results && (
                    <div className="flex items-center space-x-4 text-sm">
                      <div className="flex items-center space-x-1 text-green-600">
                        <Check className="w-4 h-4" />
                        <span>{operation.results.success_count} successful</span>
                      </div>
                      {operation.results.failure_count > 0 && (
                        <div className="flex items-center space-x-1 text-red-600">
                          <X className="w-4 h-4" />
                          <span>{operation.results.failure_count} failed</span>
                        </div>
                      )}
                    </div>
                  )}

                  {operation.completed_at && (
                    <div className="text-sm text-muted-foreground">
                      Completed at {new Date(operation.completed_at).toLocaleString()}
                    </div>
                  )}

                  {operation.error_message && (
                    <Alert variant="destructive">
                      <AlertTriangle className="h-4 w-4" />
                      <AlertDescription>
                        {operation.error_message}
                      </AlertDescription>
                    </Alert>
                  )}
                </CardContent>
              </Card>
            ))}

            {bulkOperations.length === 0 && (
              <div className="text-center py-12">
                <Eye className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium text-muted-foreground mb-2">No operations yet</h3>
                <p className="text-sm text-muted-foreground">
                  Bulk operations will appear here once you start assigning or revoking roles.
                </p>
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}