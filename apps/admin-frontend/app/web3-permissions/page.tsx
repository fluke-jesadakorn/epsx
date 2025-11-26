'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Shield, Users, Activity, Plus, Edit3, Trash2, Clock, Wallet, Search, UserPlus, ArrowLeft } from 'lucide-react'
import { useState, useEffect } from 'react'
import { toast } from 'react-hot-toast'

import { Web3PermissionManager } from '@/components/admin/Web3PermissionManager'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client'

export const dynamic = 'force-dynamic'

/**
 *
 */
export default function Web3AdminPermissionsPage() {
  const [activeView, setActiveView] = useState<'main' | 'create-group' | 'assign-wallet' | 'edit-group' | 'group-members' | 'expiring'>('main')
  const [editingGroup, setEditingGroup] = useState<PermissionGroup | null>(null)
  const [viewingMembersGroup, setViewingMembersGroup] = useState<PermissionGroup | null>(null)
  const [walletSearchTerm, setWalletSearchTerm] = useState('')
  const [searchResults, setSearchResults] = useState<any[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const [recentActivity, setRecentActivity] = useState<any[]>([])
  const [isLoadingActivity, setIsLoadingActivity] = useState(false)
  const [expiringAssignments, setExpiringAssignments] = useState<any[]>([])
  const queryClient = useQueryClient()

  // Fetch permission groups
  const { data: permissionGroups = [], isLoading: groupsLoading, error: groupsError } = useQuery({
    queryKey: ['permission-groups'],
    queryFn: async () => {
      try {
        const result = await groupMgmt.getPermissionGroups()
        return result || []
      } catch (error) {
        return []
      }
    },
    refetchInterval: 30000,
    retry: 1,
    staleTime: 5 * 60 * 1000,
    placeholderData: [],
  })

  // Fetch group analytics
  const { data: analytics, isLoading: analyticsLoading } = useQuery({
    queryKey: ['group-analytics'],
    queryFn: () => groupMgmt.getGroupAnalytics(),
    refetchInterval: 60000
  })

  // Delete group mutation
  const deleteGroupMutation = useMutation({
    mutationFn: (groupId: string) => groupMgmt.deletePermissionGroup(groupId),
    onSuccess: () => {
      toast.success('Permission group deleted successfully')
      queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
      queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to delete permission group')
    }
  })

  const handleGroupCreated = () => {
    queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
    queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
    setActiveView('main')
  }

  const handleWalletAssigned = async () => {
    queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
    setActiveView('main')
  }

  const handleDeleteGroup = (groupId: string, groupName: string) => {
    if (confirm(`Are you sure you want to delete the "${groupName}" group? This action cannot be undone.`)) {
      deleteGroupMutation.mutate(groupId)
    }
  }

  const handleEditGroup = (group: PermissionGroup) => {
    setEditingGroup(group)
    setActiveView('edit-group')
  }

  const handleGroupUpdated = () => {
    queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
    queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
    setActiveView('main')
    setEditingGroup(null)
  }

  const handleViewMembers = (group: PermissionGroup) => {
    setViewingMembersGroup(group)
    setActiveView('group-members')
  }

  // Debounced search effect
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (walletSearchTerm && walletSearchTerm.length >= 3) {
        handleWalletSearch(walletSearchTerm)
      } else {
        setSearchResults([])
      }
    }, 500)

    return () => clearTimeout(timeoutId)
  }, [walletSearchTerm])

  // Load recent activity
  useEffect(() => {
    const loadRecentActivity = async () => {
      if (activeView !== 'main') return

      setIsLoadingActivity(true)
      try {
        const response = await groupMgmt.getGroupAssignmentHistory({
          limit: 10,
          date_from: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
        })

        if (response.success && response.data) {
          setRecentActivity(response.data.history)
        } else {
          setRecentActivity([])
        }
      } catch (error) {
        setRecentActivity([])
      } finally {
        setIsLoadingActivity(false)
      }
    }

    loadRecentActivity()
  }, [activeView])

  // Load expiring assignments when view is opened
  useEffect(() => {
    if (activeView === 'expiring') {
      setIsLoadingActivity(true)
      groupMgmt.getExpiringMemberships(7)
        .then(setExpiringAssignments)
        .catch(() => setExpiringAssignments([]))
        .finally(() => setIsLoadingActivity(false))
    }
  }, [activeView])

  const handleWalletSearch = async (searchTerm: string) => {
    setIsSearching(true)
    try {
      const response = await groupMgmt.getGroupAssignmentHistory({
        user_search: searchTerm,
        limit: 20
      })

      if (response.success && response.data) {
        setSearchResults(response.data.history)
      } else {
        setSearchResults([])
      }
    } catch (error: any) {
      toast.error('Failed to search wallets: ' + error.message)
      setSearchResults([])
    } finally {
      setIsSearching(false)
    }
  }

  // Show loading state
  if (groupsLoading || analyticsLoading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
        <div className="max-w-7xl mx-auto space-y-6">
          <div className="text-center mb-12">
            <div className="h-12 sm:h-16 bg-gray-300 dark:bg-gray-700 rounded-2xl max-w-md mx-auto mb-4"></div>
            <div className="h-4 sm:h-6 bg-gray-200 dark:bg-gray-800 rounded-full max-w-sm mx-auto"></div>
          </div>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="h-48 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl border-2 border-gray-300/50 dark:border-gray-700/50"></div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative max-w-7xl mx-auto space-y-6 sm:space-y-8">
        {/* Page Header */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              🛡️ Permission Management
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Manage permission groups and wallet assignments
          </p>
        </div>

        {/* Back Button (for non-main views) */}
        {activeView !== 'main' && (
          <div className="mb-4">
            <Button
              onClick={() => {
                setActiveView('main')
                setEditingGroup(null)
                setViewingMembersGroup(null)
              }}
              variant="outline"
              className="flex items-center gap-2"
            >
              <ArrowLeft className="w-4 h-4" />
              Back to Overview
            </Button>
          </div>
        )}

        {/* Main View - Analytics Cards and Groups */}
        {activeView === 'main' && (
          <>
            {/* Analytics Cards */}
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-2xl sm:text-3xl">👥</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Groups</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-blue-600">{analytics?.total_groups || 0}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Total Groups</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-2xl sm:text-3xl">✅</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Active</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-green-600">{analytics?.total_active_memberships || 0}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Memberships</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-2xl sm:text-3xl">⏰</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Soon</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-orange-600">{analytics?.expiring_soon_count || 0}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Expiring</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-2xl sm:text-3xl">🏆</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Top</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-purple-600">
                    {analytics?.most_popular_groups?.[0]?.member_count || 0}
                  </div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300 truncate">Largest</div>
                </div>
              </div>
            </div>

            {/* Quick Actions Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mb-6 sm:mb-8">
              {[
                {
                  title: '👥 Create Group',
                  description: 'Create a new permission group',
                  gradient: 'from-blue-400 to-cyan-500',
                  bgGradient: 'from-blue-400/20 via-cyan-400/20 to-blue-400/20',
                  onClick: () => setActiveView('create-group')
                },
                {
                  title: '💼 Assign Wallet',
                  description: 'Assign wallet to a group',
                  gradient: 'from-green-400 to-emerald-500',
                  bgGradient: 'from-green-400/20 via-emerald-400/20 to-green-400/20',
                  onClick: () => setActiveView('assign-wallet')
                },
                {
                  title: '⏰ Expiring Soon',
                  description: 'View expiring assignments',
                  gradient: 'from-orange-400 to-pink-500',
                  bgGradient: 'from-orange-400/20 via-pink-400/20 to-orange-400/20',
                  onClick: () => setActiveView('expiring')
                }
              ].map((action, index) => (
                <button key={index} onClick={action.onClick} className="block group text-left">
                  <div className={`relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r ${action.bgGradient} p-0.5 hover:scale-105 transition-all duration-300`}>
                    <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
                      <div className={`absolute top-4 right-4 w-4 h-4 bg-gradient-to-r ${action.gradient} rounded-full blur-sm opacity-60`}></div>

                      <div className="p-4 sm:p-6">
                        <h3 className={`text-lg sm:text-xl font-bold bg-gradient-to-r ${action.gradient} bg-clip-text text-transparent mb-2`}>
                          {action.title}
                        </h3>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                          {action.description}
                        </p>

                        <div className="flex items-center justify-between">
                          <div className={`px-3 py-1 bg-gradient-to-r ${action.gradient} text-white rounded-full text-xs font-medium`}>
                            Open
                          </div>
                          <div className="text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </button>
              ))}
            </div>

            {/* Permission Groups Grid */}
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50">
              <h2 className="text-2xl font-bold text-indigo-600 dark:text-indigo-400 mb-6 flex items-center gap-2">
                <Shield className="w-6 h-6" />
                Permission Groups
              </h2>

              {groupsError ? (
                <div className="bg-red-50/50 dark:bg-red-900/10 border border-red-200/50 dark:border-red-800/50 rounded-2xl p-4">
                  <p className="text-sm text-gray-700 dark:text-gray-300">
                    Failed to load permission groups: {groupsError.message}
                  </p>
                </div>
              ) : (
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {permissionGroups.map((group) => {
                    const borderColor = group.is_system_group
                      ? 'border-purple-300/50 dark:border-purple-700/50'
                      : 'border-blue-300/50 dark:border-blue-700/50'

                    return (
                      <div key={group.id} className={`bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-4 sm:p-6 shadow-xl border-2 ${borderColor} hover:shadow-2xl transition-shadow`}>
                        <div className="flex items-center justify-between mb-3 sm:mb-4">
                          <div className="text-2xl sm:text-3xl">
                            {group.is_system_group ? '⚙️' : '👥'}
                          </div>
                          <div className="flex items-center gap-2">
                            {group.is_system_group && (
                              <span className="text-xs px-2 py-1 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded-full">
                                System
                              </span>
                            )}
                            <span className="text-xs px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded-full">
                              Active
                            </span>
                          </div>
                        </div>

                        <div className="space-y-3">
                          <div>
                            <h3 className="text-lg sm:text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
                              {group.name}
                            </h3>
                            {group.description && (
                              <p className="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                                {group.description}
                              </p>
                            )}
                          </div>

                          <div className="space-y-2 text-sm">
                            <div className="flex justify-between items-center">
                              <span className="text-gray-600 dark:text-gray-400">Members</span>
                              <span className="font-semibold text-blue-600">{group.member_count ?? 0}</span>
                            </div>
                            <div className="flex justify-between items-center">
                              <span className="text-gray-600 dark:text-gray-400">Permissions</span>
                              <span className="font-semibold text-green-600">{group.permissions.length}</span>
                            </div>
                          </div>

                          <div className="flex space-x-2 pt-2">
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => handleEditGroup(group)}
                              className="flex-1"
                            >
                              <Edit3 className="w-3 h-3 mr-1" />
                              Edit
                            </Button>
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => handleViewMembers(group)}
                              className="flex-1"
                            >
                              <Users className="w-3 h-3 mr-1" />
                              Members
                            </Button>
                            {!group.is_system_group && (
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => handleDeleteGroup(group.id, group.name)}
                                disabled={deleteGroupMutation.isPending}
                                className="text-red-600 hover:text-red-700"
                              >
                                <Trash2 className="w-3 h-3" />
                              </Button>
                            )}
                          </div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>
          </>
        )}

        {/* Create Group View */}
        {activeView === 'create-group' && (
          <CreateGroupSection onSuccess={handleGroupCreated} />
        )}

        {/* Assign Wallet View */}
        {activeView === 'assign-wallet' && (
          <AssignWalletSection onSuccess={handleWalletAssigned} />
        )}

        {/* Edit Group View */}
        {activeView === 'edit-group' && editingGroup && (
          <EditGroupSection group={editingGroup} onSuccess={handleGroupUpdated} />
        )}

        {/* Group Members View */}
        {activeView === 'group-members' && viewingMembersGroup && (
          <GroupMembersSection group={viewingMembersGroup} onClose={() => setActiveView('main')} />
        )}

        {/* Expiring Assignments View */}
        {activeView === 'expiring' && (
          <ExpiringAssignmentsSection
            assignments={expiringAssignments}
            isLoading={isLoadingActivity}
            onUpdate={() => {
              setIsLoadingActivity(true)
              groupMgmt.getExpiringMemberships(7)
                .then(setExpiringAssignments)
                .catch(() => setExpiringAssignments([]))
                .finally(() => setIsLoadingActivity(false))
            }}
          />
        )}
      </div>
    </div>
  )
}

// Create Group Section Component
function CreateGroupSection({ onSuccess }: { onSuccess: () => void }) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    permissions: [] as string[],
    default_expiry_days: '',
    priority_level: 0
  })

  const [selectedPermission, setSelectedPermission] = useState('')

  const availablePermissions = [
    'epsx:analytics:view',
    'epsx:analytics:advanced',
    'epsx:trading:basic',
    'epsx:trading:advanced',
    'epsx:trading:pro',
    'epsx:data:export',
    'epsx:api:read',
    'epsx:api:write',
    'epsx:notifications:manage',
    'admin:users:view',
    'admin:users:manage',
    'admin:permissions:view',
    'admin:permissions:manage',
    'admin:*:*'
  ]

  const createGroupMutation = useMutation({
    mutationFn: async (data: typeof formData) => {
      return groupMgmt.createPermissionGroup({
        name: data.name,
        description: data.description,
        permissions: data.permissions,
        default_expiry_days: data.default_expiry_days ? parseInt(data.default_expiry_days) : undefined,
        priority_level: data.priority_level
      })
    },
    onSuccess: () => {
      toast.success('Permission group created successfully')
      onSuccess()
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to create permission group')
    }
  })

  const addPermission = () => {
    if (selectedPermission && !formData.permissions.includes(selectedPermission)) {
      setFormData(prev => ({
        ...prev,
        permissions: [...prev.permissions, selectedPermission]
      }))
      setSelectedPermission('')
    }
  }

  const removePermission = (permission: string) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.filter(p => p !== permission)
    }))
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.name || formData.permissions.length === 0) {
      toast.error('Name and at least one permission are required')
      return
    }
    createGroupMutation.mutate(formData)
  }

  return (
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
      <h2 className="text-2xl font-bold text-blue-600 dark:text-blue-400 mb-6 flex items-center gap-2">
        <Plus className="w-6 h-6" />
        Create Permission Group
      </h2>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <Label htmlFor="name">Group Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
            placeholder="e.g., Premium Users"
            required
          />
        </div>

        <div>
          <Label htmlFor="description">Description</Label>
          <Textarea
            id="description"
            value={formData.description}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            placeholder="Describe what this group is for..."
          />
        </div>

        <div>
          <Label>Permissions</Label>
          <div className="flex gap-2 mb-2">
            <Select value={selectedPermission} onValueChange={setSelectedPermission}>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder="Select permission to add" />
              </SelectTrigger>
              <SelectContent>
                {availablePermissions.map(perm => (
                  <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button type="button" onClick={addPermission} disabled={!selectedPermission}>
              <Plus className="w-4 h-4" />
            </Button>
          </div>
          <div className="flex flex-wrap gap-2">
            {formData.permissions.map(perm => (
              <Badge key={perm} variant="secondary" className="flex items-center gap-1">
                {perm}
                <button
                  type="button"
                  onClick={() => removePermission(perm)}
                  className="ml-1 text-red-500 hover:text-red-700"
                >
                  ×
                </button>
              </Badge>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label htmlFor="expiry">Default Expiry (Days)</Label>
            <Input
              id="expiry"
              type="number"
              value={formData.default_expiry_days}
              onChange={(e) => setFormData(prev => ({ ...prev, default_expiry_days: e.target.value }))}
              placeholder="30"
            />
          </div>
          <div>
            <Label htmlFor="priority">Priority Level</Label>
            <Input
              id="priority"
              type="number"
              value={formData.priority_level}
              onChange={(e) => setFormData(prev => ({ ...prev, priority_level: parseInt(e.target.value) || 0 }))}
              placeholder="0"
            />
          </div>
        </div>

        <Button type="submit" disabled={createGroupMutation.isPending} className="w-full">
          {createGroupMutation.isPending ? 'Creating...' : 'Create Group'}
        </Button>
      </form>
    </div>
  )
}

// Assign Wallet Section Component
function AssignWalletSection({ onSuccess }: { onSuccess: () => void }) {
  const [formData, setFormData] = useState({
    wallet_address: '',
    group_id: '',
    expires_at: '',
    reason: ''
  })

  const { data: permissionGroups = [] } = useQuery({
    queryKey: ['permission-groups'],
    queryFn: () => groupMgmt.getPermissionGroups()
  })

  const assignWalletMutation = useMutation({
    mutationFn: async (data: typeof formData) => {
      return groupMgmt.assignUserToGroup({
        user_id: data.wallet_address,
        group_id: data.group_id,
        expires_at: data.expires_at || null,
        reason: data.reason
      })
    },
    onSuccess: () => {
      toast.success('Wallet assigned to group successfully')
      onSuccess()
      setFormData({ wallet_address: '', group_id: '', expires_at: '', reason: '' })
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to assign wallet to group')
    }
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.wallet_address || !formData.group_id) {
      toast.error('Wallet address and group are required')
      return
    }
    assignWalletMutation.mutate(formData)
  }

  return (
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
      <h2 className="text-2xl font-bold text-green-600 dark:text-green-400 mb-6 flex items-center gap-2">
        <UserPlus className="w-6 h-6" />
        Assign Wallet to Group
      </h2>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <Label htmlFor="wallet_address">Wallet Address</Label>
          <Input
            id="wallet_address"
            value={formData.wallet_address}
            onChange={(e) => setFormData(prev => ({ ...prev, wallet_address: e.target.value }))}
            placeholder="0x..."
            required
          />
        </div>

        <div>
          <Label htmlFor="group_id">Permission Group</Label>
          <Select value={formData.group_id} onValueChange={(value) => setFormData(prev => ({ ...prev, group_id: value }))}>
            <SelectTrigger>
              <SelectValue placeholder="Select permission group" />
            </SelectTrigger>
            <SelectContent>
              {permissionGroups.map(group => (
                <SelectItem key={group.id} value={group.id}>{group.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label htmlFor="expires_at">Expires At (Optional)</Label>
          <Input
            id="expires_at"
            type="datetime-local"
            value={formData.expires_at}
            onChange={(e) => setFormData(prev => ({ ...prev, expires_at: e.target.value }))}
          />
        </div>

        <div>
          <Label htmlFor="reason">Reason (Optional)</Label>
          <Textarea
            id="reason"
            value={formData.reason}
            onChange={(e) => setFormData(prev => ({ ...prev, reason: e.target.value }))}
            placeholder="Reason for assignment..."
          />
        </div>

        <Button type="submit" disabled={assignWalletMutation.isPending} className="w-full">
          {assignWalletMutation.isPending ? 'Assigning...' : 'Assign Wallet'}
        </Button>
      </form>
    </div>
  )
}

// Edit Group Section Component
function EditGroupSection({ group, onSuccess }: { group: PermissionGroup; onSuccess: () => void }) {
  const [formData, setFormData] = useState({
    name: group.name || '',
    description: group.description || '',
    permissions: group.permissions || [],
    default_expiry_days: group.default_expiry_days?.toString() || '',
    priority_level: group.priority_level || 0
  })

  const [selectedPermission, setSelectedPermission] = useState('')

  const availablePermissions = [
    'epsx:analytics:view',
    'epsx:analytics:advanced',
    'epsx:trading:basic',
    'epsx:trading:advanced',
    'epsx:trading:pro',
    'epsx:data:export',
    'epsx:api:read',
    'epsx:api:write',
    'epsx:notifications:manage',
    'admin:users:view',
    'admin:users:manage',
    'admin:permissions:view',
    'admin:permissions:manage',
    'admin:*:*'
  ]

  const updateGroupMutation = useMutation({
    mutationFn: async (data: typeof formData) => {
      return groupMgmt.updatePermissionGroup(group.id, {
        name: data.name,
        description: data.description,
        permissions: data.permissions,
        default_expiry_days: data.default_expiry_days ? parseInt(data.default_expiry_days) : undefined,
        priority_level: data.priority_level
      })
    },
    onSuccess: () => {
      toast.success('Permission group updated successfully')
      onSuccess()
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to update permission group')
    }
  })

  const addPermission = () => {
    if (selectedPermission && !formData.permissions.includes(selectedPermission)) {
      setFormData(prev => ({
        ...prev,
        permissions: [...prev.permissions, selectedPermission]
      }))
      setSelectedPermission('')
    }
  }

  const removePermission = (permission: string) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.filter(p => p !== permission)
    }))
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.name || formData.permissions.length === 0) {
      toast.error('Name and at least one permission are required')
      return
    }
    updateGroupMutation.mutate(formData)
  }

  return (
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
      <h2 className="text-2xl font-bold text-purple-600 dark:text-purple-400 mb-6 flex items-center gap-2">
        <Edit3 className="w-6 h-6" />
        Edit Permission Group
      </h2>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <Label htmlFor="name">Group Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
            placeholder="e.g., Premium Users"
            required
          />
        </div>

        <div>
          <Label htmlFor="description">Description</Label>
          <Textarea
            id="description"
            value={formData.description}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            placeholder="Describe what this group is for..."
          />
        </div>

        <div>
          <Label>Permissions</Label>
          <div className="flex gap-2 mb-2">
            <Select value={selectedPermission} onValueChange={setSelectedPermission}>
              <SelectTrigger className="flex-1">
                <SelectValue placeholder="Select permission to add" />
              </SelectTrigger>
              <SelectContent>
                {availablePermissions.map(perm => (
                  <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button type="button" onClick={addPermission} disabled={!selectedPermission}>
              <Plus className="w-4 h-4" />
            </Button>
          </div>
          <div className="flex flex-wrap gap-2">
            {formData.permissions.map(perm => (
              <Badge key={perm} variant="secondary" className="flex items-center gap-1">
                {perm}
                <button
                  type="button"
                  onClick={() => removePermission(perm)}
                  className="ml-1 text-red-500 hover:text-red-700"
                >
                  ×
                </button>
              </Badge>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label htmlFor="expiry">Default Expiry (Days)</Label>
            <Input
              id="expiry"
              type="number"
              value={formData.default_expiry_days}
              onChange={(e) => setFormData(prev => ({ ...prev, default_expiry_days: e.target.value }))}
              placeholder="30"
            />
          </div>
          <div>
            <Label htmlFor="priority">Priority Level</Label>
            <Input
              id="priority"
              type="number"
              value={formData.priority_level}
              onChange={(e) => setFormData(prev => ({ ...prev, priority_level: parseInt(e.target.value) || 0 }))}
              placeholder="0"
            />
          </div>
        </div>

        <Button type="submit" disabled={updateGroupMutation.isPending} className="w-full">
          {updateGroupMutation.isPending ? 'Updating...' : 'Update Group'}
        </Button>
      </form>
    </div>
  )
}

// Group Members Section Component
function GroupMembersSection({ group, onClose }: { group: PermissionGroup; onClose: () => void }) {
  const [newWalletAddress, setNewWalletAddress] = useState('')
  const [showAddMember, setShowAddMember] = useState(false)
  const queryClient = useQueryClient()

  const addMemberMutation = useMutation({
    mutationFn: async (walletAddress: string) => {
      return groupMgmt.assignUserToGroup({
        user_id: walletAddress,
        group_id: group.id,
        reason: 'Added via Members view'
      })
    },
    onSuccess: () => {
      toast.success('Member added successfully')
      setNewWalletAddress('')
      setShowAddMember(false)
      queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
      queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to add member')
    }
  })

  const handleAddMember = (e: React.FormEvent) => {
    e.preventDefault()
    if (!newWalletAddress?.startsWith('0x')) {
      toast.error('Please enter a valid wallet address')
      return
    }
    addMemberMutation.mutate(newWalletAddress)
  }

  return (
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50">
      <h2 className="text-2xl font-bold text-indigo-600 dark:text-indigo-400 mb-2 flex items-center gap-2">
        <Users className="w-6 h-6" />
        Members of "{group.name}"
      </h2>
      <div className="text-sm text-gray-600 dark:text-gray-400 mb-6">
        Total members: {group.member_count ?? 0}
      </div>

      <div className="space-y-4">
        {/* Add Member Section */}
        <div className="border-b pb-4">
          <div className="flex justify-between items-center mb-3">
            <h3 className="font-semibold">Manage Members</h3>
            <Button
              onClick={() => setShowAddMember(!showAddMember)}
              variant="outline"
              size="sm"
            >
              <UserPlus className="w-4 h-4 mr-2" />
              Add Member
            </Button>
          </div>

          {showAddMember && (
            <form onSubmit={handleAddMember} className="flex gap-2">
              <Input
                value={newWalletAddress}
                onChange={(e) => setNewWalletAddress(e.target.value)}
                placeholder="Enter wallet address (0x...)"
                className="flex-1"
              />
              <Button
                type="submit"
                disabled={addMemberMutation.isPending}
                size="sm"
              >
                {addMemberMutation.isPending ? 'Adding...' : 'Add'}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setShowAddMember(false)
                  setNewWalletAddress('')
                }}
                size="sm"
              >
                Cancel
              </Button>
            </form>
          )}
        </div>

        {/* Members List */}
        <div className="space-y-3">
          <h3 className="font-semibold">Current Members</h3>

          {group.member_count === 0 ? (
            <div className="text-center py-8 text-gray-500">
              <Users className="w-12 h-12 mx-auto mb-4 text-gray-300" />
              <p className="text-lg font-medium">No members in this group</p>
              <p className="text-sm">Add members to grant them the group's permissions</p>
            </div>
          ) : (
            <Alert>
              <AlertDescription>
                Member list display requires backend implementation of group membership endpoint.
                Currently showing member count: {group.member_count}
              </AlertDescription>
            </Alert>
          )}
        </div>

        {/* Group Info */}
        <div className="border-t pt-4">
          <h3 className="font-semibold mb-2">Group Permissions</h3>
          <div className="flex flex-wrap gap-2">
            {group.permissions.map(permission => (
              <Badge key={permission} variant="secondary">
                {permission}
              </Badge>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

// Expiring Assignments Section Component
function ExpiringAssignmentsSection({
  assignments,
  isLoading,
  onUpdate
}: {
  assignments: any[]
  isLoading: boolean
  onUpdate: () => void
}) {
  const queryClient = useQueryClient()

  const extendAssignmentMutation = useMutation({
    mutationFn: async ({ userId, groupId, days }: { userId: string; groupId: string; days: number }) => {
      const newExpiryDate = new Date()
      newExpiryDate.setDate(newExpiryDate.getDate() + days)

      return groupMgmt.assignUserToGroup({
        user_id: userId,
        group_id: groupId,
        expires_at: newExpiryDate.toISOString(),
        reason: `Extended expiry by ${days} days`
      })
    },
    onSuccess: () => {
      toast.success('Assignment extended successfully')
      queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
      onUpdate()
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to extend assignment')
    }
  })

  const removeAssignmentMutation = useMutation({
    mutationFn: async ({ userId, groupId }: { userId: string; groupId: string }) => {
      return groupMgmt.removeUserFromGroup(userId, groupId)
    },
    onSuccess: () => {
      toast.success('Assignment removed successfully')
      queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
      onUpdate()
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to remove assignment')
    }
  })

  return (
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50">
      <h2 className="text-2xl font-bold text-orange-600 dark:text-orange-400 mb-2 flex items-center gap-2">
        <Clock className="w-6 h-6" />
        Expiring Assignments
      </h2>
      <div className="text-sm text-gray-600 dark:text-gray-400 mb-6">
        Assignments expiring in the next 7 days
      </div>

      <div className="space-y-4">
        {isLoading ? (
          <div className="text-center py-8">
            <div className="h-8 w-8 border-b-2 border-orange-600 mx-auto mb-4"></div>
            <p className="text-gray-500">Loading expiring assignments...</p>
          </div>
        ) : assignments.length > 0 ? (
          <>
            <div className="flex justify-between items-center">
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Found {assignments.length} expiring assignment{assignments.length !== 1 ? 's' : ''}
              </p>
            </div>

            {assignments.map((assignment, index) => {
              const daysUntilExpiry = assignment.expires_at ?
                Math.ceil((new Date(assignment.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)) : null

              return (
                <div key={index} className="p-4 border-orange-200 bg-orange-50/50 dark:bg-orange-900/10 rounded-lg border">
                  <div className="flex justify-between items-start">
                    <div className="space-y-2">
                      <div className="font-mono text-sm font-medium">
                        {assignment.user_email || `${(assignment.user_id?.slice(0, 6)) || ''}...${(assignment.user_id?.slice(-4)) || ''}`}
                      </div>
                      <div className="flex items-center gap-2 text-sm">
                        <Badge variant="outline" className="text-xs">
                          {assignment.group?.name || assignment.group_id}
                        </Badge>
                        {daysUntilExpiry !== null && (
                          <span className="text-orange-600 font-medium">
                            {daysUntilExpiry === 0 ? 'Expires today' :
                              daysUntilExpiry === 1 ? 'Expires tomorrow' :
                                `Expires in ${daysUntilExpiry} days`}
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-gray-500">
                        Granted: {new Date(assignment.granted_at).toLocaleDateString()}
                        {assignment.expires_at && (
                          <> • Expires: {new Date(assignment.expires_at).toLocaleDateString()}</>
                        )}
                      </div>
                    </div>

                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => extendAssignmentMutation.mutate({ userId: assignment.user_id, groupId: assignment.group_id, days: 30 })}
                        disabled={extendAssignmentMutation.isPending}
                      >
                        <Clock className="w-3 h-3 mr-1" />
                        +30d
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => extendAssignmentMutation.mutate({ userId: assignment.user_id, groupId: assignment.group_id, days: 90 })}
                        disabled={extendAssignmentMutation.isPending}
                      >
                        <Clock className="w-3 h-3 mr-1" />
                        +90d
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        className="text-red-600 hover:text-red-700"
                        onClick={() => {
                          if (confirm('Remove this assignment?')) {
                            removeAssignmentMutation.mutate({ userId: assignment.user_id, groupId: assignment.group_id })
                          }
                        }}
                        disabled={removeAssignmentMutation.isPending}
                      >
                        <Trash2 className="w-3 h-3" />
                      </Button>
                    </div>
                  </div>
                </div>
              )
            })}
          </>
        ) : (
          <div className="text-center py-8 text-gray-500">
            <Clock className="w-12 h-12 mx-auto mb-4 text-gray-300" />
            <p className="text-lg font-medium">No expiring assignments</p>
            <p className="text-sm">All assignments are valid for more than 7 days</p>
          </div>
        )}
      </div>
    </div>
  )
}
