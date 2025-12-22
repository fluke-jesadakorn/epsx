/**
 * Group Manager Component
 * Main interface for managing permission groups in the Web3 group-based system
 * 
 * Features:
 * - Create, edit, delete permission groups
 * - Manage group memberships
 * - View group analytics and insights
 * - Handle Web3 auto-assignment rules
 */

'use client'

import {
  Activity,
  AlertTriangle, CheckCircle,
  Clock,
  Edit,
  Filter, MoreHorizontal,
  Plus,
  Search,
  Star,
  Trash2,
  Users
} from 'lucide-react'
import { useCallback, useMemo, useState } from 'react'

import { GroupEditor } from './GroupEditor'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle
} from '@/components/ui/dialog'
import {
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useToast } from '@/components/ui/use-toast'
import { adminButtonVariants, adminCardVariants } from '@/design-system'
import {
  useGroupAnalytics,
  useGroups
} from '@/hooks/useGroupPermissions'
import { Group } from '@/lib/api/group-management-client'
import { cn } from '@/lib/shared'

// Sub-components

// Stub type for build compatibility
type GroupPermissionStats = any;

interface GroupManagerProps {
  className?: string
}

/**
 *
 * @param root0
 * @param root0.className
 */
export function GroupManager({ className }: GroupManagerProps) {
  const { toast } = useToast()
  const [activeTab, setActiveTab] = useState('all')
  const [searchTerm, setSearchTerm] = useState('')
  const [showGroupEditor, setShowGroupEditor] = useState(false)
  const [editingGroup, setEditingGroup] = useState<Group | null>(null)

  // Permission check bypassed - backend handles authorization
  const canManageGroups = true;

  // Hooks
  const {
    groups,
    loading: isLoading,
    error,
    deleteGroup
  } = useGroups()

  const { stats } = useGroupAnalytics()
  // Backend handles permission checking - no client-side validation needed

  // Filter groups based on search
  const filteredGroups = useMemo(() => {
    if (!searchTerm) { return groups }
    const searchLower = searchTerm.toLowerCase()
    return groups.filter(group =>
      group.name.toLowerCase().includes(searchLower) ||
      group.description?.toLowerCase().includes(searchLower) ||
      group.permissions.some(p => p.toLowerCase().includes(searchLower))
    )
  }, [groups, searchTerm])

  // Separate system and custom groups
  const systemGroups = useMemo(() => {
    return groups.filter(group => group.group_type === 'system' || group.group_type === 'admin')
  }, [groups])

  const customGroups = useMemo(() => {
    return groups.filter(group => group.group_type !== 'system' && group.group_type !== 'admin')
  }, [groups])

  // Event handlers
  const handleCreateGroup = useCallback(() => {
    setEditingGroup(null)
    setShowGroupEditor(true)
  }, [])

  const handleEditGroup = useCallback((group: Group) => {
    setEditingGroup(group)
    setShowGroupEditor(true)
  }, [])

  const handleDeleteGroup = useCallback(async (group: Group) => {
    if (group.group_type === 'system' || group.group_type === 'admin') {
      toast({
        title: 'Cannot Delete System Group',
        description: 'System-managed groups cannot be deleted.',
        variant: 'destructive'
      })
      return
    }

    try {
      await deleteGroup(group.id)
      toast({
        title: 'Group Deleted',
        description: `Permission group "${group.name}" has been deleted.`
      })
    } catch (_error) {
      toast({
        title: 'Delete Failed',
        description: _error instanceof Error ? _error.message : 'Failed to delete group',
        variant: 'destructive'
      })
    }
  }, [deleteGroup, toast])

  const handleGroupEditorClose = useCallback(() => {
    setShowGroupEditor(false)
    setEditingGroup(null)
  }, [])

  const handleGroupSaved = useCallback((group: Group) => {
    setShowGroupEditor(false)
    setEditingGroup(null)
    toast({
      title: editingGroup ? 'Group Updated' : 'Group Created',
      description: `Group "${group.name}" has been ${editingGroup ? 'updated' : 'created'}.`
    })
  }, [editingGroup, toast])

  // Helper functions
  const getGroupIcon = (group: Group) => {
    if (group.group_type === 'system' || group.group_type === 'admin') { return <Star className="h-4 w-4" /> }
    return <Users className="h-4 w-4" />
  }

  const getGroupBadgeVariant = (group: Group) => {
    if (group.group_type === 'system' || group.group_type === 'admin') { return 'default' }
    if ((group.display_order || 0) > 5) { return 'destructive' }
    return 'secondary'
  }

  const getPriorityLabel = (priority: number) => {
    if (priority >= 8) { return 'Critical' }
    if (priority >= 5) { return 'High' }
    if (priority >= 3) { return 'Medium' }
    return 'Low'
  }

  // Loading state
  if (isLoading) {
    return (
      <div className={cn('space-y-6', className)}>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {[...Array(6)].map((_, i) => (
            <Card key={i} className={adminCardVariants({ variant: 'default' })}>
              <CardHeader className="space-y-2">
                <div className="h-4 bg-gray-200 rounded animate-pulse" />
                <div className="h-3 bg-gray-200 rounded w-2/3 animate-pulse" />
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="h-2 bg-gray-200 rounded animate-pulse" />
                  <div className="h-2 bg-gray-200 rounded w-1/2 animate-pulse" />
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  // Error state
  if (error) {
    return (
      <div className={cn('space-y-6', className)}>
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Failed to load permission groups: {error || 'Unknown error'}
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">Groups</h1>
          <p className="text-sm text-gray-600 mt-1">
            Manage user groups and Web3 auto-assignment rules
          </p>
        </div>
        {canManageGroups && (
          <Button
            onClick={handleCreateGroup}
            className={adminButtonVariants({ variant: 'primary', size: 'sm' })}
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Group
          </Button>
        )}
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Groups</p>
                <p className="text-2xl font-bold text-gray-900">{stats.totalGroups}</p>
              </div>
              <Users className="h-8 w-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Members</p>
                <p className="text-2xl font-bold text-gray-900">{stats.totalUsers}</p>
              </div>
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Expiring Soon</p>
                <p className="text-2xl font-bold text-gray-900">{stats.recentActivity}</p>
              </div>
              <Clock className="h-8 w-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Health Score</p>
                <p className="text-2xl font-bold text-gray-900">{stats.health_score}%</p>
              </div>
              <Activity className="h-8 w-8 text-purple-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Search and Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
          <Input
            placeholder="Search groups, permissions, or descriptions..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-10"
          />
        </div>
        <Button variant="outline" size="sm">
          <Filter className="h-4 w-4 mr-2" />
          Filters
        </Button>
      </div>

      {/* Groups Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="all">All Groups ({filteredGroups.length})</TabsTrigger>
          <TabsTrigger value="system">System ({systemGroups.length})</TabsTrigger>
          <TabsTrigger value="custom">Custom ({customGroups.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="mt-6">
          <GroupList
            groups={filteredGroups}
            onEdit={handleEditGroup}
            onDelete={handleDeleteGroup}
            canManage={canManageGroups}
          />
        </TabsContent>

        <TabsContent value="system" className="mt-6">
          <GroupList
            groups={systemGroups.filter(g =>
              !searchTerm ||
              g.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
              g.description?.toLowerCase().includes(searchTerm.toLowerCase())
            )}
            onEdit={handleEditGroup}
            onDelete={handleDeleteGroup}
            canManage={canManageGroups}
          />
        </TabsContent>

        <TabsContent value="custom" className="mt-6">
          <GroupList
            groups={customGroups.filter(g =>
              !searchTerm ||
              g.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
              g.description?.toLowerCase().includes(searchTerm.toLowerCase())
            )}
            onEdit={handleEditGroup}
            onDelete={handleDeleteGroup}
            canManage={canManageGroups}
          />
        </TabsContent>
      </Tabs>

      {/* Group Editor Dialog */}
      <Dialog open={showGroupEditor} onOpenChange={setShowGroupEditor}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editingGroup ? 'Edit Group' : 'Create Group'}
            </DialogTitle>
          </DialogHeader>
          <GroupEditor
            group={editingGroup}
            onSave={handleGroupSaved}
            onCancel={handleGroupEditorClose}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

// Group List Component
interface GroupListProps {
  groups: Group[]
  onEdit: (group: Group) => void
  onDelete: (group: Group) => void
  canManage: boolean
}

function GroupList({ groups, onEdit, onDelete, canManage }: GroupListProps) {
  if (groups.length === 0) {
    return (
      <Card className="p-8 text-center">
        <Users className="h-12 w-12 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 mb-2">No groups found</h3>
        <p className="text-gray-600">
          {canManage ? 'Create your first group to get started.' : 'No groups match your search criteria.'}
        </p>
      </Card>
    )
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
      {groups.map((group) => (
        <Card
          key={group.id}
          className={adminCardVariants({ variant: 'default' })}
        >
          <CardHeader className="pb-3">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                {group.group_type === 'system' || group.group_type === 'admin' ? (
                  <Star className="h-4 w-4 text-yellow-600" />
                ) : (
                  <Users className="h-4 w-4 text-blue-600" />
                )}
                <CardTitle className="text-base">{group.name}</CardTitle>
              </div>
              <div className="flex items-center gap-1">
                <Badge variant={group.group_type === 'system' || group.group_type === 'admin' ? 'default' : 'secondary'}>
                  {group.group_type === 'system' || group.group_type === 'admin' ? 'System' : 'Custom'}
                </Badge>
                {canManage && (
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => onEdit(group)}>
                        <Edit className="h-4 w-4 mr-2" />
                        Edit
                      </DropdownMenuItem>
                      {!(group.group_type === 'system' || group.group_type === 'admin') && (
                        <DropdownMenuItem
                          onClick={() => onDelete(group)}
                          className="text-red-600"
                        >
                          <Trash2 className="h-4 w-4 mr-2" />
                          Delete
                        </DropdownMenuItem>
                      )}
                    </DropdownMenuContent>
                  </DropdownMenu>
                )}
              </div>
            </div>
            {group.description && (
              <CardDescription className="text-xs">
                {group.description}
              </CardDescription>
            )}
          </CardHeader>

          <CardContent className="space-y-3">
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-600">Permissions</span>
              <Badge variant="outline">{group.permissions.length}</Badge>
            </div>

            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-600">Priority</span>
              <Badge variant={(group.display_order || 0) > 5 ? 'destructive' : 'secondary'}>
                {group.display_order || 0}
              </Badge>
            </div>

            {/* default_expiry_days removed from backend response, using group_metadata if needed */}

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
  )
}

export default GroupManager