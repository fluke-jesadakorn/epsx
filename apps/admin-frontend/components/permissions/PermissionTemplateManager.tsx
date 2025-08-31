'use client'

import { useState, useMemo, useEffect } from 'react'
import { 
  Template, 
  Plus, 
  Edit, 
  Copy, 
  Trash2, 
  Download, 
  Upload,
  Search,
  Filter,
  Tag,
  Clock,
  Shield,
  Users,
  TrendingUp,
  Star,
  StarOff,
  Eye,
  Settings,
  Save,
  X,
  Check,
  AlertTriangle,
  Layers,
  Zap,
  Activity,
  FileText,
  Calendar
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useToast } from '@/components/ui/use-toast'
import { format } from 'date-fns'

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface PermissionTemplate {
  id: string
  name: string
  description: string
  basePermission: string
  platform: 'epsx' | 'admin' | 'epsx-pay' | 'epsx-token'
  resource: string
  action: string
  defaultDuration: number // minutes
  category: 'analytics' | 'admin' | 'premium' | 'emergency' | 'custom'
  tags: string[]
  isActive: boolean
  isFavorite: boolean
  createdBy: string
  createdAt: Date
  updatedAt: Date
  usageCount: number
  lastUsed?: Date
  conditions?: PermissionCondition[]
  variables?: PermissionVariable[]
}

interface PermissionCondition {
  id: string
  type: 'time_range' | 'user_role' | 'platform_access' | 'usage_limit'
  operator: 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'contains'
  value: string
  isActive: boolean
}

interface PermissionVariable {
  id: string
  name: string
  type: 'number' | 'string' | 'boolean' | 'duration'
  defaultValue: any
  description: string
  isRequired: boolean
}

interface TemplateStats {
  totalTemplates: number
  activeTemplates: number
  favoriteTemplates: number
  totalUsage: number
  mostUsedTemplate: string
  averageUsage: number
}

interface PermissionTemplateManagerProps {
  onTemplateSelect?: (template: PermissionTemplate) => void
  className?: string
}

// ============================================================================
// MOCK DATA
// ============================================================================

const MOCK_TEMPLATES: PermissionTemplate[] = [
  {
    id: 'template-1',
    name: 'Analytics Viewer',
    description: 'Temporary access to analytics dashboard with view permissions',
    basePermission: 'epsx:analytics:view',
    platform: 'epsx',
    resource: 'analytics',
    action: 'view',
    defaultDuration: 240, // 4 hours
    category: 'analytics',
    tags: ['analytics', 'dashboard', 'temporary', 'read-only'],
    isActive: true,
    isFavorite: true,
    createdBy: 'admin',
    createdAt: new Date('2024-01-15T10:00:00Z'),
    updatedAt: new Date('2024-01-20T15:30:00Z'),
    usageCount: 25,
    lastUsed: new Date('2024-01-25T09:15:00Z')
  },
  {
    id: 'template-2',
    name: 'Premium Rankings Access',
    description: 'Access to top 100 rankings with 24-hour duration',
    basePermission: 'epsx:rankings:view:100',
    platform: 'epsx',
    resource: 'rankings',
    action: 'view',
    defaultDuration: 1440, // 24 hours
    category: 'premium',
    tags: ['rankings', 'premium', 'data', 'extended'],
    isActive: true,
    isFavorite: false,
    createdBy: 'admin',
    createdAt: new Date('2024-01-10T12:00:00Z'),
    updatedAt: new Date('2024-01-15T14:20:00Z'),
    usageCount: 18,
    lastUsed: new Date('2024-01-24T16:45:00Z')
  },
  {
    id: 'template-3',
    name: 'Emergency Admin Access',
    description: 'Emergency user management access with extended permissions',
    basePermission: 'admin:users:manage',
    platform: 'admin',
    resource: 'users',
    action: 'manage',
    defaultDuration: 480, // 8 hours
    category: 'emergency',
    tags: ['admin', 'emergency', 'users', 'critical'],
    isActive: true,
    isFavorite: true,
    createdBy: 'super-admin',
    createdAt: new Date('2024-01-05T08:00:00Z'),
    updatedAt: new Date('2024-01-18T11:10:00Z'),
    usageCount: 12,
    lastUsed: new Date('2024-01-23T14:30:00Z')
  },
  {
    id: 'template-4',
    name: 'Data Export Specialist',
    description: 'Temporary data export capabilities for analytics data',
    basePermission: 'epsx:analytics:export',
    platform: 'epsx',
    resource: 'analytics',
    action: 'export',
    defaultDuration: 60, // 1 hour
    category: 'analytics',
    tags: ['export', 'data', 'analytics', 'short-term'],
    isActive: false,
    isFavorite: false,
    createdBy: 'data-admin',
    createdAt: new Date('2024-01-12T14:00:00Z'),
    updatedAt: new Date('2024-01-19T10:25:00Z'),
    usageCount: 8,
    lastUsed: new Date('2024-01-22T13:20:00Z')
  }
]

const PERMISSION_CATEGORIES = [
  { value: 'analytics', label: 'Analytics', color: 'bg-blue-100 text-blue-800' },
  { value: 'admin', label: 'Administration', color: 'bg-red-100 text-red-800' },
  { value: 'premium', label: 'Premium Features', color: 'bg-purple-100 text-purple-800' },
  { value: 'emergency', label: 'Emergency Access', color: 'bg-orange-100 text-orange-800' },
  { value: 'custom', label: 'Custom', color: 'bg-gray-100 text-gray-800' }
] as const

const DURATION_PRESETS = [
  { label: '15 minutes', minutes: 15 },
  { label: '1 hour', minutes: 60 },
  { label: '4 hours', minutes: 240 },
  { label: '8 hours', minutes: 480 },
  { label: '1 day', minutes: 1440 },
  { label: '3 days', minutes: 4320 },
  { label: '1 week', minutes: 10080 }
] as const

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function PermissionTemplateManager({ onTemplateSelect, className = '' }: PermissionTemplateManagerProps) {
  const { toast } = useToast()
  
  // State management
  const [templates, setTemplates] = useState<PermissionTemplate[]>(MOCK_TEMPLATES)
  const [searchTerm, setSearchTerm] = useState('')
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'inactive'>('all')
  const [favoriteFilter, setFavoriteFilter] = useState(false)
  const [sortBy, setSortBy] = useState<'name' | 'created' | 'usage' | 'updated'>('name')
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc')
  
  // Form state
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [editingTemplate, setEditingTemplate] = useState<PermissionTemplate | null>(null)
  const [formData, setFormData] = useState<Partial<PermissionTemplate>>({
    name: '',
    description: '',
    basePermission: '',
    platform: 'epsx',
    resource: '',
    action: '',
    defaultDuration: 240,
    category: 'custom',
    tags: [],
    isActive: true
  })
  const [newTag, setNewTag] = useState('')
  
  // Computed values
  const filteredTemplates = useMemo(() => {
    let filtered = templates.filter(template => {
      // Search filter
      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase()
        const matchesSearch = (
          template.name.toLowerCase().includes(searchLower) ||
          template.description.toLowerCase().includes(searchLower) ||
          template.basePermission.toLowerCase().includes(searchLower) ||
          template.tags.some(tag => tag.toLowerCase().includes(searchLower))
        )
        if (!matchesSearch) return false
      }
      
      // Category filter
      if (categoryFilter !== 'all' && template.category !== categoryFilter) {
        return false
      }
      
      // Status filter
      if (statusFilter !== 'all') {
        if (statusFilter === 'active' && !template.isActive) return false
        if (statusFilter === 'inactive' && template.isActive) return false
      }
      
      // Favorite filter
      if (favoriteFilter && !template.isFavorite) {
        return false
      }
      
      return true
    })
    
    // Sort templates
    filtered.sort((a, b) => {
      let aVal: any, bVal: any
      
      switch (sortBy) {
        case 'name':
          aVal = a.name.toLowerCase()
          bVal = b.name.toLowerCase()
          break
        case 'created':
          aVal = a.createdAt.getTime()
          bVal = b.createdAt.getTime()
          break
        case 'usage':
          aVal = a.usageCount
          bVal = b.usageCount
          break
        case 'updated':
          aVal = a.updatedAt.getTime()
          bVal = b.updatedAt.getTime()
          break
        default:
          return 0
      }
      
      const comparison = aVal < bVal ? -1 : aVal > bVal ? 1 : 0
      return sortOrder === 'asc' ? comparison : -comparison
    })
    
    return filtered
  }, [templates, searchTerm, categoryFilter, statusFilter, favoriteFilter, sortBy, sortOrder])

  const templateStats = useMemo((): TemplateStats => {
    const activeTemplates = templates.filter(t => t.isActive)
    const favoriteTemplates = templates.filter(t => t.isFavorite)
    const totalUsage = templates.reduce((sum, t) => sum + t.usageCount, 0)
    const mostUsedTemplate = templates.reduce((prev, current) => 
      prev.usageCount > current.usageCount ? prev : current
    ).name
    
    return {
      totalTemplates: templates.length,
      activeTemplates: activeTemplates.length,
      favoriteTemplates: favoriteTemplates.length,
      totalUsage,
      mostUsedTemplate,
      averageUsage: templates.length > 0 ? totalUsage / templates.length : 0
    }
  }, [templates])

  // Event handlers
  const handleCreateTemplate = () => {
    if (!formData.name || !formData.basePermission) {
      toast({
        title: 'Validation Error',
        description: 'Name and base permission are required',
        variant: 'destructive'
      })
      return
    }
    
    const newTemplate: PermissionTemplate = {
      id: `template-${Date.now()}`,
      name: formData.name!,
      description: formData.description || '',
      basePermission: formData.basePermission!,
      platform: formData.platform || 'epsx',
      resource: formData.resource || '',
      action: formData.action || '',
      defaultDuration: formData.defaultDuration || 240,
      category: formData.category || 'custom',
      tags: formData.tags || [],
      isActive: formData.isActive ?? true,
      isFavorite: false,
      createdBy: 'current-admin',
      createdAt: new Date(),
      updatedAt: new Date(),
      usageCount: 0
    }
    
    setTemplates(prev => [...prev, newTemplate])
    setShowCreateDialog(false)
    resetForm()
    
    toast({
      title: 'Success',
      description: `Template "${newTemplate.name}" created successfully`
    })
  }

  const handleUpdateTemplate = () => {
    if (!editingTemplate || !formData.name || !formData.basePermission) {
      toast({
        title: 'Validation Error',
        description: 'Name and base permission are required',
        variant: 'destructive'
      })
      return
    }
    
    setTemplates(prev => prev.map(template => 
      template.id === editingTemplate.id 
        ? { ...template, ...formData, updatedAt: new Date() }
        : template
    ))
    
    setEditingTemplate(null)
    resetForm()
    
    toast({
      title: 'Success',
      description: `Template "${formData.name}" updated successfully`
    })
  }

  const handleToggleFavorite = (templateId: string) => {
    setTemplates(prev => prev.map(template =>
      template.id === templateId
        ? { ...template, isFavorite: !template.isFavorite }
        : template
    ))
  }

  const handleToggleActive = (templateId: string) => {
    setTemplates(prev => prev.map(template =>
      template.id === templateId
        ? { ...template, isActive: !template.isActive }
        : template
    ))
  }

  const handleDeleteTemplate = (templateId: string) => {
    const template = templates.find(t => t.id === templateId)
    if (!template) return
    
    setTemplates(prev => prev.filter(t => t.id !== templateId))
    
    toast({
      title: 'Success',
      description: `Template "${template.name}" deleted successfully`
    })
  }

  const handleCloneTemplate = (templateId: string) => {
    const template = templates.find(t => t.id === templateId)
    if (!template) return
    
    const clonedTemplate: PermissionTemplate = {
      ...template,
      id: `template-${Date.now()}`,
      name: `${template.name} (Copy)`,
      isFavorite: false,
      createdAt: new Date(),
      updatedAt: new Date(),
      usageCount: 0,
      lastUsed: undefined
    }
    
    setTemplates(prev => [...prev, clonedTemplate])
    
    toast({
      title: 'Success',
      description: `Template "${clonedTemplate.name}" created as copy`
    })
  }

  const handleAddTag = () => {
    if (!newTag.trim()) return
    
    const currentTags = formData.tags || []
    if (currentTags.includes(newTag.trim())) {
      toast({
        title: 'Warning',
        description: 'Tag already exists',
        variant: 'destructive'
      })
      return
    }
    
    setFormData(prev => ({
      ...prev,
      tags: [...currentTags, newTag.trim()]
    }))
    setNewTag('')
  }

  const handleRemoveTag = (tagToRemove: string) => {
    setFormData(prev => ({
      ...prev,
      tags: (prev.tags || []).filter(tag => tag !== tagToRemove)
    }))
  }

  const resetForm = () => {
    setFormData({
      name: '',
      description: '',
      basePermission: '',
      platform: 'epsx',
      resource: '',
      action: '',
      defaultDuration: 240,
      category: 'custom',
      tags: [],
      isActive: true
    })
    setNewTag('')
  }

  const startEdit = (template: PermissionTemplate) => {
    setEditingTemplate(template)
    setFormData({ ...template })
    setShowCreateDialog(true)
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-xl font-semibold">Permission Templates</h3>
          <p className="text-muted-foreground">
            Create and manage reusable permission templates with embedded timestamps
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
          <Button variant="outline" size="sm">
            <Upload className="h-4 w-4 mr-2" />
            Import
          </Button>
          <Button 
            size="sm"
            onClick={() => {
              resetForm()
              setEditingTemplate(null)
              setShowCreateDialog(true)
            }}
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Template
          </Button>
        </div>
      </div>

      {/* Template Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Template className="h-5 w-5 text-blue-500" />
              <div>
                <p className="text-2xl font-bold">{templateStats.totalTemplates}</p>
                <p className="text-sm text-muted-foreground">Total Templates</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Check className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-2xl font-bold">{templateStats.activeTemplates}</p>
                <p className="text-sm text-muted-foreground">Active</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Star className="h-5 w-5 text-yellow-500" />
              <div>
                <p className="text-2xl font-bold">{templateStats.favoriteTemplates}</p>
                <p className="text-sm text-muted-foreground">Favorites</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5 text-purple-500" />
              <div>
                <p className="text-2xl font-bold">{templateStats.totalUsage}</p>
                <p className="text-sm text-muted-foreground">Total Usage</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters and Search */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Filter className="h-5 w-5" />
            Filters & Search
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-6 gap-4">
            <div className="md:col-span-2">
              <Label>Search Templates</Label>
              <div className="relative">
                <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search by name, description, permission..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-9"
                />
              </div>
            </div>

            <div>
              <Label>Category</Label>
              <Select value={categoryFilter} onValueChange={setCategoryFilter}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Categories</SelectItem>
                  {PERMISSION_CATEGORIES.map(cat => (
                    <SelectItem key={cat.value} value={cat.value}>
                      {cat.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Status</Label>
              <Select value={statusFilter} onValueChange={(value: any) => setStatusFilter(value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All</SelectItem>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="inactive">Inactive</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Sort By</Label>
              <Select value={sortBy} onValueChange={(value: any) => setSortBy(value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="name">Name</SelectItem>
                  <SelectItem value="created">Created</SelectItem>
                  <SelectItem value="usage">Usage</SelectItem>
                  <SelectItem value="updated">Updated</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-end gap-2">
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="favorites"
                  checked={favoriteFilter}
                  onCheckedChange={setFavoriteFilter}
                />
                <Label htmlFor="favorites" className="text-sm">
                  Favorites only
                </Label>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')}
              >
                {sortOrder === 'asc' ? '↑' : '↓'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Templates Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredTemplates.length === 0 ? (
          <div className="col-span-full text-center py-12">
            <Template className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No templates found</h3>
            <p className="text-muted-foreground mb-4">
              {searchTerm || categoryFilter !== 'all' || statusFilter !== 'all' || favoriteFilter
                ? 'Try adjusting your filters'
                : 'Create your first permission template to get started'
              }
            </p>
            <Button onClick={() => setShowCreateDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Create Template
            </Button>
          </div>
        ) : (
          filteredTemplates.map((template) => (
            <PermissionTemplateCard
              key={template.id}
              template={template}
              onEdit={startEdit}
              onClone={handleCloneTemplate}
              onDelete={handleDeleteTemplate}
              onToggleFavorite={handleToggleFavorite}
              onToggleActive={handleToggleActive}
              onSelect={() => onTemplateSelect?.(template)}
            />
          ))
        )}
      </div>

      {/* Create/Edit Template Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editingTemplate ? 'Edit Template' : 'Create New Template'}
            </DialogTitle>
            <DialogDescription>
              {editingTemplate
                ? 'Update the permission template configuration'
                : 'Create a reusable permission template with embedded timestamp support'
              }
            </DialogDescription>
          </DialogHeader>

          <TemplateForm
            formData={formData}
            onFormDataChange={setFormData}
            newTag={newTag}
            onNewTagChange={setNewTag}
            onAddTag={handleAddTag}
            onRemoveTag={handleRemoveTag}
            onSave={editingTemplate ? handleUpdateTemplate : handleCreateTemplate}
            onCancel={() => {
              setShowCreateDialog(false)
              setEditingTemplate(null)
              resetForm()
            }}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ============================================================================
// TEMPLATE CARD COMPONENT
// ============================================================================

interface PermissionTemplateCardProps {
  template: PermissionTemplate
  onEdit: (template: PermissionTemplate) => void
  onClone: (id: string) => void
  onDelete: (id: string) => void
  onToggleFavorite: (id: string) => void
  onToggleActive: (id: string) => void
  onSelect?: () => void
}

function PermissionTemplateCard({
  template,
  onEdit,
  onClone,
  onDelete,
  onToggleFavorite,
  onToggleActive,
  onSelect
}: PermissionTemplateCardProps) {
  const categoryConfig = PERMISSION_CATEGORIES.find(c => c.value === template.category)
  
  const formatDuration = (minutes: number) => {
    if (minutes < 60) return `${minutes}m`
    if (minutes < 1440) return `${Math.round(minutes / 60)}h`
    return `${Math.round(minutes / 1440)}d`
  }

  return (
    <Card className={`transition-all hover:shadow-md ${
      !template.isActive ? 'opacity-75' : ''
    }`}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <h4 className="font-medium truncate">{template.name}</h4>
              {template.isFavorite && (
                <Star className="h-4 w-4 text-yellow-500 fill-current" />
              )}
            </div>
            <p className="text-sm text-muted-foreground line-clamp-2">
              {template.description}
            </p>
          </div>
          
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm">
                <Settings className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {onSelect && (
                <>
                  <DropdownMenuItem onClick={onSelect}>
                    <Zap className="h-4 w-4 mr-2" />
                    Use Template
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                </>
              )}
              <DropdownMenuItem onClick={() => onEdit(template)}>
                <Edit className="h-4 w-4 mr-2" />
                Edit
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onClone(template.id)}>
                <Copy className="h-4 w-4 mr-2" />
                Clone
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onToggleFavorite(template.id)}>
                {template.isFavorite ? (
                  <StarOff className="h-4 w-4 mr-2" />
                ) : (
                  <Star className="h-4 w-4 mr-2" />
                )}
                {template.isFavorite ? 'Remove Favorite' : 'Add Favorite'}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => onToggleActive(template.id)}>
                <Activity className="h-4 w-4 mr-2" />
                {template.isActive ? 'Deactivate' : 'Activate'}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => onDelete(template.id)}
                className="text-red-600"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </CardHeader>

      <CardContent className="pt-0 space-y-3">
        {/* Permission */}
        <div>
          <p className="text-xs text-muted-foreground">Permission</p>
          <p className="text-sm font-mono">{template.basePermission}</p>
        </div>

        {/* Category and Duration */}
        <div className="flex items-center justify-between">
          <Badge className={categoryConfig?.color || 'bg-gray-100 text-gray-800'}>
            {categoryConfig?.label || template.category}
          </Badge>
          <div className="flex items-center gap-1 text-xs text-muted-foreground">
            <Clock className="h-3 w-3" />
            {formatDuration(template.defaultDuration)}
          </div>
        </div>

        {/* Tags */}
        {template.tags.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {template.tags.slice(0, 3).map((tag, index) => (
              <Badge key={index} variant="outline" className="text-xs">
                {tag}
              </Badge>
            ))}
            {template.tags.length > 3 && (
              <Badge variant="outline" className="text-xs">
                +{template.tags.length - 3} more
              </Badge>
            )}
          </div>
        )}

        {/* Usage Stats */}
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <Users className="h-3 w-3" />
            {template.usageCount} uses
          </div>
          {template.lastUsed && (
            <div className="flex items-center gap-1">
              <Calendar className="h-3 w-3" />
              {format(template.lastUsed, 'MMM d')}
            </div>
          )}
        </div>

        {/* Status Indicator */}
        <div className="flex items-center gap-2 pt-2 border-t">
          <div className={`w-2 h-2 rounded-full ${
            template.isActive ? 'bg-green-500' : 'bg-gray-400'
          }`} />
          <span className="text-xs text-muted-foreground">
            {template.isActive ? 'Active' : 'Inactive'}
          </span>
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// TEMPLATE FORM COMPONENT
// ============================================================================

interface TemplateFormProps {
  formData: Partial<PermissionTemplate>
  onFormDataChange: (data: Partial<PermissionTemplate>) => void
  newTag: string
  onNewTagChange: (tag: string) => void
  onAddTag: () => void
  onRemoveTag: (tag: string) => void
  onSave: () => void
  onCancel: () => void
}

function TemplateForm({
  formData,
  onFormDataChange,
  newTag,
  onNewTagChange,
  onAddTag,
  onRemoveTag,
  onSave,
  onCancel
}: TemplateFormProps) {
  const updateFormData = (field: keyof PermissionTemplate, value: any) => {
    onFormDataChange({ ...formData, [field]: value })
  }

  return (
    <div className="space-y-6">
      <Tabs defaultValue="basic" className="w-full">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="basic">Basic Info</TabsTrigger>
          <TabsTrigger value="permission">Permission</TabsTrigger>
          <TabsTrigger value="advanced">Advanced</TabsTrigger>
        </TabsList>

        <TabsContent value="basic" className="space-y-4">
          {/* Name and Description */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="name">Template Name *</Label>
              <Input
                id="name"
                value={formData.name || ''}
                onChange={(e) => updateFormData('name', e.target.value)}
                placeholder="e.g., Analytics Viewer"
              />
            </div>
            <div>
              <Label htmlFor="category">Category</Label>
              <Select 
                value={formData.category || 'custom'} 
                onValueChange={(value) => updateFormData('category', value)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PERMISSION_CATEGORIES.map(cat => (
                    <SelectItem key={cat.value} value={cat.value}>
                      {cat.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div>
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={formData.description || ''}
              onChange={(e) => updateFormData('description', e.target.value)}
              placeholder="Describe what this template grants access to..."
              rows={3}
            />
          </div>

          {/* Default Duration */}
          <div>
            <Label>Default Duration: {Math.round((formData.defaultDuration || 0) / 60 * 100) / 100} hours</Label>
            <div className="space-y-2">
              <Slider
                value={[formData.defaultDuration || 240]}
                onValueChange={(value) => updateFormData('defaultDuration', value[0])}
                min={15}
                max={10080} // 1 week
                step={15}
              />
              <div className="grid grid-cols-4 gap-2">
                {DURATION_PRESETS.slice(0, 4).map((preset) => (
                  <Button
                    key={preset.label}
                    variant="outline"
                    size="sm"
                    onClick={() => updateFormData('defaultDuration', preset.minutes)}
                    className="text-xs"
                  >
                    {preset.label}
                  </Button>
                ))}
              </div>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="permission" className="space-y-4">
          {/* Platform and Permission */}
          <div className="grid grid-cols-3 gap-4">
            <div>
              <Label htmlFor="platform">Platform</Label>
              <Select 
                value={formData.platform || 'epsx'} 
                onValueChange={(value: any) => updateFormData('platform', value)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="epsx">EPSX</SelectItem>
                  <SelectItem value="admin">Admin</SelectItem>
                  <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                  <SelectItem value="epsx-token">EPSX Token</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label htmlFor="resource">Resource</Label>
              <Input
                id="resource"
                value={formData.resource || ''}
                onChange={(e) => updateFormData('resource', e.target.value)}
                placeholder="e.g., analytics, users"
              />
            </div>
            <div>
              <Label htmlFor="action">Action</Label>
              <Input
                id="action"
                value={formData.action || ''}
                onChange={(e) => updateFormData('action', e.target.value)}
                placeholder="e.g., view, manage"
              />
            </div>
          </div>

          <div>
            <Label htmlFor="basePermission">Base Permission *</Label>
            <Input
              id="basePermission"
              value={formData.basePermission || ''}
              onChange={(e) => updateFormData('basePermission', e.target.value)}
              placeholder="e.g., epsx:analytics:view"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Format: platform:resource:action (timestamp will be added automatically)
            </p>
          </div>

          {/* Permission Preview */}
          {formData.basePermission && (
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                <div className="space-y-2">
                  <div><strong>Permission Preview:</strong></div>
                  <code className="text-sm bg-gray-100 p-2 rounded block">
                    {formData.basePermission}:{Math.floor(Date.now() / 1000) + (formData.defaultDuration || 0) * 60}
                  </code>
                  <div className="text-xs text-muted-foreground">
                    Final permission will include timestamp when granted
                  </div>
                </div>
              </AlertDescription>
            </Alert>
          )}
        </TabsContent>

        <TabsContent value="advanced" className="space-y-4">
          {/* Tags */}
          <div>
            <Label>Tags</Label>
            <div className="space-y-2">
              <div className="flex gap-2">
                <Input
                  value={newTag}
                  onChange={(e) => onNewTagChange(e.target.value)}
                  placeholder="Add a tag..."
                  onKeyPress={(e) => e.key === 'Enter' && onAddTag()}
                />
                <Button type="button" onClick={onAddTag}>
                  <Tag className="h-4 w-4" />
                </Button>
              </div>
              
              {formData.tags && formData.tags.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {formData.tags.map((tag, index) => (
                    <Badge key={index} variant="outline" className="text-xs">
                      {tag}
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => onRemoveTag(tag)}
                        className="ml-1 h-auto p-0 text-xs"
                      >
                        <X className="h-3 w-3" />
                      </Button>
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Status */}
          <div className="flex items-center space-x-2">
            <Switch
              id="isActive"
              checked={formData.isActive ?? true}
              onCheckedChange={(checked) => updateFormData('isActive', checked)}
            />
            <Label htmlFor="isActive">Template is active</Label>
          </div>
        </TabsContent>
      </Tabs>

      {/* Action Buttons */}
      <div className="flex gap-2 pt-4">
        <Button onClick={onSave} className="flex-1">
          <Save className="h-4 w-4 mr-2" />
          Save Template
        </Button>
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
      </div>
    </div>
  )
}