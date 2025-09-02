'use client'

/**
 * MIGRATED: User Management Component
 * Shows migration from legacy pattern to hybrid data strategy
 * 
 * BEFORE: Manual loading, AdminApiService calls, manual state management
 * AFTER: SWR hooks, OIDC authentication, real-time updates, serverless optimized
 */

import { adminClientData, type AdminFilters } from '@/lib/admin-client-data'
import { adminServerData } from '@/lib/admin-server-data'
import type { AdminUser } from '@/services/adminApiService'
import { USER_LEVEL_CONFIGS, UserLevel } from '@/types/admin/userLevels'
import {
  AlertTriangle,
  CheckCircle,
  Crown,
  History,
  RefreshCw,
  Search,
  Star,
  Users,
  UserX,
  Loader2,
} from 'lucide-react'
import { useState, useTransition } from 'react'

import { useToast } from '@/components/ui/toast'
import { _Card, _CardContent, _CardDescription, _CardHeader, _CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'

interface UserManagementMigratedProps {
  initialFilters?: AdminFilters
}

/**
 * MIGRATED: User Management using Hybrid Data Strategy
 * 
 * Key Changes:
 * ✅ Replaced manual loadUsers() with SWR hooks
 * ✅ Removed manual loading/error state management
 * ✅ Added real-time updates via SWR
 * ✅ Implemented OIDC authentication 
 * ✅ Added cache invalidation instead of manual refresh
 * ✅ Serverless optimized data fetching
 */
export function UserManagementMigrated({ initialFilters = {} }: UserManagementMigratedProps) {
  // ============================================================================
  // MIGRATED: SWR Data Fetching (replaces manual AdminApiService calls)
  // ============================================================================
  
  const [filters, setFilters] = useState<AdminFilters>(initialFilters)
  
  // ✅ NEW: SWR hook replaces manual loadUsers() function
  const { 
    data: usersData, 
    error, 
    isLoading,
    mutate: revalidateUsers 
  } = adminClientData.useUsers(filters, {
    refreshInterval: 30000, // Auto-refresh every 30 seconds
    revalidateOnFocus: true, // Refresh when window regains focus
  })
  
  // ✅ NEW: Real-time connection for live updates
  const { isConnected } = adminClientData.useRealTime(true)
  
  // ✅ NEW: Cache management utilities
  const { invalidateUsers } = adminClientData.useCache()
  
  // ============================================================================
  // Component State (Modal/Dialog Management)
  // ============================================================================
  
  const [searchTerm, setSearchTerm] = useState('')
  const [filterRole, setFilterRole] = useState<string>('all')
  const [filterStatus, setFilterStatus] = useState<string>('all')
  const [actionLoading, setActionLoading] = useState<string | null>(null)
  const [isPending, startTransition] = useTransition()
  const { addToast } = useToast()

  // User level assignment state
  const [showLevelModal, setShowLevelModal] = useState(false)
  const [selectedUser, setSelectedUser] = useState<AdminUser | null>(null)
  const [selectedLevel, setSelectedLevel] = useState<UserLevel>(UserLevel.BRONZE)
  const [levelReason, setLevelReason] = useState('')

  // Permission profile assignment state
  const [showProfileModal, setShowProfileModal] = useState(false)
  const [profileUserId, setProfileUserId] = useState<string>('')
  const [profileId, setProfileId] = useState<string>('')
  const [expiresAt, setExpiresAt] = useState<string>('')

  // Soft delete state
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [userToDelete, setUserToDelete] = useState<string>('')
  const [deleteReason, setDeleteReason] = useState<string>('')

  // ============================================================================
  // MIGRATED: Data Processing (replaces manual filtering)
  // ============================================================================
  
  // ✅ NEW: Process SWR data instead of local state
  const users = usersData?.users || []
  const totalUsers = usersData?.total || 0
  
  // Filter users client-side for immediate response
  const filteredUsers = users.filter(user => {
    const matchesSearch = !searchTerm || 
      user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.name?.toLowerCase().includes(searchTerm.toLowerCase())
    
    const matchesRole = filterRole === 'all' || user.role === filterRole
    const matchesStatus = filterStatus === 'all' || 
      (filterStatus === 'active' && !user.disabled) ||
      (filterStatus === 'inactive' && user.disabled)
    
    return matchesSearch && matchesRole && matchesStatus
  })

  // ============================================================================
  // MIGRATED: Event Handlers (now use cache invalidation)
  // ============================================================================

  /**
   * ✅ MIGRATED: Role change now uses cache invalidation instead of manual refresh
   */
  const handleRoleChange = async (uid: string, newRole: string) => {
    try {
      setActionLoading(uid)
      
      // Make API call (this could be moved to a mutation hook)
      const response = await fetch(`/api/v1/admin/users/${uid}/role`, {
        method: 'PUT',
        credentials: 'include', // OIDC cookies
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ role: newRole }),
      })
      
      if (!response.ok) {
        throw new Error(`Failed to update role: ${response.status}`)
      }
      
      // ✅ NEW: Cache invalidation instead of manual loadUsers()
      await invalidateUsers(filters)
      await revalidateUsers() // SWR revalidation
      
      addToast({
        type: 'success',
        title: 'Role updated successfully',
        description: `User role changed to ${newRole}`,
      })
      
    } catch (err: any) {
      console.error('Failed to change role', { error: err.message, uid, newRole })
      addToast({
        type: 'error',
        title: 'Failed to change user role',
        description: err.message,
      })
    } finally {
      setActionLoading(null)
    }
  }

  /**
   * ✅ MIGRATED: Status toggle with cache invalidation
   */
  const handleStatusToggle = async (uid: string, disabled: boolean) => {
    try {
      setActionLoading(uid)
      
      const response = await fetch(`/api/v1/admin/users/${uid}/status`, {
        method: 'PUT',
        credentials: 'include', // OIDC cookies
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ active: !disabled }),
      })
      
      if (!response.ok) {
        throw new Error(`Failed to update status: ${response.status}`)
      }
      
      // ✅ NEW: Cache invalidation
      await invalidateUsers(filters)
      await revalidateUsers()
      
      addToast({
        type: 'success',
        title: 'Status updated successfully',
        description: `User ${disabled ? 'activated' : 'deactivated'}`,
      })
      
    } catch (err: any) {
      console.error('Failed to toggle status', { error: err.message, uid, disabled })
      addToast({
        type: 'error',
        title: 'Failed to update user status',
        description: err.message,
      })
    } finally {
      setActionLoading(null)
    }
  }

  /**
   * ✅ MIGRATED: Filter handling with immediate SWR updates
   */
  const handleFilterChange = () => {
    const newFilters = {
      ...filters,
      search: searchTerm || undefined,
      role: filterRole !== 'all' ? filterRole : undefined,
      status: filterStatus !== 'all' ? filterStatus : undefined,
      page: 1, // Reset to first page
    }
    
    setFilters(newFilters)
    // SWR will automatically refetch with new filters
  }

  /**
   * ✅ NEW: Manual refresh using SWR revalidation
   */
  const handleManualRefresh = async () => {
    await revalidateUsers()
    addToast({
      type: 'success',
      title: 'Data refreshed',
      description: 'User data has been updated',
    })
  }

  // ============================================================================
  // Render Component
  // ============================================================================

  return (
    <div className="space-y-6">
      {/* ✅ NEW: Migration status indicator */}
      <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
        <div className="flex items-center gap-2 text-green-800">
          <CheckCircle className="h-4 w-4" />
          <span className="font-medium">MIGRATED to Hybrid Data Strategy</span>
          <span className="text-sm">
            ({isConnected ? '🟢 Real-time connected' : '🔴 Real-time disconnected'})
          </span>
        </div>
        <p className="text-sm text-green-600 mt-1">
          Using SWR hooks, OIDC authentication, cache invalidation, serverless optimized
        </p>
      </div>

      {/* Header with real-time refresh */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">User Management</h2>
          <p className="text-gray-600">
            {isLoading ? 'Loading...' : `${filteredUsers.length} of ${totalUsers} users`}
          </p>
        </div>
        
        <div className="flex gap-2">
          <Button
            onClick={handleManualRefresh}
            disabled={isLoading}
            variant="outline"
            size="sm"
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* ✅ MIGRATED: Filters with immediate client-side response */}
      <_Card>
        <_CardHeader>
          <_CardTitle className="flex items-center gap-2">
            <Search className="h-5 w-5" />
            Filter Users
          </_CardTitle>
          <_CardDescription>
            Filter users by search term, role, or status - updates in real-time
          </_CardDescription>
        </_CardHeader>
        <_CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-3">
            <div>
              <Label htmlFor="search">Search</Label>
              <Input
                id="search"
                placeholder="Search by email or name..."
                value={searchTerm}
                onChange={(e) => {
                  setSearchTerm(e.target.value)
                  // Auto-apply filters after typing stops
                  setTimeout(handleFilterChange, 300)
                }}
              />
            </div>
            
            <div>
              <Label htmlFor="role-filter">Role</Label>
              <Select value={filterRole} onValueChange={setFilterRole}>
                <SelectTrigger>
                  <SelectValue placeholder="Select role" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Roles</SelectItem>
                  <SelectItem value="admin">Admin</SelectItem>
                  <SelectItem value="user">User</SelectItem>
                  <SelectItem value="guest">Guest</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div>
              <Label htmlFor="status-filter">Status</Label>
              <Select value={filterStatus} onValueChange={setFilterStatus}>
                <SelectTrigger>
                  <SelectValue placeholder="Select status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="inactive">Inactive</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          
          <Button onClick={handleFilterChange} className="w-full">
            Apply Filters
          </Button>
        </_CardContent>
      </_Card>

      {/* ✅ MIGRATED: Error handling from SWR */}
      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <div className="flex items-center gap-2 text-red-800">
            <AlertTriangle className="h-4 w-4" />
            <span className="font-medium">Error loading users</span>
          </div>
          <p className="text-sm text-red-600 mt-1">{error.message}</p>
          <Button 
            onClick={handleManualRefresh} 
            variant="outline" 
            size="sm" 
            className="mt-2"
          >
            Retry
          </Button>
        </div>
      )}

      {/* ✅ MIGRATED: User list with SWR data */}
      <_Card>
        <_CardHeader>
          <_CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Users ({filteredUsers.length})
          </_CardTitle>
        </_CardHeader>
        <_CardContent>
          {isLoading ? (
            <div className="space-y-4">
              {[1, 2, 3].map(i => (
                <div key={i} className="animate-pulse">
                  <div className="h-4 bg-gray-200 rounded w-full"></div>
                </div>
              ))}
            </div>
          ) : filteredUsers.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              <UserX className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No users found matching the current filters</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b">
                    <th className="text-left p-2">Email</th>
                    <th className="text-left p-2">Name</th>
                    <th className="text-left p-2">Role</th>
                    <th className="text-left p-2">Status</th>
                    <th className="text-left p-2">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredUsers.map((user) => (
                    <tr key={user.id} className="border-b hover:bg-gray-50">
                      <td className="p-2">{user.email}</td>
                      <td className="p-2">{user.name || 'N/A'}</td>
                      <td className="p-2">
                        <Select
                          value={user.role}
                          onValueChange={(newRole) => handleRoleChange(user.id, newRole)}
                          disabled={actionLoading === user.id}
                        >
                          <SelectTrigger className="w-32">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="admin">Admin</SelectItem>
                            <SelectItem value="user">User</SelectItem>
                            <SelectItem value="guest">Guest</SelectItem>
                          </SelectContent>
                        </Select>
                      </td>
                      <td className="p-2">
                        <Button
                          variant={user.disabled ? "destructive" : "default"}
                          size="sm"
                          onClick={() => handleStatusToggle(user.id, user.disabled)}
                          disabled={actionLoading === user.id}
                        >
                          {actionLoading === user.id ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : user.disabled ? (
                            'Inactive'
                          ) : (
                            'Active'
                          )}
                        </Button>
                      </td>
                      <td className="p-2">
                        <div className="flex gap-1">
                          <Button size="sm" variant="outline">
                            Edit
                          </Button>
                          <Button 
                            size="sm" 
                            variant="destructive"
                            onClick={() => {
                              setUserToDelete(user.id)
                              setDeleteDialogOpen(true)
                            }}
                          >
                            Delete
                          </Button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </_CardContent>
      </_Card>

      {/* Performance comparison */}
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h3 className="font-medium text-blue-800 mb-2">Migration Benefits Achieved:</h3>
        <div className="grid gap-2 md:grid-cols-2 text-sm text-blue-700">
          <div>
            <strong>Before:</strong> Manual loadUsers(), manual state management, no real-time updates
          </div>
          <div>
            <strong>After:</strong> SWR hooks, automatic caching, real-time updates, OIDC auth
          </div>
        </div>
      </div>
    </div>
  )
}

export default UserManagementMigrated