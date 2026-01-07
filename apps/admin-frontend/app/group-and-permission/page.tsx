'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Clock, Edit3, Plus, Shield, Trash2, UserPlus, Users } from 'lucide-react'
import Link from 'next/link'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'

import { PermissionRegistry } from '@/components/permissions/PermissionRegistry'
import { PageHeader } from '@/components/shared/PageHeader'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete'
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client'

export const dynamic = 'force-dynamic'

/**
 *
 */
export default function Web3AdminPermissionsPage() {
  const [expiringAssignments, setExpiringAssignments] = useState<any[]>([])
  // Delete confirmation modal state
  const [deleteConfirm, setDeleteConfirm] = useState<{ isOpen: boolean; groupId: string; groupName: string } | null>(null)
  const queryClient = useQueryClient()

  // State for managing different views
  const [activeView, setActiveView] = useState<'main' | 'create-group' | 'assign-wallet' | 'edit-group' | 'group-members' | 'expiring' | 'permissions'>('main')
  const [editingGroup, setEditingGroup] = useState<PermissionGroup | null>(null)
  const [isLoadingActivity, setIsLoadingActivity] = useState(false)
  const [recentActivity, setRecentActivity] = useState<any[]>([])


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
    setDeleteConfirm({ isOpen: true, groupId, groupName })
  }

  const confirmDelete = () => {
    if (deleteConfirm) {
      deleteGroupMutation.mutate(deleteConfirm.groupId)
      setDeleteConfirm(null)
    }
  }

  const cancelDelete = () => {
    setDeleteConfirm(null)
  }

  const handleGroupUpdated = () => {
    queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
    queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
    setActiveView('main')
    setEditingGroup(null)
  }


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

        if (response && response.history) {
          setRecentActivity(response.history)
        }
      } catch (error) {
        // Handle error
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


  // Show loading state
  if (groupsLoading || analyticsLoading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
        <div className="max-w-7xl mx-auto space-y-6">
          <div className="text-center mb-12">
            <div className="h-12 sm:h-16 bg-gray-300 dark:bg-gray-700 rounded-2xl max-w-md mx-auto mb-4"></div>
            <div className="h-4 sm:h-6 bg-gray-200 dark:bg-gray-800 rounded-full max-w-sm mx-auto"></div>
          </div>
          <div className="grid gap-3 sm:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
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


      <div className="relative max-w-7xl mx-auto px-3 sm:px-4 lg:px-6 space-y-6 sm:space-y-8">
        {/* Page Header */}
        <PageHeader
          title="Group & Permission Management"
          description="Manage permission groups and wallet assignments"
          icon={Shield}
        />

        {/* Back Button (for non-main views) */}
        {activeView !== 'main' && (
          <div className="mb-4">
            <Button
              onClick={() => {
                setActiveView('main')
                setEditingGroup(null)
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
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4 lg:gap-6 mb-6 sm:mb-8">
              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-xl sm:rounded-2xl lg:rounded-3xl p-3 sm:p-4 lg:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-2 sm:mb-3 lg:mb-4">
                  <div className="text-xl sm:text-2xl lg:text-3xl">👥</div>
                  <span className="text-xs sm:text-xs lg:text-sm font-medium text-gray-600 dark:text-gray-300">Groups</span>
                </div>
                <div className="space-y-1">
                  <div className="text-xl sm:text-2xl lg:text-3xl font-bold text-blue-600 break-all">{analytics?.total_groups || 0}</div>
                  <div className="text-xs sm:text-xs lg:text-sm text-gray-700 dark:text-gray-200">Total Groups</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-xl sm:rounded-2xl lg:rounded-3xl p-3 sm:p-4 lg:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-2 sm:mb-3 lg:mb-4">
                  <div className="text-xl sm:text-2xl lg:text-3xl">✅</div>
                  <span className="text-xs sm:text-xs lg:text-sm font-medium text-gray-600 dark:text-gray-300">Active</span>
                </div>
                <div className="space-y-1">
                  <div className="text-xl sm:text-2xl lg:text-3xl font-bold text-green-600 break-all">{analytics?.total_active_memberships || 0}</div>
                  <div className="text-xs sm:text-xs lg:text-sm text-gray-700 dark:text-gray-200">Memberships</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-xl sm:rounded-2xl lg:rounded-3xl p-3 sm:p-4 lg:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-2 sm:mb-3 lg:mb-4">
                  <div className="text-xl sm:text-2xl lg:text-3xl">⏰</div>
                  <span className="text-xs sm:text-xs lg:text-sm font-medium text-gray-600 dark:text-gray-300">Soon</span>
                </div>
                <div className="space-y-1">
                  <div className="text-xl sm:text-2xl lg:text-3xl font-bold text-orange-600 break-all">{analytics?.expiring_soon_count || 0}</div>
                  <div className="text-xs sm:text-xs lg:text-sm text-gray-700 dark:text-gray-200">Expiring</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-xl sm:rounded-2xl lg:rounded-3xl p-3 sm:p-4 lg:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50 hover:shadow-2xl transition-shadow">
                <div className="flex items-center justify-between mb-2 sm:mb-3 lg:mb-4">
                  <div className="text-xl sm:text-2xl lg:text-3xl">🏆</div>
                  <span className="text-xs sm:text-xs lg:text-sm font-medium text-gray-600 dark:text-gray-300">Top</span>
                </div>
                <div className="space-y-1">
                  <div className="text-xl sm:text-2xl lg:text-3xl font-bold text-purple-600 break-all">
                    {analytics?.most_popular_groups?.[0]?.member_count || 0}
                  </div>
                  <div className="text-xs sm:text-xs lg:text-sm text-gray-700 dark:text-gray-200 truncate">Largest</div>
                </div>
              </div>
            </div>

            {/* Quick Actions Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6 mb-6 sm:mb-8">
              {/* Permission Registry - Links to page */}
              <button onClick={() => setActiveView('permissions')} className="block group text-left">
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-purple-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                  <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl h-full">
                    <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-purple-400 to-pink-500 rounded-full blur-sm opacity-60"></div>
                    <div className="p-4 sm:p-6">
                      <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-purple-400 to-pink-500 bg-clip-text text-transparent mb-2">
                        🔑 Permissions
                      </h3>
                      <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                        Define global permissions
                      </p>
                      <div className="flex items-center justify-between mt-auto">
                        <div className="px-3 py-1 bg-gradient-to-r from-purple-400 to-pink-500 text-white rounded-full text-xs font-medium">
                          Registry
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                      </div>
                    </div>
                  </div>
                </div>
              </button>

              {/* Create Group - Links to page */}
              <Link href="/group-and-permission/create-group" className="block group text-left">
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-cyan-400/20 to-blue-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                  <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl h-full">
                    <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full blur-sm opacity-60"></div>
                    <div className="p-4 sm:p-6">
                      <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-400 to-cyan-500 bg-clip-text text-transparent mb-2">
                        👥 Create Group
                      </h3>
                      <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                        Create a new permission group
                      </p>
                      <div className="flex items-center justify-between mt-auto">
                        <div className="px-3 py-1 bg-gradient-to-r from-blue-400 to-cyan-500 text-white rounded-full text-xs font-medium">
                          Open
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                      </div>
                    </div>
                  </div>
                </div>
              </Link>

              {/* Assign Wallet - Links to page */}
              <Link href="/group-and-permission/assign-wallet" className="block group text-left">
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-green-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                  <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl h-full">
                    <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full blur-sm opacity-60"></div>
                    <div className="p-4 sm:p-6">
                      <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-400 to-emerald-500 bg-clip-text text-transparent mb-2">
                        💼 Assign Wallet
                      </h3>
                      <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                        Assign wallet to a group
                      </p>
                      <div className="flex items-center justify-between mt-auto">
                        <div className="px-3 py-1 bg-gradient-to-r from-green-400 to-emerald-500 text-white rounded-full text-xs font-medium">
                          Open
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                      </div>
                    </div>
                  </div>
                </div>
              </Link>

              {/* Expiring Soon - Links to page */}
              <Link href="/group-and-permission/expiring" className="block group text-left">
                <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-orange-400/20 via-pink-400/20 to-orange-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                  <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl h-full">
                    <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-orange-400 to-pink-500 rounded-full blur-sm opacity-60"></div>
                    <div className="p-4 sm:p-6">
                      <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-orange-400 to-pink-500 bg-clip-text text-transparent mb-2">
                        ⏰ Expiring Soon
                      </h3>
                      <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                        View expiring assignments
                      </p>
                      <div className="flex items-center justify-between mt-auto">
                        <div className="px-3 py-1 bg-gradient-to-r from-orange-400 to-pink-500 text-white rounded-full text-xs font-medium">
                          Open
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                      </div>
                    </div>
                  </div>
                </div>
              </Link>
            </div>


            {/* Permission Groups Grid */}
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50">
              <h2 className="text-2xl font-bold text-indigo-600 dark:text-indigo-400 mb-6 flex items-center gap-2">
                <Shield className="w-6 h-6" />
                Permission Groups
              </h2>

              {groupsError ? (
                <div className="bg-red-50/50 dark:bg-red-900/20 border border-red-200/50 dark:border-red-700/50 rounded-2xl p-4">
                  <p className="text-sm text-gray-800 dark:text-gray-200">
                    Failed to load permission groups: {groupsError.message}
                  </p>
                </div>
              ) : (
                <div className="grid gap-3 sm:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
                  {permissionGroups.map((group) => {
                    const isSystem = group.group_type === 'system' || group.is_system_group;
                    const borderColor = isSystem
                      ? 'border-purple-300/50 dark:border-purple-700/50'
                      : 'border-blue-300/50 dark:border-blue-700/50'

                    return (
                      <div key={group.id} className={`bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-xl sm:rounded-2xl p-3 sm:p-4 lg:p-6 shadow-xl border-2 ${borderColor} hover:shadow-2xl transition-shadow`}>
                        <div className="flex items-center justify-between mb-3 sm:mb-4">
                          <div className="text-2xl sm:text-3xl">
                            {isSystem ? '⚙️' : '👥'}
                          </div>
                          <div className="flex items-center gap-2">
                            {isSystem && (
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
                              <p className="text-xs text-gray-600 dark:text-gray-300 line-clamp-2">
                                {group.description}
                              </p>
                            )}
                          </div>

                          <div className="space-y-2 text-sm">
                            <div className="flex justify-between items-center">
                              <span className="text-gray-700 dark:text-gray-300">Members</span>
                              <span className="font-semibold text-blue-600">{group.member_count ?? 0}</span>
                            </div>
                            <div className="flex justify-between items-center">
                              <span className="text-gray-700 dark:text-gray-300">Permissions</span>
                              <span className="font-semibold text-green-600">{group.permissions.length}</span>
                            </div>
                          </div>

                          <div className="flex flex-col gap-2 pt-2">
                            <Link href={`/group-and-permission/groups/${group.id}/edit`} className="w-full">
                              <Button
                                size="sm"
                                variant="outline"
                                className="w-full justify-center px-4 text-blue-600 border-blue-200 hover:bg-blue-50 hover:text-blue-700 dark:border-blue-800 dark:hover:bg-blue-900/20 dark:text-blue-400"
                              >
                                <Edit3 className="w-4 h-4 mr-2" />
                                Edit Group Settings
                              </Button>
                            </Link>
                            <Link href={`/group-and-permission/groups/${group.id}/members`} className="w-full">
                              <Button
                                size="sm"
                                variant="outline"
                                className="w-full justify-center px-4 text-indigo-600 border-indigo-200 hover:bg-indigo-50 hover:text-indigo-700 dark:border-indigo-800 dark:hover:bg-indigo-900/20 dark:text-indigo-400"
                              >
                                <Users className="w-4 h-4 mr-2" />
                                Manage Members
                              </Button>
                            </Link>
                            {(group.group_type !== 'system' && !group.is_system_group) && (
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => handleDeleteGroup(group.id, group.name)}
                                disabled={deleteGroupMutation.isPending}
                                className="w-full justify-center px-4 text-red-600 border-red-200 hover:bg-red-50 hover:text-red-700 dark:border-red-800 dark:hover:bg-red-900/20 dark:text-red-400"
                              >
                                <Trash2 className="w-4 h-4 mr-2" />
                                Delete Group
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

            {/* Recent Activity Section */}
            <div className="mt-8 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50">
              <div className="flex items-center gap-2 mb-4">
                <Clock className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
                <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100">Recent Activity</h3>
              </div>
              {isLoadingActivity ? (
                <div className="text-center py-8">
                  <div className="h-6 w-6 border-b-2 border-indigo-600 mx-auto mb-2 animate-spin rounded-full"></div>
                  <p className="text-sm text-gray-500">Loading activity...</p>
                </div>
              ) : recentActivity.length === 0 ? (
                <p className="text-gray-500 dark:text-gray-400 text-sm">No recent activity found.</p>
              ) : (
                <div className="space-y-3">
                  {recentActivity.map((activity, idx) => (
                    <div key={idx} className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
                      <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-gray-100">
                          {activity.user_id ? `${activity.user_id.substring(0, 6)}...${activity.user_id.substring(activity.user_id.length - 4)}` : 'Unknown User'}
                        </div>
                        <div className="text-xs text-gray-500">
                          {activity.action_type === 'ASSIGN' ? 'Assigned to' : 'Removed from'} <span className="font-medium">{activity.group_name}</span>
                        </div>
                      </div>
                      <div className="text-xs text-gray-400">
                        {new Date(activity.timestamp).toLocaleDateString()}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}

        {/* Global Permission Registry View */}
        {activeView === 'permissions' && (
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
            <PermissionRegistry />
          </div>
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

        {/* Delete Confirmation Modal */}
        {deleteConfirm && (
          <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl border border-gray-200 dark:border-gray-700">
              <div className="flex items-center gap-3 mb-4">
                <div className="p-3 bg-red-100 dark:bg-red-900/30 rounded-full">
                  <Trash2 className="w-6 h-6 text-red-600 dark:text-red-400" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  Delete Permission Group
                </h3>
              </div>
              <p className="text-gray-600 dark:text-gray-400 mb-6">
                Are you sure you want to delete <strong>"{deleteConfirm.groupName}"</strong>?
                This action cannot be undone.
              </p>
              <div className="flex gap-3">
                <Button
                  variant="outline"
                  onClick={cancelDelete}
                  className="flex-1"
                >
                  Cancel
                </Button>
                <Button
                  variant="destructive"
                  onClick={confirmDelete}
                  disabled={deleteGroupMutation.isPending}
                  className="flex-1 bg-red-600 hover:bg-red-700 text-white"
                >
                  {deleteGroupMutation.isPending ? 'Deleting...' : 'Delete'}
                </Button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div >
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

  // Fetch available permissions dynamically
  const { data: availablePermissions = [] } = useQuery({
    queryKey: ['available-permissions'],
    queryFn: () => groupMgmt.getAvailablePermissions(),
    staleTime: 5 * 60 * 1000
  })

  const [selectedPermission, setSelectedPermission] = useState('')

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
                {availablePermissions.map((perm: string) => (
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

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">
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
          <WalletAutocomplete
            value={formData.wallet_address}
            onChange={(value) => setFormData(prev => ({ ...prev, wallet_address: value }))}
            placeholder="Enter wallet address (0x...)"
            excludeGroupId={formData.group_id}
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
    name: group.name,
    description: group.description,
    permissions: group.permissions,
    default_expiry_days: group.default_expiry_days?.toString() || '',
    priority_level: group.priority_level || 0
  })

  // Fetch available permissions dynamically
  const { data: availablePermissions = [] } = useQuery({
    queryKey: ['available-permissions'],
    queryFn: () => groupMgmt.getAvailablePermissions(),
    staleTime: 5 * 60 * 1000
  })

  const [selectedPermission, setSelectedPermission] = useState('')

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
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
      <h2 className="text-2xl font-bold text-blue-600 dark:text-blue-400 mb-6 flex items-center gap-2">
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
                {availablePermissions.map((perm: string) => (
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

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">
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

// Expiring Assignments Section Component
function ExpiringAssignmentsSection({ assignments, isLoading, onUpdate }: { assignments: any[], isLoading: boolean, onUpdate: () => void }) {
  // Logic from original component or simplified for this context
  if (isLoading) return <div>Loading...</div>
  return (
    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50">
      <h2 className="text-2xl font-bold text-orange-600 dark:text-orange-400 mb-6 flex items-center gap-2">
        <Clock className="w-6 h-6" />
        Expiring Assignments (Next 7 Days)
      </h2>

      {assignments.length === 0 ? (
        <p className="text-gray-500">No assignments expiring soon.</p>
      ) : (
        <div className="space-y-4">
          {assignments.map((assignment: any) => (
            <div key={assignment.id} className="flex justify-between items-center p-3 border rounded-lg">
              <div>
                <p className="font-medium">{assignment.user_id}</p>
                <p className="text-sm text-gray-500">Group: {assignment.group?.name}</p>
              </div>
              <div className="text-right">
                <p className="text-orange-600 font-medium">{new Date(assignment.expires_at).toLocaleDateString()}</p>
                <p className="text-xs text-gray-400">Expires</p>
              </div>
            </div>
          ))}
        </div>
      )}
      <Button onClick={onUpdate} variant="outline" className="mt-4">Refresh</Button>
    </div>
  )
}
