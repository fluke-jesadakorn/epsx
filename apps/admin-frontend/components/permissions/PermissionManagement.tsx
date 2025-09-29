/**
 * Permission Management Component
 * Main dashboard for managing user permissions, groups, and inheritance
 */

'use client'

import React, { useState, useCallback, useMemo } from 'react'
import { 
  Shield, Users, Settings, Search, Filter, Plus, Edit, Trash2,
  AlertTriangle, CheckCircle, Clock, Star, Key, GitBranch, Activity
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useToast } from '@/components/ui/use-toast'
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger 
} from '@/components/ui/dialog'

import { adminCardVariants, adminButtonVariants } from '@/design-system'
import { cn } from '@/lib/shared'

export interface UserPermissionSummary {
  id: string;
  name: string;
  email: string;
  walletAddress?: string;
  permissions: string[];
  groups: string[];
  lastActive: string;
  status: 'active' | 'inactive' | 'suspended';
  riskScore: number;
  hasExpiredPermissions: boolean;
}

export interface PermissionGroup {
  id: string;
  name: string;
  description?: string;
  permissions: string[];
  userCount: number;
  isSystemGroup: boolean;
  priority: number;
}

export interface PermissionAuditLog {
  id: string;
  action: 'grant' | 'revoke' | 'modify' | 'expire';
  permission: string;
  userId: string;
  userName: string;
  performedBy: string;
  timestamp: string;
  reason?: string;
  metadata?: Record<string, any>;
}

interface PermissionManagementProps {
  users?: UserPermissionSummary[];
  groups?: PermissionGroup[];
  auditLog?: PermissionAuditLog[];
  onUserPermissionChange?: (userId: string, permissions: string[]) => Promise<void>;
  onGroupChange?: (group: PermissionGroup) => Promise<void>;
  className?: string;
}

export function PermissionManagement({ 
  users = [], 
  groups = [], 
  auditLog = [],
  onUserPermissionChange,
  onGroupChange,
  className 
}: PermissionManagementProps) {
  const { toast } = useToast()
  const [activeTab, setActiveTab] = useState('users')
  const [searchTerm, setSearchTerm] = useState('')
  const [filterStatus, setFilterStatus] = useState('all')
  const [selectedUser, setSelectedUser] = useState<UserPermissionSummary | null>(null)
  const [showUserDetails, setShowUserDetails] = useState(false)

  // Filter users based on search and status
  const filteredUsers = useMemo(() => {
    return users.filter(user => {
      const matchesSearch = !searchTerm || 
        user.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
        user.walletAddress?.toLowerCase().includes(searchTerm.toLowerCase())
      
      const matchesStatus = filterStatus === 'all' || user.status === filterStatus
      
      return matchesSearch && matchesStatus
    })
  }, [users, searchTerm, filterStatus])

  // Permission statistics
  const permissionStats = useMemo(() => {
    const totalUsers = users.length
    const activeUsers = users.filter(u => u.status === 'active').length
    const usersWithExpiredPerms = users.filter(u => u.hasExpiredPermissions).length
    const highRiskUsers = users.filter(u => u.riskScore > 70).length
    
    const allPermissions = new Set<string>()
    users.forEach(user => {
      user.permissions.forEach(perm => allPermissions.add(perm))
    })

    return {
      totalUsers,
      activeUsers,
      totalPermissions: allPermissions.size,
      totalGroups: groups.length,
      usersWithExpiredPerms,
      highRiskUsers
    }
  }, [users, groups])

  const handleUserClick = useCallback((user: UserPermissionSummary) => {
    setSelectedUser(user)
    setShowUserDetails(true)
  }, [])

  const getRiskBadgeVariant = (riskScore: number) => {
    if (riskScore >= 80) return 'destructive'
    if (riskScore >= 60) return 'secondary'
    return 'outline'
  }

  const getRiskLabel = (riskScore: number) => {
    if (riskScore >= 80) return 'High Risk'
    if (riskScore >= 60) return 'Medium Risk'
    if (riskScore >= 40) return 'Low Risk'
    return 'Minimal Risk'
  }

  const getStatusBadgeVariant = (status: UserPermissionSummary['status']) => {
    switch (status) {
      case 'active': return 'default'
      case 'inactive': return 'secondary'
      case 'suspended': return 'destructive'
      default: return 'outline'
    }
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">Permission Management</h1>
          <p className="text-sm text-gray-600 mt-1">
            Manage user permissions, groups, and access control
          </p>
        </div>
        <div className="flex gap-2">
          <Button 
            className={adminButtonVariants({ variant: 'secondary', size: 'sm' })}
          >
            <Settings className="h-4 w-4 mr-2" />
            Settings
          </Button>
          <Button 
            className={adminButtonVariants({ variant: 'primary', size: 'sm' })}
          >
            <Plus className="h-4 w-4 mr-2" />
            Add User
          </Button>
        </div>
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Users</p>
                <p className="text-2xl font-bold text-gray-900">{permissionStats.totalUsers}</p>
              </div>
              <Users className="h-8 w-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Users</p>
                <p className="text-2xl font-bold text-gray-900">{permissionStats.activeUsers}</p>
              </div>
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Permissions</p>
                <p className="text-2xl font-bold text-gray-900">{permissionStats.totalPermissions}</p>
              </div>
              <Key className="h-8 w-8 text-purple-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Groups</p>
                <p className="text-2xl font-bold text-gray-900">{permissionStats.totalGroups}</p>
              </div>
              <Shield className="h-8 w-8 text-indigo-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Expired Perms</p>
                <p className="text-2xl font-bold text-gray-900">{permissionStats.usersWithExpiredPerms}</p>
              </div>
              <Clock className="h-8 w-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">High Risk</p>
                <p className="text-2xl font-bold text-gray-900">{permissionStats.highRiskUsers}</p>
              </div>
              <AlertTriangle className="h-8 w-8 text-red-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Alerts */}
      {permissionStats.usersWithExpiredPerms > 0 && (
        <Alert>
          <Clock className="h-4 w-4" />
          <AlertDescription>
            {permissionStats.usersWithExpiredPerms} users have expired permissions that need attention.
          </AlertDescription>
        </Alert>
      )}

      {permissionStats.highRiskUsers > 0 && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {permissionStats.highRiskUsers} users have high risk scores and should be reviewed.
          </AlertDescription>
        </Alert>
      )}

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="users">Users</TabsTrigger>
          <TabsTrigger value="groups">Groups</TabsTrigger>
          <TabsTrigger value="permissions">Permissions</TabsTrigger>
          <TabsTrigger value="audit">Audit Log</TabsTrigger>
        </TabsList>

        <TabsContent value="users" className="space-y-4">
          {/* Search and Filters */}
          <div className="flex flex-col sm:flex-row gap-4">
            <div className="relative flex-1">
              <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
              <Input
                placeholder="Search users by name, email, or wallet address..."
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
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="active">Active</SelectItem>
                <SelectItem value="inactive">Inactive</SelectItem>
                <SelectItem value="suspended">Suspended</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Users List */}
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
            {filteredUsers.map((user) => (
              <Card 
                key={user.id} 
                className={cn(
                  adminCardVariants({ variant: 'default' }),
                  'cursor-pointer hover:shadow-md transition-shadow'
                )}
                onClick={() => handleUserClick(user)}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle className="text-base">{user.name}</CardTitle>
                      <p className="text-sm text-gray-600">{user.email}</p>
                      {user.walletAddress && (
                        <p className="text-xs text-gray-500 font-mono">
                          {user.walletAddress.slice(0, 6)}...{user.walletAddress.slice(-4)}
                        </p>
                      )}
                    </div>
                    <div className="flex flex-col gap-1 items-end">
                      <Badge variant={getStatusBadgeVariant(user.status)}>
                        {user.status}
                      </Badge>
                      <Badge variant={getRiskBadgeVariant(user.riskScore)}>
                        {getRiskLabel(user.riskScore)}
                      </Badge>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Permissions</span>
                    <Badge variant="outline">{user.permissions.length}</Badge>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Groups</span>
                    <Badge variant="outline">{user.groups.length}</Badge>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Last Active</span>
                    <span className="text-xs text-gray-500">
                      {new Date(user.lastActive).toLocaleDateString()}
                    </span>
                  </div>

                  {user.hasExpiredPermissions && (
                    <div className="flex items-center gap-2 p-2 bg-orange-50 border border-orange-200 rounded">
                      <Clock className="h-4 w-4 text-orange-600" />
                      <span className="text-sm text-orange-700">Has expired permissions</span>
                    </div>
                  )}

                  <div className="pt-2 border-t border-gray-100">
                    <div className="flex flex-wrap gap-1">
                      {user.permissions.slice(0, 3).map((permission, index) => (
                        <Badge key={index} variant="outline" className="text-xs">
                          {permission.split(':')[1] || permission}
                        </Badge>
                      ))}
                      {user.permissions.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{user.permissions.length - 3} more
                        </Badge>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>

          {filteredUsers.length === 0 && (
            <Card className="p-8 text-center">
              <Users className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No users found</h3>
              <p className="text-gray-600">
                No users match your search criteria. Try adjusting your filters.
              </p>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="groups" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
            {groups.map((group) => (
              <Card key={group.id} className={adminCardVariants({ variant: 'default' })}>
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-2">
                      {group.isSystemGroup ? (
                        <Star className="h-4 w-4 text-yellow-600" />
                      ) : (
                        <Shield className="h-4 w-4 text-blue-600" />
                      )}
                      <CardTitle className="text-base">{group.name}</CardTitle>
                    </div>
                    <Badge variant={group.isSystemGroup ? 'default' : 'secondary'}>
                      {group.isSystemGroup ? 'System' : 'Custom'}
                    </Badge>
                  </div>
                  {group.description && (
                    <p className="text-sm text-gray-600">{group.description}</p>
                  )}
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Members</span>
                    <Badge variant="outline">{group.userCount}</Badge>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Permissions</span>
                    <Badge variant="outline">{group.permissions.length}</Badge>
                  </div>
                  
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Priority</span>
                    <Badge variant="outline">{group.priority}</Badge>
                  </div>

                  <div className="pt-2 border-t border-gray-100">
                    <div className="flex flex-wrap gap-1">
                      {group.permissions.slice(0, 3).map((permission, index) => (
                        <Badge key={index} variant="outline" className="text-xs">
                          {permission.split(':')[1] || permission}
                        </Badge>
                      ))}
                      {group.permissions.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{group.permissions.length - 3} more
                        </Badge>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="permissions" className="space-y-4">
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle>All Permissions</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
                {Array.from(new Set(users.flatMap(u => u.permissions))).sort().map((permission, index) => (
                  <div key={index} className="p-2 bg-gray-50 border rounded text-sm">
                    {permission}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="audit" className="space-y-4">
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
            </CardHeader>
            <CardContent>
              {auditLog.length === 0 ? (
                <p className="text-sm text-gray-500 text-center py-8">No audit log entries</p>
              ) : (
                <div className="space-y-3">
                  {auditLog.slice(0, 20).map((entry) => (
                    <div key={entry.id} className="flex items-center justify-between p-3 bg-gray-50 rounded">
                      <div className="flex items-center gap-3">
                        <Activity className="h-4 w-4 text-blue-600" />
                        <div>
                          <p className="text-sm font-medium">
                            {entry.action.toUpperCase()} {entry.permission}
                          </p>
                          <p className="text-xs text-gray-600">
                            {entry.userName} by {entry.performedBy}
                          </p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="text-xs text-gray-500">
                          {new Date(entry.timestamp).toLocaleDateString()}
                        </p>
                        <p className="text-xs text-gray-500">
                          {new Date(entry.timestamp).toLocaleTimeString()}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* User Details Dialog */}
      <Dialog open={showUserDetails} onOpenChange={setShowUserDetails}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>
              {selectedUser ? `${selectedUser.name} - Permission Details` : 'User Details'}
            </DialogTitle>
          </DialogHeader>
          {selectedUser && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm font-medium text-gray-600">Email</p>
                  <p className="text-sm">{selectedUser.email}</p>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-600">Status</p>
                  <Badge variant={getStatusBadgeVariant(selectedUser.status)}>
                    {selectedUser.status}
                  </Badge>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-600">Risk Score</p>
                  <Badge variant={getRiskBadgeVariant(selectedUser.riskScore)}>
                    {selectedUser.riskScore} - {getRiskLabel(selectedUser.riskScore)}
                  </Badge>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-600">Last Active</p>
                  <p className="text-sm">{new Date(selectedUser.lastActive).toLocaleDateString()}</p>
                </div>
              </div>

              <div>
                <p className="text-sm font-medium text-gray-600 mb-2">Permissions ({selectedUser.permissions.length})</p>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-1">
                  {selectedUser.permissions.map((permission, index) => (
                    <div key={index} className="p-2 bg-gray-50 border rounded text-sm">
                      {permission}
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <p className="text-sm font-medium text-gray-600 mb-2">Groups ({selectedUser.groups.length})</p>
                <div className="space-y-1">
                  {selectedUser.groups.map((group, index) => (
                    <Badge key={index} variant="outline" className="mr-2">
                      {group}
                    </Badge>
                  ))}
                </div>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}

export default PermissionManagement