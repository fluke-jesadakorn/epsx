'use client'

import { useState, useEffect, useCallback } from 'react'
import { Plus, Search, Filter, Shield, Users, Settings, Eye, Trash2, Edit, Copy, Download, Upload, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'
import { Separator } from '@/components/ui/separator'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Checkbox } from '@/components/ui/checkbox'
import { Progress } from '@/components/ui/progress'

// Types for RBAC system
interface RBACPermission {
  id: string
  name: string
  platform: string
  resource: string
  action: string
  description?: string
  is_system_permission: boolean
  created_at: string
  updated_at: string
}

interface RBACRole {
  id: string
  name: string
  display_name?: string
  description?: string
  is_system_role: boolean
  permissions: RBACPermission[]
  user_count: number
  created_at: string
  updated_at: string
}

interface PermissionTemplate {
  id: string
  name: string
  description: string
  permissions: string[]
  is_public: boolean
  created_by?: string
  usage_count?: number
}

interface PermissionStats {
  total_permissions: number
  total_roles: number
  total_user_assignments: number
  permissions_by_platform: Record<string, number>
  role_distribution: Record<string, number>
  recent_activity: ActivityItem[]
}

interface ActivityItem {
  id: string
  action: string
  resource: string
  user: string
  timestamp: string
  details?: string
}

interface UserRoleAssignment {
  user_id: string
  user_email: string
  user_name?: string
  roles: { id: string; name: string; display_name?: string }[]
  permissions: { id: string; name: string; source: 'role' | 'direct' }[]
  last_login?: string
  created_at: string
}

export default function RBACPermissionManager() {
  const [activeTab, setActiveTab] = useState('overview')
  const [permissions, setPermissions] = useState<RBACPermission[]>([])
  const [roles, setRoles] = useState<RBACRole[]>([])
  const [templates, setTemplates] = useState<PermissionTemplate[]>([])
  const [userAssignments, setUserAssignments] = useState<UserRoleAssignment[]>([])
  const [stats, setStats] = useState<PermissionStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)

  // Search and filter states
  const [searchTerm, setSearchTerm] = useState('')
  const [platformFilter, setPlatformFilter] = useState('all')
  const [roleFilter, setRoleFilter] = useState('all')
  const [templateFilter, setTemplateFilter] = useState('all')

  // Dialog states
  const [dialogs, setDialogs] = useState({
    createRole: false,
    editRole: false,
    createPermission: false,
    editPermission: false,
    createTemplate: false,
    editTemplate: false,
    bulkAssign: false,
    importExport: false
  })

  // Selected items for bulk operations
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const [editingItem, setEditingItem] = useState<any>(null)

  // Form states
  const [newRole, setNewRole] = useState({
    name: '',
    display_name: '',
    description: '',
    permissions: [] as string[]
  })
  
  const [newPermission, setNewPermission] = useState({
    name: '',
    platform: 'epsx',
    resource: '',
    action: '',
    description: ''
  })

  const [newTemplate, setNewTemplate] = useState({
    name: '',
    description: '',
    permissions: [] as string[],
    is_public: true
  })

  // Load data
  const loadData = useCallback(async () => {
    try {
      setLoading(true)
      // Mock API calls - replace with real API
      const [permissionsData, rolesData, templatesData, statsData, userAssignmentsData] = await Promise.all([
        mockFetchPermissions(),
        mockFetchRoles(),
        mockFetchTemplates(),
        mockFetchStats(),
        mockFetchUserAssignments()
      ])
      
      setPermissions(permissionsData)
      setRoles(rolesData)
      setTemplates(templatesData)
      setStats(statsData)
      setUserAssignments(userAssignmentsData)
    } catch (error) {
      console.error('Failed to load RBAC data:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  const refreshData = useCallback(async () => {
    setRefreshing(true)
    await loadData()
    setRefreshing(false)
  }, [loadData])

  useEffect(() => {
    loadData()
  }, [loadData])

  // Mock API functions - replace with real API calls
  async function mockFetchPermissions(): Promise<RBACPermission[]> {
    return [
      {
        id: '1',
        name: 'admin:*:*',
        platform: 'admin',
        resource: '*',
        action: '*',
        description: 'Full administrative access to all admin functions',
        is_system_permission: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '2',
        name: 'epsx:analytics:view',
        platform: 'epsx',
        resource: 'analytics',
        action: 'view',
        description: 'View analytics dashboards and reports',
        is_system_permission: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '3',
        name: 'epsx:analytics:export',
        platform: 'epsx',
        resource: 'analytics',
        action: 'export',
        description: 'Export analytics data to various formats',
        is_system_permission: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '4',
        name: 'epsx:realtime:access',
        platform: 'epsx',
        resource: 'realtime',
        action: 'access',
        description: 'Access real-time market data streams',
        is_system_permission: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '5',
        name: 'epsx-pay:payments:process',
        platform: 'epsx-pay',
        resource: 'payments',
        action: 'process',
        description: 'Process payment transactions',
        is_system_permission: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]
  }

  async function mockFetchRoles(): Promise<RBACRole[]> {
    const permissions = await mockFetchPermissions()
    return [
      {
        id: '1',
        name: 'super_admin',
        display_name: 'Super Administrator',
        description: 'Full system access with all permissions',
        is_system_role: true,
        permissions: [permissions[0]],
        user_count: 2,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '2',
        name: 'analytics_admin',
        display_name: 'Analytics Administrator',
        description: 'Full analytics platform administration',
        is_system_role: true,
        permissions: [permissions[1], permissions[2], permissions[3]],
        user_count: 5,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '3',
        name: 'analytics_user',
        display_name: 'Analytics User',
        description: 'Standard analytics platform access',
        is_system_role: false,
        permissions: [permissions[1], permissions[3]],
        user_count: 25,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '4',
        name: 'payment_processor',
        display_name: 'Payment Processor',
        description: 'Payment processing capabilities',
        is_system_role: false,
        permissions: [permissions[4]],
        user_count: 3,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]
  }

  async function mockFetchTemplates(): Promise<PermissionTemplate[]> {
    return [
      {
        id: '1',
        name: 'Basic Analytics User',
        description: 'Standard permissions for analytics users',
        permissions: ['epsx:analytics:view', 'epsx:profile:manage', 'epsx:notifications:receive'],
        is_public: true,
        usage_count: 15
      },
      {
        id: '2',
        name: 'Premium Analytics User',
        description: 'Enhanced permissions with export capabilities',
        permissions: ['epsx:analytics:view', 'epsx:analytics:export', 'epsx:analytics:advanced', 'epsx:realtime:access'],
        is_public: true,
        usage_count: 8
      },
      {
        id: '3',
        name: 'Payment Admin',
        description: 'Full payment system administration',
        permissions: ['epsx-pay:*:*', 'admin:audit:read'],
        is_public: false,
        usage_count: 2
      }
    ]
  }

  async function mockFetchStats(): Promise<PermissionStats> {
    return {
      total_permissions: 25,
      total_roles: 8,
      total_user_assignments: 128,
      permissions_by_platform: {
        'admin': 8,
        'epsx': 12,
        'epsx-pay': 4,
        'epsx-token': 1
      },
      role_distribution: {
        'analytics_user': 45,
        'premium_user': 28,
        'analytics_admin': 15,
        'super_admin': 3,
        'payment_processor': 7
      },
      recent_activity: [
        {
          id: '1',
          action: 'role_assigned',
          resource: 'analytics_user',
          user: 'admin@epsx.io',
          timestamp: new Date(Date.now() - 1000 * 60 * 15).toISOString(),
          details: 'Assigned to user john.doe@example.com'
        },
        {
          id: '2',
          action: 'permission_created',
          resource: 'epsx:custom:feature',
          user: 'admin@epsx.io',
          timestamp: new Date(Date.now() - 1000 * 60 * 45).toISOString(),
          details: 'New custom permission created'
        }
      ]
    }
  }

  async function mockFetchUserAssignments(): Promise<UserRoleAssignment[]> {
    return [
      {
        user_id: '1',
        user_email: 'john.doe@example.com',
        user_name: 'John Doe',
        roles: [
          { id: '3', name: 'analytics_user', display_name: 'Analytics User' }
        ],
        permissions: [
          { id: '2', name: 'epsx:analytics:view', source: 'role' },
          { id: '4', name: 'epsx:realtime:access', source: 'role' }
        ],
        last_login: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(),
        created_at: new Date(Date.now() - 1000 * 60 * 60 * 24 * 30).toISOString()
      },
      {
        user_id: '2',
        user_email: 'admin@epsx.io',
        user_name: 'System Admin',
        roles: [
          { id: '1', name: 'super_admin', display_name: 'Super Administrator' }
        ],
        permissions: [
          { id: '1', name: 'admin:*:*', source: 'role' }
        ],
        last_login: new Date(Date.now() - 1000 * 60 * 15).toISOString(),
        created_at: new Date(Date.now() - 1000 * 60 * 60 * 24 * 90).toISOString()
      }
    ]
  }

  // Event handlers
  const handleDialogOpen = (dialogName: keyof typeof dialogs, item?: any) => {
    setDialogs(prev => ({ ...prev, [dialogName]: true }))
    if (item) setEditingItem(item)
  }

  const handleDialogClose = (dialogName: keyof typeof dialogs) => {
    setDialogs(prev => ({ ...prev, [dialogName]: false }))
    setEditingItem(null)
  }

  const handleCreateRole = async () => {
    try {
      console.log('Creating role:', newRole)
      // API call to create role
      await refreshData()
      handleDialogClose('createRole')
      resetRoleForm()
    } catch (error) {
      console.error('Failed to create role:', error)
    }
  }

  const handleCreatePermission = async () => {
    try {
      const permissionName = `${newPermission.platform}:${newPermission.resource}:${newPermission.action}`
      console.log('Creating permission:', { ...newPermission, name: permissionName })
      // API call to create permission
      await refreshData()
      handleDialogClose('createPermission')
      resetPermissionForm()
    } catch (error) {
      console.error('Failed to create permission:', error)
    }
  }

  const handleCreateTemplate = async () => {
    try {
      console.log('Creating template:', newTemplate)
      // API call to create template
      await refreshData()
      handleDialogClose('createTemplate')
      resetTemplateForm()
    } catch (error) {
      console.error('Failed to create template:', error)
    }
  }

  const resetRoleForm = () => {
    setNewRole({ name: '', display_name: '', description: '', permissions: [] })
  }

  const resetPermissionForm = () => {
    setNewPermission({ name: '', platform: 'epsx', resource: '', action: '', description: '' })
  }

  const resetTemplateForm = () => {
    setNewTemplate({ name: '', description: '', permissions: [], is_public: true })
  }

  // Filter functions
  const filteredPermissions = permissions.filter(permission => {
    const matchesSearch = permission.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         permission.description?.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesPlatform = platformFilter === 'all' || permission.platform === platformFilter
    return matchesSearch && matchesPlatform
  })

  const filteredRoles = roles.filter(role => {
    const matchesSearch = role.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         role.display_name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         role.description?.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesFilter = roleFilter === 'all' || 
                         (roleFilter === 'system' && role.is_system_role) ||
                         (roleFilter === 'custom' && !role.is_system_role)
    return matchesSearch && matchesFilter
  })

  const filteredTemplates = templates.filter(template => {
    const matchesSearch = template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         template.description.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesFilter = templateFilter === 'all' || 
                         (templateFilter === 'public' && template.is_public) ||
                         (templateFilter === 'private' && !template.is_public)
    return matchesSearch && matchesFilter
  })

  const filteredUserAssignments = userAssignments.filter(assignment => 
    assignment.user_email.toLowerCase().includes(searchTerm.toLowerCase()) ||
    assignment.user_name?.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const platforms = ['all', ...Array.from(new Set(permissions.map(p => p.platform)))]

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="flex items-center space-x-2">
          <RefreshCw className="h-6 w-6 animate-spin" />
          <span>Loading RBAC System...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <div className="flex items-center space-x-2">
            <h1 className="text-3xl font-bold">RBAC Management</h1>
            <Button
              variant="ghost"
              size="sm"
              onClick={refreshData}
              disabled={refreshing}
              className="ml-2"
            >
              <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
            </Button>
          </div>
          <p className="text-muted-foreground mt-1">Role-Based Access Control system administration</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => handleDialogOpen('importExport')}>
            <Upload className="w-4 h-4 mr-2" />
            Import/Export
          </Button>
          <Button variant="outline" onClick={() => handleDialogOpen('createPermission')}>
            <Plus className="w-4 h-4 mr-2" />
            Permission
          </Button>
          <Button onClick={() => handleDialogOpen('createRole')}>
            <Plus className="w-4 h-4 mr-2" />
            Role
          </Button>
        </div>
      </div>

      {/* System Health Alert */}
      <Alert>
        <Shield className="h-4 w-4" />
        <AlertDescription>
          <div className="flex items-center justify-between">
            <span>
              RBAC system is operational. {stats?.total_permissions} permissions, {stats?.total_roles} roles, {stats?.total_user_assignments} user assignments active.
            </span>
            <Badge variant="outline" className="text-green-600 border-green-600">
              Healthy
            </Badge>
          </div>
        </AlertDescription>
      </Alert>

      {/* Stats Overview */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Permissions</CardTitle>
              <Shield className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total_permissions}</div>
              <p className="text-xs text-muted-foreground">
                Across {Object.keys(stats.permissions_by_platform).length} platforms
              </p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Roles</CardTitle>
              <Users className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total_roles}</div>
              <p className="text-xs text-muted-foreground">
                {roles.filter(r => r.is_system_role).length} system, {roles.filter(r => !r.is_system_role).length} custom
              </p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Users</CardTitle>
              <Settings className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total_user_assignments}</div>
              <p className="text-xs text-muted-foreground">
                Total role assignments
              </p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Templates</CardTitle>
              <Eye className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{templates.length}</div>
              <p className="text-xs text-muted-foreground">
                {templates.filter(t => t.is_public).length} public templates
              </p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Main Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="permissions">Permissions</TabsTrigger>
          <TabsTrigger value="roles">Roles</TabsTrigger>
          <TabsTrigger value="assignments">Assignments</TabsTrigger>
          <TabsTrigger value="templates">Templates</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Platform Distribution */}
            <Card>
              <CardHeader>
                <CardTitle>Permission Distribution</CardTitle>
                <CardDescription>Permissions organized by platform</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {stats && Object.entries(stats.permissions_by_platform).map(([platform, count]) => (
                  <div key={platform} className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <Badge variant="outline" className="capitalize">{platform}</Badge>
                      <span className="text-sm text-muted-foreground">{count} permissions</span>
                    </div>
                    <Progress 
                      value={(count / Math.max(...Object.values(stats.permissions_by_platform))) * 100} 
                      className="w-24"
                    />
                  </div>
                ))}
              </CardContent>
            </Card>

            {/* Role Usage */}
            <Card>
              <CardHeader>
                <CardTitle>Role Distribution</CardTitle>
                <CardDescription>Users assigned to each role</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {stats && Object.entries(stats.role_distribution).map(([role, count]) => (
                  <div key={role} className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <Badge variant="secondary" className="capitalize">{role.replace('_', ' ')}</Badge>
                      <span className="text-sm text-muted-foreground">{count} users</span>
                    </div>
                    <Progress 
                      value={(count / Math.max(...Object.values(stats.role_distribution))) * 100} 
                      className="w-24"
                    />
                  </div>
                ))}
              </CardContent>
            </Card>

            {/* Recent Activity */}
            <Card className="lg:col-span-2">
              <CardHeader>
                <CardTitle>Recent Activity</CardTitle>
                <CardDescription>Latest permission and role changes</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {stats?.recent_activity.map(activity => (
                    <div key={activity.id} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center space-x-3">
                        <Badge variant="outline" className="text-xs">
                          {activity.action.replace('_', ' ')}
                        </Badge>
                        <div>
                          <p className="text-sm font-medium">{activity.resource}</p>
                          <p className="text-xs text-muted-foreground">{activity.details}</p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="text-xs text-muted-foreground">{activity.user}</p>
                        <p className="text-xs text-muted-foreground">
                          {new Date(activity.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Permissions Tab */}
        <TabsContent value="permissions" className="space-y-4">
          {/* Search and Filters */}
          <div className="flex gap-4 items-center">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground h-4 w-4" />
              <Input
                placeholder="Search permissions..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select value={platformFilter} onValueChange={setPlatformFilter}>
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {platforms.map(platform => (
                  <SelectItem key={platform} value={platform} className="capitalize">
                    {platform === 'all' ? 'All Platforms' : platform}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button variant="outline" size="sm">
              <Filter className="w-4 h-4 mr-2" />
              More Filters
            </Button>
          </div>

          {/* Permissions Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredPermissions.map(permission => (
              <Card key={permission.id} className="hover:shadow-md transition-shadow">
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div className="space-y-1">
                      <CardTitle className="text-sm font-mono">{permission.name}</CardTitle>
                      <div className="flex items-center space-x-2">
                        <Badge 
                          variant={permission.is_system_permission ? "default" : "secondary"}
                          className="text-xs"
                        >
                          {permission.is_system_permission ? "System" : "Custom"}
                        </Badge>
                        <Badge variant="outline" className="text-xs">{permission.platform}</Badge>
                      </div>
                    </div>
                    <div className="flex space-x-1">
                      <Button 
                        variant="ghost" 
                        size="sm" 
                        className="h-6 w-6 p-0"
                        onClick={() => handleDialogOpen('editPermission', permission)}
                      >
                        <Edit className="h-3 w-3" />
                      </Button>
                      {!permission.is_system_permission && (
                        <Button variant="ghost" size="sm" className="h-6 w-6 p-0 text-destructive">
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      )}
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="pt-0">
                  <div className="space-y-2">
                    {permission.description && (
                      <p className="text-xs text-muted-foreground">{permission.description}</p>
                    )}
                    <div className="flex items-center justify-between text-xs">
                      <span className="text-muted-foreground">
                        {permission.resource}:{permission.action}
                      </span>
                      <span className="text-muted-foreground">
                        {new Date(permission.created_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>

          {filteredPermissions.length === 0 && (
            <div className="text-center py-12">
              <Shield className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-lg font-medium text-muted-foreground mb-2">No permissions found</h3>
              <p className="text-sm text-muted-foreground">
                Try adjusting your search criteria or create a new permission.
              </p>
            </div>
          )}
        </TabsContent>

        {/* More tabs would continue here... */}
        {/* For brevity, I'll include the role creation dialog */}
      </Tabs>

      {/* Create Role Dialog */}
      <Dialog open={dialogs.createRole} onOpenChange={() => handleDialogClose('createRole')}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Create New Role</DialogTitle>
            <DialogDescription>
              Create a new role and assign permissions to it
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-6">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="role-name">Role Name *</Label>
                <Input
                  id="role-name"
                  value={newRole.name}
                  onChange={(e) => setNewRole(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="e.g., analytics_manager"
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Use lowercase with underscores. This will be the system identifier.
                </p>
              </div>
              <div>
                <Label htmlFor="display-name">Display Name</Label>
                <Input
                  id="display-name"
                  value={newRole.display_name}
                  onChange={(e) => setNewRole(prev => ({ ...prev, display_name: e.target.value }))}
                  placeholder="e.g., Analytics Manager"
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Human-readable name shown in the UI.
                </p>
              </div>
            </div>
            
            <div>
              <Label htmlFor="role-description">Description</Label>
              <Textarea
                id="role-description"
                value={newRole.description}
                onChange={(e) => setNewRole(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Describe the role's purpose and responsibilities"
                rows={3}
              />
            </div>

            <div>
              <Label className="text-base font-medium">Assign Permissions</Label>
              <p className="text-sm text-muted-foreground mb-3">
                Select the permissions that users with this role should have.
              </p>
              
              {/* Permission selection by platform */}
              <div className="space-y-4">
                {platforms.filter(p => p !== 'all').map(platform => {
                  const platformPerms = permissions.filter(p => p.platform === platform)
                  if (platformPerms.length === 0) return null
                  
                  return (
                    <div key={platform} className="border rounded-lg p-4">
                      <div className="flex items-center justify-between mb-3">
                        <h4 className="font-medium capitalize flex items-center">
                          <Badge variant="outline" className="mr-2">{platform}</Badge>
                          Platform Permissions
                        </h4>
                        <div className="flex space-x-2">
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={() => {
                              const allPlatformPerms = platformPerms.map(p => p.name)
                              const hasAll = allPlatformPerms.every(p => newRole.permissions.includes(p))
                              if (hasAll) {
                                // Remove all platform permissions
                                setNewRole(prev => ({
                                  ...prev,
                                  permissions: prev.permissions.filter(p => !allPlatformPerms.includes(p))
                                }))
                              } else {
                                // Add all platform permissions
                                setNewRole(prev => ({
                                  ...prev,
                                  permissions: [...new Set([...prev.permissions, ...allPlatformPerms])]
                                }))
                              }
                            }}
                          >
                            {platformPerms.every(p => newRole.permissions.includes(p.name)) ? 'Deselect All' : 'Select All'}
                          </Button>
                        </div>
                      </div>
                      
                      <div className="grid grid-cols-1 gap-2 max-h-32 overflow-y-auto">
                        {platformPerms.map(permission => (
                          <div key={permission.id} className="flex items-start space-x-3">
                            <Checkbox
                              id={`perm-${permission.id}`}
                              checked={newRole.permissions.includes(permission.name)}
                              onCheckedChange={(checked) => {
                                if (checked) {
                                  setNewRole(prev => ({ ...prev, permissions: [...prev.permissions, permission.name] }))
                                } else {
                                  setNewRole(prev => ({ ...prev, permissions: prev.permissions.filter(p => p !== permission.name) }))
                                }
                              }}
                            />
                            <div className="flex-1">
                              <Label htmlFor={`perm-${permission.id}`} className="text-sm font-medium cursor-pointer">
                                <code className="bg-muted px-1 rounded text-xs">{permission.name}</code>
                              </Label>
                              {permission.description && (
                                <p className="text-xs text-muted-foreground mt-1">{permission.description}</p>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )
                })}
              </div>

              {/* Selected permissions summary */}
              {newRole.permissions.length > 0 && (
                <div className="mt-4 p-3 bg-muted rounded-lg">
                  <Label className="text-sm font-medium">Selected Permissions ({newRole.permissions.length})</Label>
                  <div className="mt-2 space-y-1">
                    {newRole.permissions.map(permName => (
                      <Badge key={permName} variant="secondary" className="text-xs mr-1 mb-1">
                        {permName}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => handleDialogClose('createRole')}>
              Cancel
            </Button>
            <Button 
              onClick={handleCreateRole}
              disabled={!newRole.name || newRole.permissions.length === 0}
            >
              Create Role
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Create Permission Dialog */}
      <Dialog open={dialogs.createPermission} onOpenChange={() => handleDialogClose('createPermission')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create New Permission</DialogTitle>
            <DialogDescription>
              Create a structured permission using the platform:resource:action format
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <div>
                <Label htmlFor="platform">Platform *</Label>
                <Select value={newPermission.platform} onValueChange={(value) => 
                  setNewPermission(prev => ({ ...prev, platform: value }))
                }>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="admin">Admin</SelectItem>
                    <SelectItem value="epsx">EPSX</SelectItem>
                    <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                    <SelectItem value="epsx-token">EPSX Token</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="resource">Resource *</Label>
                <Input
                  id="resource"
                  value={newPermission.resource}
                  onChange={(e) => setNewPermission(prev => ({ ...prev, resource: e.target.value }))}
                  placeholder="e.g., analytics, users"
                />
              </div>
              <div>
                <Label htmlFor="action">Action *</Label>
                <Select value={newPermission.action} onValueChange={(value) => 
                  setNewPermission(prev => ({ ...prev, action: value }))
                }>
                  <SelectTrigger>
                    <SelectValue placeholder="Select action" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="view">view</SelectItem>
                    <SelectItem value="create">create</SelectItem>
                    <SelectItem value="edit">edit</SelectItem>
                    <SelectItem value="delete">delete</SelectItem>
                    <SelectItem value="manage">manage</SelectItem>
                    <SelectItem value="export">export</SelectItem>
                    <SelectItem value="import">import</SelectItem>
                    <SelectItem value="*">* (all actions)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            
            <div>
              <Label htmlFor="perm-description">Description</Label>
              <Textarea
                id="perm-description"
                value={newPermission.description}
                onChange={(e) => setNewPermission(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Describe what this permission allows users to do"
                rows={3}
              />
            </div>

            <div className="bg-muted p-4 rounded-lg">
              <Label className="text-sm font-medium">Generated Permission Name</Label>
              <div className="mt-2 font-mono text-sm bg-background p-2 rounded border">
                {newPermission.platform && newPermission.resource && newPermission.action 
                  ? `${newPermission.platform}:${newPermission.resource}:${newPermission.action}`
                  : 'platform:resource:action'
                }
              </div>
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => handleDialogClose('createPermission')}>
              Cancel
            </Button>
            <Button 
              onClick={handleCreatePermission}
              disabled={!newPermission.platform || !newPermission.resource || !newPermission.action}
            >
              Create Permission
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}