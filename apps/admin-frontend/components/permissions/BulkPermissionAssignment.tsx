'use client'

import { useState, useMemo, useCallback } from 'react'
import { 
  Users, 
  Plus, 
  Minus,
  Search, 
  Filter,
  Clock,
  Shield,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Play,
  Pause,
  RotateCcw,
  Download,
  Upload,
  Eye,
  Zap,
  Calendar,
  Settings,
  User,
  Mail,
  Building,
  Tag,
  Layers,
  Activity,
  TrendingUp,
  X
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Separator } from '@/components/ui/separator'
import { Progress } from '@/components/ui/progress'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { format, addMinutes } from 'date-fns'

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface UserProfile {
  id: string
  name: string
  email: string
  role: string
  department?: string
  lastActive: Date
  currentPermissions: string[]
  isActive: boolean
}

interface BulkAssignmentTarget {
  userId: string
  user: UserProfile
  selectedTemplates: string[]
  customDuration?: number // minutes, overrides template default
  customExpiry?: Date
  reason?: string
  priority: 'low' | 'normal' | 'high' | 'urgent'
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'skipped'
  errorMessage?: string
}

interface PermissionTemplate {
  id: string
  name: string
  description: string
  basePermission: string
  platform: string
  defaultDuration: number
  category: string
  tags: string[]
}

interface BulkAssignmentJob {
  id: string
  name: string
  targets: BulkAssignmentTarget[]
  templates: PermissionTemplate[]
  createdAt: Date
  startedAt?: Date
  completedAt?: Date
  status: 'draft' | 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'
  progress: number
  successCount: number
  failureCount: number
  skipCount: number
}

interface BulkPermissionAssignmentProps {
  className?: string
}

// ============================================================================
// MOCK DATA
// ============================================================================

const MOCK_USERS: UserProfile[] = [
  {
    id: 'user-1',
    name: 'John Doe',
    email: 'john@example.com',
    role: 'Analytics Manager',
    department: 'Business Intelligence',
    lastActive: new Date('2024-01-25T10:30:00Z'),
    currentPermissions: ['epsx:analytics:view', 'epsx:dashboard:manage'],
    isActive: true
  },
  {
    id: 'user-2',
    name: 'Jane Smith',
    email: 'jane@example.com',
    role: 'Data Analyst',
    department: 'Business Intelligence',
    lastActive: new Date('2024-01-25T14:15:00Z'),
    currentPermissions: ['epsx:analytics:view'],
    isActive: true
  },
  {
    id: 'user-3',
    name: 'Bob Wilson',
    email: 'bob@example.com',
    role: 'System Administrator',
    department: 'IT',
    lastActive: new Date('2024-01-25T09:45:00Z'),
    currentPermissions: ['admin:users:manage', 'admin:system:configure'],
    isActive: true
  },
  {
    id: 'user-4',
    name: 'Alice Johnson',
    email: 'alice@example.com',
    role: 'Product Manager',
    department: 'Product',
    lastActive: new Date('2024-01-24T16:20:00Z'),
    currentPermissions: ['epsx:analytics:view', 'epsx:reports:generate'],
    isActive: true
  },
  {
    id: 'user-5',
    name: 'Charlie Brown',
    email: 'charlie@example.com',
    role: 'Sales Manager',
    department: 'Sales',
    lastActive: new Date('2024-01-25T11:30:00Z'),
    currentPermissions: ['epsx:analytics:view'],
    isActive: false
  }
]

const MOCK_TEMPLATES: PermissionTemplate[] = [
  {
    id: 'template-1',
    name: 'Analytics Viewer',
    description: 'Temporary access to analytics dashboard',
    basePermission: 'epsx:analytics:view',
    platform: 'epsx',
    defaultDuration: 240,
    category: 'analytics',
    tags: ['analytics', 'dashboard']
  },
  {
    id: 'template-2',
    name: 'Premium Rankings',
    description: 'Access to premium ranking data',
    basePermission: 'epsx:rankings:view:100',
    platform: 'epsx',
    defaultDuration: 1440,
    category: 'premium',
    tags: ['rankings', 'premium']
  },
  {
    id: 'template-3',
    name: 'Emergency Admin',
    description: 'Emergency administrative access',
    basePermission: 'admin:users:manage',
    platform: 'admin',
    defaultDuration: 480,
    category: 'emergency',
    tags: ['admin', 'emergency']
  }
]

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function BulkPermissionAssignment({ className = '' }: BulkPermissionAssignmentProps) {
  const { toast } = useToast()
  
  // State management
  const [users] = useState<UserProfile[]>(MOCK_USERS)
  const [templates] = useState<PermissionTemplate[]>(MOCK_TEMPLATES)
  const [selectedUsers, setSelectedUsers] = useState<string[]>([])
  const [selectedTemplates, setSelectedTemplates] = useState<string[]>([])
  const [assignmentTargets, setAssignmentTargets] = useState<BulkAssignmentTarget[]>([])
  const [currentJob, setCurrentJob] = useState<BulkAssignmentJob | null>(null)
  
  // Filters and search
  const [userSearchTerm, setUserSearchTerm] = useState('')
  const [templateSearchTerm, setTemplateSearchTerm] = useState('')
  const [userRoleFilter, setUserRoleFilter] = useState<string>('all')
  const [userStatusFilter, setUserStatusFilter] = useState<string>('all')
  const [templateCategoryFilter, setTemplateCategoryFilter] = useState<string>('all')
  
  // Job configuration
  const [jobName, setJobName] = useState('')
  const [globalReason, setGlobalReason] = useState('')
  const [defaultPriority, setDefaultPriority] = useState<'low' | 'normal' | 'high' | 'urgent'>('normal')
  const [scheduleType, setScheduleType] = useState<'immediate' | 'scheduled'>('immediate')
  const [scheduledTime, setScheduledTime] = useState<string>('')
  
  // UI state
  const [activeTab, setActiveTab] = useState<'users' | 'templates' | 'configure' | 'review'>('users')
  const [showPreview, setShowPreview] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)

  // Filtered data
  const filteredUsers = useMemo(() => {
    return users.filter(user => {
      if (userSearchTerm) {
        const searchLower = userSearchTerm.toLowerCase()
        if (!user.name.toLowerCase().includes(searchLower) &&
            !user.email.toLowerCase().includes(searchLower) &&
            !user.role.toLowerCase().includes(searchLower) &&
            !user.department?.toLowerCase().includes(searchLower)) {
          return false
        }
      }
      
      if (userRoleFilter !== 'all' && user.role !== userRoleFilter) {
        return false
      }
      
      if (userStatusFilter !== 'all') {
        if (userStatusFilter === 'active' && !user.isActive) return false
        if (userStatusFilter === 'inactive' && user.isActive) return false
      }
      
      return true
    })
  }, [users, userSearchTerm, userRoleFilter, userStatusFilter])

  const filteredTemplates = useMemo(() => {
    return templates.filter(template => {
      if (templateSearchTerm) {
        const searchLower = templateSearchTerm.toLowerCase()
        if (!template.name.toLowerCase().includes(searchLower) &&
            !template.description.toLowerCase().includes(searchLower) &&
            !template.basePermission.toLowerCase().includes(searchLower)) {
          return false
        }
      }
      
      if (templateCategoryFilter !== 'all' && template.category !== templateCategoryFilter) {
        return false
      }
      
      return true
    })
  }, [templates, templateSearchTerm, templateCategoryFilter])

  // Statistics
  const stats = useMemo(() => {
    return {
      selectedUsers: selectedUsers.length,
      selectedTemplates: selectedTemplates.length,
      totalAssignments: selectedUsers.length * selectedTemplates.length,
      estimatedDuration: Math.max(...selectedTemplates.map(id => 
        templates.find(t => t.id === id)?.defaultDuration || 0
      ), 0)
    }
  }, [selectedUsers, selectedTemplates, templates])

  // Event handlers
  const handleUserSelect = useCallback((userId: string, selected: boolean) => {
    if (selected) {
      setSelectedUsers(prev => [...prev, userId])
    } else {
      setSelectedUsers(prev => prev.filter(id => id !== userId))
    }
  }, [])

  const handleTemplateSelect = useCallback((templateId: string, selected: boolean) => {
    if (selected) {
      setSelectedTemplates(prev => [...prev, templateId])
    } else {
      setSelectedTemplates(prev => prev.filter(id => id !== templateId))
    }
  }, [])

  const handleSelectAllUsers = useCallback(() => {
    if (selectedUsers.length === filteredUsers.length) {
      setSelectedUsers([])
    } else {
      setSelectedUsers(filteredUsers.map(u => u.id))
    }
  }, [selectedUsers, filteredUsers])

  const handleSelectAllTemplates = useCallback(() => {
    if (selectedTemplates.length === filteredTemplates.length) {
      setSelectedTemplates([])
    } else {
      setSelectedTemplates(filteredTemplates.map(t => t.id))
    }
  }, [selectedTemplates, filteredTemplates])

  const generateAssignmentTargets = useCallback(() => {
    const targets: BulkAssignmentTarget[] = []
    
    selectedUsers.forEach(userId => {
      const user = users.find(u => u.id === userId)
      if (user) {
        targets.push({
          userId,
          user,
          selectedTemplates: [...selectedTemplates],
          reason: globalReason,
          priority: defaultPriority,
          status: 'pending'
        })
      }
    })
    
    setAssignmentTargets(targets)
    setActiveTab('configure')
  }, [selectedUsers, selectedTemplates, users, globalReason, defaultPriority])

  const handleStartBulkAssignment = async () => {
    if (!jobName.trim()) {
      toast({
        title: 'Validation Error',
        description: 'Please provide a job name',
        variant: 'destructive'
      })
      return
    }
    
    const job: BulkAssignmentJob = {
      id: `job-${Date.now()}`,
      name: jobName,
      targets: assignmentTargets,
      templates: templates.filter(t => selectedTemplates.includes(t.id)),
      createdAt: new Date(),
      status: 'queued',
      progress: 0,
      successCount: 0,
      failureCount: 0,
      skipCount: 0
    }
    
    setCurrentJob(job)
    setIsProcessing(true)
    
    // Simulate bulk processing
    setTimeout(() => {
      simulateBulkProcessing(job)
    }, 1000)
  }

  const simulateBulkProcessing = async (job: BulkAssignmentJob) => {
    const updatedJob = { ...job, status: 'running' as const, startedAt: new Date() }
    setCurrentJob(updatedJob)
    
    // Simulate processing each target
    for (let i = 0; i < updatedJob.targets.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      const target = updatedJob.targets[i]
      const success = Math.random() > 0.1 // 90% success rate
      
      updatedJob.targets[i] = {
        ...target,
        status: success ? 'completed' : 'failed',
        errorMessage: success ? undefined : 'Permission assignment failed'
      }
      
      if (success) {
        updatedJob.successCount++
      } else {
        updatedJob.failureCount++
      }
      
      updatedJob.progress = ((i + 1) / updatedJob.targets.length) * 100
      setCurrentJob({ ...updatedJob })
    }
    
    updatedJob.status = 'completed'
    updatedJob.completedAt = new Date()
    setCurrentJob({ ...updatedJob })
    setIsProcessing(false)
    
    toast({
      title: 'Bulk Assignment Complete',
      description: `Successfully assigned permissions to ${updatedJob.successCount} users`
    })
  }

  const resetAssignment = () => {
    setSelectedUsers([])
    setSelectedTemplates([])
    setAssignmentTargets([])
    setCurrentJob(null)
    setJobName('')
    setGlobalReason('')
    setActiveTab('users')
    setIsProcessing(false)
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-xl font-semibold">Bulk Permission Assignment</h3>
          <p className="text-muted-foreground">
            Assign permissions to multiple users simultaneously with embedded timestamps
          </p>
        </div>
        <div className="flex items-center gap-2">
          {currentJob && (
            <Badge variant={
              currentJob.status === 'completed' ? 'default' :
              currentJob.status === 'running' ? 'secondary' :
              currentJob.status === 'failed' ? 'destructive' : 'outline'
            }>
              {currentJob.status}
            </Badge>
          )}
          <Button variant="outline" size="sm" onClick={resetAssignment}>
            <RotateCcw className="h-4 w-4 mr-2" />
            Reset
          </Button>
        </div>
      </div>

      {/* Progress Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Users className="h-5 w-5 text-blue-500" />
              <div>
                <p className="text-2xl font-bold">{stats.selectedUsers}</p>
                <p className="text-sm text-muted-foreground">Selected Users</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-2xl font-bold">{stats.selectedTemplates}</p>
                <p className="text-sm text-muted-foreground">Templates</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Layers className="h-5 w-5 text-purple-500" />
              <div>
                <p className="text-2xl font-bold">{stats.totalAssignments}</p>
                <p className="text-sm text-muted-foreground">Total Assignments</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-orange-500" />
              <div>
                <p className="text-2xl font-bold">
                  {stats.estimatedDuration < 60 ? `${stats.estimatedDuration}m` :
                   stats.estimatedDuration < 1440 ? `${Math.round(stats.estimatedDuration / 60)}h` :
                   `${Math.round(stats.estimatedDuration / 1440)}d`}
                </p>
                <p className="text-sm text-muted-foreground">Max Duration</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Interface */}
      <Card>
        <CardContent className="p-0">
          <Tabs value={activeTab} onValueChange={(value: any) => setActiveTab(value)}>
            <div className="px-6 pt-6">
              <TabsList className="grid w-full grid-cols-4">
                <TabsTrigger value="users" className="flex items-center gap-2">
                  <User className="h-4 w-4" />
                  Select Users
                </TabsTrigger>
                <TabsTrigger value="templates" className="flex items-center gap-2">
                  <Shield className="h-4 w-4" />
                  Choose Templates
                </TabsTrigger>
                <TabsTrigger value="configure" className="flex items-center gap-2">
                  <Settings className="h-4 w-4" />
                  Configure
                </TabsTrigger>
                <TabsTrigger value="review" className="flex items-center gap-2">
                  <Eye className="h-4 w-4" />
                  Review & Execute
                </TabsTrigger>
              </TabsList>
            </div>

            <TabsContent value="users" className="p-6 space-y-4">
              <UserSelectionTab
                users={filteredUsers}
                selectedUsers={selectedUsers}
                searchTerm={userSearchTerm}
                roleFilter={userRoleFilter}
                statusFilter={userStatusFilter}
                onSearchChange={setUserSearchTerm}
                onRoleFilterChange={setUserRoleFilter}
                onStatusFilterChange={setUserStatusFilter}
                onUserSelect={handleUserSelect}
                onSelectAll={handleSelectAllUsers}
              />
              
              <div className="flex justify-end">
                <Button 
                  onClick={() => setActiveTab('templates')}
                  disabled={selectedUsers.length === 0}
                >
                  Next: Choose Templates
                </Button>
              </div>
            </TabsContent>

            <TabsContent value="templates" className="p-6 space-y-4">
              <TemplateSelectionTab
                templates={filteredTemplates}
                selectedTemplates={selectedTemplates}
                searchTerm={templateSearchTerm}
                categoryFilter={templateCategoryFilter}
                onSearchChange={setTemplateSearchTerm}
                onCategoryFilterChange={setTemplateCategoryFilter}
                onTemplateSelect={handleTemplateSelect}
                onSelectAll={handleSelectAllTemplates}
              />
              
              <div className="flex justify-between">
                <Button variant="outline" onClick={() => setActiveTab('users')}>
                  Back: Select Users
                </Button>
                <Button 
                  onClick={generateAssignmentTargets}
                  disabled={selectedTemplates.length === 0}
                >
                  Next: Configure Assignment
                </Button>
              </div>
            </TabsContent>

            <TabsContent value="configure" className="p-6 space-y-4">
              <ConfigurationTab
                jobName={jobName}
                globalReason={globalReason}
                defaultPriority={defaultPriority}
                scheduleType={scheduleType}
                scheduledTime={scheduledTime}
                onJobNameChange={setJobName}
                onGlobalReasonChange={setGlobalReason}
                onDefaultPriorityChange={setDefaultPriority}
                onScheduleTypeChange={setScheduleType}
                onScheduledTimeChange={setScheduledTime}
                assignmentTargets={assignmentTargets}
                onTargetsChange={setAssignmentTargets}
              />
              
              <div className="flex justify-between">
                <Button variant="outline" onClick={() => setActiveTab('templates')}>
                  Back: Choose Templates
                </Button>
                <Button onClick={() => setActiveTab('review')}>
                  Next: Review & Execute
                </Button>
              </div>
            </TabsContent>

            <TabsContent value="review" className="p-6 space-y-4">
              <ReviewAndExecuteTab
                assignmentTargets={assignmentTargets}
                selectedTemplates={selectedTemplates}
                templates={templates}
                currentJob={currentJob}
                isProcessing={isProcessing}
                onStartAssignment={handleStartBulkAssignment}
                onShowPreview={() => setShowPreview(true)}
              />
              
              <div className="flex justify-between">
                <Button variant="outline" onClick={() => setActiveTab('configure')}>
                  Back: Configure
                </Button>
                <Button
                  onClick={handleStartBulkAssignment}
                  disabled={isProcessing || assignmentTargets.length === 0}
                  className="bg-green-600 hover:bg-green-700"
                >
                  <Play className="h-4 w-4 mr-2" />
                  {isProcessing ? 'Processing...' : 'Start Bulk Assignment'}
                </Button>
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* Preview Dialog */}
      <Dialog open={showPreview} onOpenChange={setShowPreview}>
        <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Assignment Preview</DialogTitle>
            <DialogDescription>
              Preview of all permission assignments that will be created
            </DialogDescription>
          </DialogHeader>
          
          <AssignmentPreview
            assignmentTargets={assignmentTargets}
            templates={templates.filter(t => selectedTemplates.includes(t.id))}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ============================================================================
// TAB COMPONENTS
// ============================================================================

interface UserSelectionTabProps {
  users: UserProfile[]
  selectedUsers: string[]
  searchTerm: string
  roleFilter: string
  statusFilter: string
  onSearchChange: (term: string) => void
  onRoleFilterChange: (role: string) => void
  onStatusFilterChange: (status: string) => void
  onUserSelect: (userId: string, selected: boolean) => void
  onSelectAll: () => void
}

function UserSelectionTab({
  users,
  selectedUsers,
  searchTerm,
  roleFilter,
  statusFilter,
  onSearchChange,
  onRoleFilterChange,
  onStatusFilterChange,
  onUserSelect,
  onSelectAll
}: UserSelectionTabProps) {
  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="md:col-span-2">
          <Label>Search Users</Label>
          <div className="relative">
            <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search by name, email, role..."
              value={searchTerm}
              onChange={(e) => onSearchChange(e.target.value)}
              className="pl-9"
            />
          </div>
        </div>
        
        <div>
          <Label>Role</Label>
          <Select value={roleFilter} onValueChange={onRoleFilterChange}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Roles</SelectItem>
              <SelectItem value="Analytics Manager">Analytics Manager</SelectItem>
              <SelectItem value="Data Analyst">Data Analyst</SelectItem>
              <SelectItem value="System Administrator">System Administrator</SelectItem>
              <SelectItem value="Product Manager">Product Manager</SelectItem>
              <SelectItem value="Sales Manager">Sales Manager</SelectItem>
            </SelectContent>
          </Select>
        </div>
        
        <div>
          <Label>Status</Label>
          <Select value={statusFilter} onValueChange={onStatusFilterChange}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Users</SelectItem>
              <SelectItem value="active">Active Only</SelectItem>
              <SelectItem value="inactive">Inactive Only</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Select All */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <Checkbox
            id="select-all-users"
            checked={selectedUsers.length === users.length && users.length > 0}
            onCheckedChange={onSelectAll}
          />
          <Label htmlFor="select-all-users">
            Select all users ({users.length})
          </Label>
        </div>
        <Badge variant="secondary">
          {selectedUsers.length} selected
        </Badge>
      </div>

      {/* User List */}
      <ScrollArea className="h-[400px]">
        <div className="space-y-2">
          {users.map((user) => (
            <Card key={user.id} className={`p-4 ${
              selectedUsers.includes(user.id) ? 'ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/20' : ''
            }`}>
              <div className="flex items-center space-x-3">
                <Checkbox
                  id={`user-${user.id}`}
                  checked={selectedUsers.includes(user.id)}
                  onCheckedChange={(checked) => onUserSelect(user.id, !!checked)}
                />
                
                <div className="flex-1">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <h4 className="font-medium">{user.name}</h4>
                        <Badge variant={user.isActive ? 'default' : 'secondary'}>
                          {user.isActive ? 'Active' : 'Inactive'}
                        </Badge>
                      </div>
                      <p className="text-sm text-muted-foreground">{user.email}</p>
                      <p className="text-sm text-muted-foreground">{user.role} • {user.department}</p>
                    </div>
                    
                    <div className="text-right text-xs text-muted-foreground">
                      <div>Last active: {format(user.lastActive, 'MMM d, HH:mm')}</div>
                      <div>{user.currentPermissions.length} current permissions</div>
                    </div>
                  </div>
                </div>
              </div>
            </Card>
          ))}
        </div>
      </ScrollArea>
    </div>
  )
}

interface TemplateSelectionTabProps {
  templates: PermissionTemplate[]
  selectedTemplates: string[]
  searchTerm: string
  categoryFilter: string
  onSearchChange: (term: string) => void
  onCategoryFilterChange: (category: string) => void
  onTemplateSelect: (templateId: string, selected: boolean) => void
  onSelectAll: () => void
}

function TemplateSelectionTab({
  templates,
  selectedTemplates,
  searchTerm,
  categoryFilter,
  onSearchChange,
  onCategoryFilterChange,
  onTemplateSelect,
  onSelectAll
}: TemplateSelectionTabProps) {
  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="md:col-span-2">
          <Label>Search Templates</Label>
          <div className="relative">
            <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search by name, description, permission..."
              value={searchTerm}
              onChange={(e) => onSearchChange(e.target.value)}
              className="pl-9"
            />
          </div>
        </div>
        
        <div>
          <Label>Category</Label>
          <Select value={categoryFilter} onValueChange={onCategoryFilterChange}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Categories</SelectItem>
              <SelectItem value="analytics">Analytics</SelectItem>
              <SelectItem value="admin">Administration</SelectItem>
              <SelectItem value="premium">Premium</SelectItem>
              <SelectItem value="emergency">Emergency</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Select All */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <Checkbox
            id="select-all-templates"
            checked={selectedTemplates.length === templates.length && templates.length > 0}
            onCheckedChange={onSelectAll}
          />
          <Label htmlFor="select-all-templates">
            Select all templates ({templates.length})
          </Label>
        </div>
        <Badge variant="secondary">
          {selectedTemplates.length} selected
        </Badge>
      </div>

      {/* Template Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {templates.map((template) => (
          <Card key={template.id} className={`p-4 cursor-pointer transition-all ${
            selectedTemplates.includes(template.id) 
              ? 'ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/20' 
              : 'hover:shadow-md'
          }`}>
            <div className="flex items-start space-x-3">
              <Checkbox
                id={`template-${template.id}`}
                checked={selectedTemplates.includes(template.id)}
                onCheckedChange={(checked) => onTemplateSelect(template.id, !!checked)}
              />
              
              <div className="flex-1">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium">{template.name}</h4>
                  <Badge variant="outline">
                    {template.category}
                  </Badge>
                </div>
                
                <p className="text-sm text-muted-foreground mb-2">
                  {template.description}
                </p>
                
                <div className="space-y-1 text-xs">
                  <div>
                    <strong>Permission:</strong> {template.basePermission}
                  </div>
                  <div className="flex items-center justify-between">
                    <span>
                      <strong>Duration:</strong> {
                        template.defaultDuration < 60 ? `${template.defaultDuration}m` :
                        template.defaultDuration < 1440 ? `${Math.round(template.defaultDuration / 60)}h` :
                        `${Math.round(template.defaultDuration / 1440)}d`
                      }
                    </span>
                    <Badge variant="secondary" className="text-xs">
                      {template.platform}
                    </Badge>
                  </div>
                </div>
                
                {template.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1 mt-2">
                    {template.tags.slice(0, 3).map((tag, index) => (
                      <Badge key={index} variant="outline" className="text-xs">
                        {tag}
                      </Badge>
                    ))}
                    {template.tags.length > 3 && (
                      <Badge variant="outline" className="text-xs">
                        +{template.tags.length - 3}
                      </Badge>
                    )}
                  </div>
                )}
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  )
}

interface ConfigurationTabProps {
  jobName: string
  globalReason: string
  defaultPriority: 'low' | 'normal' | 'high' | 'urgent'
  scheduleType: 'immediate' | 'scheduled'
  scheduledTime: string
  onJobNameChange: (name: string) => void
  onGlobalReasonChange: (reason: string) => void
  onDefaultPriorityChange: (priority: 'low' | 'normal' | 'high' | 'urgent') => void
  onScheduleTypeChange: (type: 'immediate' | 'scheduled') => void
  onScheduledTimeChange: (time: string) => void
  assignmentTargets: BulkAssignmentTarget[]
  onTargetsChange: (targets: BulkAssignmentTarget[]) => void
}

function ConfigurationTab({
  jobName,
  globalReason,
  defaultPriority,
  scheduleType,
  scheduledTime,
  onJobNameChange,
  onGlobalReasonChange,
  onDefaultPriorityChange,
  onScheduleTypeChange,
  onScheduledTimeChange,
  assignmentTargets,
  onTargetsChange
}: ConfigurationTabProps) {
  return (
    <div className="space-y-6">
      {/* Job Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>Job Configuration</CardTitle>
          <CardDescription>Configure the bulk assignment job settings</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="job-name">Job Name *</Label>
              <Input
                id="job-name"
                value={jobName}
                onChange={(e) => onJobNameChange(e.target.value)}
                placeholder="e.g., Weekly Analytics Access Grant"
              />
            </div>
            
            <div>
              <Label htmlFor="priority">Default Priority</Label>
              <Select value={defaultPriority} onValueChange={onDefaultPriorityChange}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="low">Low</SelectItem>
                  <SelectItem value="normal">Normal</SelectItem>
                  <SelectItem value="high">High</SelectItem>
                  <SelectItem value="urgent">Urgent</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          
          <div>
            <Label htmlFor="global-reason">Global Reason</Label>
            <Textarea
              id="global-reason"
              value={globalReason}
              onChange={(e) => onGlobalReasonChange(e.target.value)}
              placeholder="Reason for bulk permission assignment..."
              rows={2}
            />
          </div>
          
          <div className="space-y-4">
            <Label>Scheduling</Label>
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  id="immediate"
                  name="schedule"
                  checked={scheduleType === 'immediate'}
                  onChange={() => onScheduleTypeChange('immediate')}
                />
                <Label htmlFor="immediate">Execute Immediately</Label>
              </div>
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  id="scheduled"
                  name="schedule"
                  checked={scheduleType === 'scheduled'}
                  onChange={() => onScheduleTypeChange('scheduled')}
                />
                <Label htmlFor="scheduled">Schedule for Later</Label>
              </div>
            </div>
            
            {scheduleType === 'scheduled' && (
              <div>
                <Label htmlFor="scheduled-time">Scheduled Time</Label>
                <Input
                  id="scheduled-time"
                  type="datetime-local"
                  value={scheduledTime}
                  onChange={(e) => onScheduledTimeChange(e.target.value)}
                  min={new Date().toISOString().slice(0, 16)}
                />
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Assignment Summary */}
      <Card>
        <CardHeader>
          <CardTitle>Assignment Summary</CardTitle>
          <CardDescription>
            {assignmentTargets.length} users will receive permissions from selected templates
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-muted-foreground">
            Each user will receive all selected permission templates with their default durations.
            You can modify individual assignments in the review step.
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

interface ReviewAndExecuteTabProps {
  assignmentTargets: BulkAssignmentTarget[]
  selectedTemplates: string[]
  templates: PermissionTemplate[]
  currentJob: BulkAssignmentJob | null
  isProcessing: boolean
  onStartAssignment: () => void
  onShowPreview: () => void
}

function ReviewAndExecuteTab({
  assignmentTargets,
  selectedTemplates,
  templates,
  currentJob,
  isProcessing,
  onStartAssignment,
  onShowPreview
}: ReviewAndExecuteTabProps) {
  const selectedTemplateObjects = templates.filter(t => selectedTemplates.includes(t.id))
  
  return (
    <div className="space-y-6">
      {/* Summary */}
      <Card>
        <CardHeader>
          <CardTitle>Assignment Summary</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
            <div>
              <strong>Users:</strong> {assignmentTargets.length}
            </div>
            <div>
              <strong>Templates:</strong> {selectedTemplateObjects.length}
            </div>
            <div>
              <strong>Total Assignments:</strong> {assignmentTargets.length * selectedTemplateObjects.length}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Selected Templates */}
      <Card>
        <CardHeader>
          <CardTitle>Selected Templates</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {selectedTemplateObjects.map((template) => (
              <div key={template.id} className="flex items-center justify-between p-2 border rounded">
                <div>
                  <div className="font-medium">{template.name}</div>
                  <div className="text-sm text-muted-foreground">{template.basePermission}</div>
                </div>
                <Badge>
                  {template.defaultDuration < 60 ? `${template.defaultDuration}m` :
                   template.defaultDuration < 1440 ? `${Math.round(template.defaultDuration / 60)}h` :
                   `${Math.round(template.defaultDuration / 1440)}d`}
                </Badge>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Job Progress */}
      {currentJob && (
        <Card>
          <CardHeader>
            <CardTitle>Job Progress</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span>Progress</span>
              <span>{Math.round(currentJob.progress)}%</span>
            </div>
            <Progress value={currentJob.progress} />
            
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">{currentJob.successCount}</div>
                <div className="text-muted-foreground">Success</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600">{currentJob.failureCount}</div>
                <div className="text-muted-foreground">Failed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-gray-600">{currentJob.skipCount}</div>
                <div className="text-muted-foreground">Skipped</div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Preview Button */}
      <div className="flex justify-center">
        <Button variant="outline" onClick={onShowPreview}>
          <Eye className="h-4 w-4 mr-2" />
          Preview All Assignments
        </Button>
      </div>
    </div>
  )
}

interface AssignmentPreviewProps {
  assignmentTargets: BulkAssignmentTarget[]
  templates: PermissionTemplate[]
}

function AssignmentPreview({ assignmentTargets, templates }: AssignmentPreviewProps) {
  return (
    <ScrollArea className="h-[60vh]">
      <div className="space-y-4">
        {assignmentTargets.map((target) => (
          <Card key={target.userId}>
            <CardHeader>
              <CardTitle className="text-lg">{target.user.name}</CardTitle>
              <CardDescription>{target.user.email}</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {target.selectedTemplates.map((templateId) => {
                  const template = templates.find(t => t.id === templateId)
                  if (!template) return null
                  
                  const expiryTime = addMinutes(new Date(), template.defaultDuration)
                  const embeddedPermission = `${template.basePermission}:${Math.floor(expiryTime.getTime() / 1000)}`
                  
                  return (
                    <div key={templateId} className="p-3 border rounded">
                      <div className="font-medium">{template.name}</div>
                      <div className="text-sm text-muted-foreground font-mono">
                        {embeddedPermission}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Expires: {format(expiryTime, 'PPp')}
                      </div>
                    </div>
                  )
                })}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </ScrollArea>
  )
}