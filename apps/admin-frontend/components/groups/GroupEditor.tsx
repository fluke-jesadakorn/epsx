/**
 * Group Editor Component
 * Interface for creating and editing permission groups
 * 
 * Features:
 * - Create new permission groups
 * - Edit existing groups (except system groups)
 * - Select permissions from available list
 * - Configure group settings (priority, expiry, description)
 * - Validation and error handling
 */

'use client'

import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { 
  Shield, Key, AlertTriangle, CheckCircle, Info,
  Search, X, Plus, Minus, Star, Clock, Users,
  Globe, MapPin, Smartphone, Monitor, Cpu, Database
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Checkbox } from '@/components/ui/form-components'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useToast } from '@/components/ui/use-toast'

import { 
  usePermissionGroups, 
  useAvailablePermissions 
} from '@/hooks/useGroupPermissions'
import { 
  PermissionGroup, 
  CreateGroupRequest, 
  UpdateGroupRequest 
} from '@/lib/api/group-management-client'
import { adminCardVariants, adminButtonVariants } from '@/design-system'
import { cn } from '@/lib/shared'

interface GroupEditorProps {
  group?: PermissionGroup | null
  onSave: (group: PermissionGroup) => void
  onCancel: () => void
  className?: string
}

interface PermissionCategory {
  name: string
  icon: React.ReactNode
  permissions: string[]
  description: string
}

export function GroupEditor({ group, onSave, onCancel, className }: GroupEditorProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)
  
  // Form state
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    permissions: [] as string[],
    priority_level: 1,
    default_expiry_days: null as number | null
  })
  
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string>('all')

  // Hooks
  const { createGroup, updateGroup } = usePermissionGroups()
  const { permissions: availablePermissions, isLoading: loadingPermissions } = useAvailablePermissions()

  // Initialize form data when group prop changes
  useEffect(() => {
    if (group) {
      setFormData({
        name: group.name,
        description: group.description || '',
        permissions: [...group.permissions],
        priority_level: group.priority_level,
        default_expiry_days: group.default_expiry_days
      })
    } else {
      setFormData({
        name: '',
        description: '',
        permissions: [],
        priority_level: 1,
        default_expiry_days: null
      })
    }
    setErrors({})
  }, [group])

  // Categorize permissions
  const permissionCategories = useMemo<PermissionCategory[]>(() => {
    const categories: PermissionCategory[] = [
      {
        name: 'Admin',
        icon: <Shield className="h-4 w-4" />,
        permissions: availablePermissions.filter(p => p.startsWith('admin:')),
        description: 'Administrative and system management permissions'
      },
      {
        name: 'Users',
        icon: <Users className="h-4 w-4" />,
        permissions: availablePermissions.filter(p => p.includes('users')),
        description: 'User management and account operations'
      },
      {
        name: 'Analytics',
        icon: <Cpu className="h-4 w-4" />,
        permissions: availablePermissions.filter(p => p.includes('analytics')),
        description: 'Analytics and reporting access'
      },
      {
        name: 'Web3',
        icon: <Globe className="h-4 w-4" />,
        permissions: availablePermissions.filter(p => p.includes('web3') || p.includes('blockchain')),
        description: 'Web3 and blockchain-related permissions'
      },
      {
        name: 'API',
        icon: <Database className="h-4 w-4" />,
        permissions: availablePermissions.filter(p => p.includes('api')),
        description: 'API access and integration permissions'
      },
      {
        name: 'Platform',
        icon: <Monitor className="h-4 w-4" />,
        permissions: availablePermissions.filter(p => p.startsWith('epsx:') || p.startsWith('platform:')),
        description: 'Core platform access permissions'
      }
    ]
    
    return categories
  }, [availablePermissions])

  // Filter permissions based on search and category
  const filteredPermissions = useMemo(() => {
    let permissions = availablePermissions
    
    // Filter by category
    if (selectedCategory !== 'all') {
      const category = permissionCategories.find(c => c.name.toLowerCase() === selectedCategory)
      permissions = category?.permissions || []
    }
    
    // Filter by search term
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase()
      permissions = permissions.filter(p => p.toLowerCase().includes(searchLower))
    }
    
    return permissions.sort()
  }, [availablePermissions, permissionCategories, selectedCategory, searchTerm])

  // Form validation
  const validateForm = useCallback(() => {
    const newErrors: Record<string, string> = {}
    
    if (!formData.name.trim()) {
      newErrors.name = 'Group name is required'
    } else if (formData.name.length < 3) {
      newErrors.name = 'Group name must be at least 3 characters'
    }
    
    if (formData.permissions.length === 0) {
      newErrors.permissions = 'At least one permission must be selected'
    }
    
    if (formData.priority_level < 0 || formData.priority_level > 10) {
      newErrors.priority_level = 'Priority level must be between 0 and 10'
    }
    
    if (formData.default_expiry_days !== null && formData.default_expiry_days <= 0) {
      newErrors.default_expiry_days = 'Expiry days must be positive'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }, [formData])

  // Event handlers
  const handleInputChange = useCallback((field: string, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }))
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }))
    }
  }, [errors])

  const handlePermissionToggle = useCallback((permission: string) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.includes(permission)
        ? prev.permissions.filter(p => p !== permission)
        : [...prev.permissions, permission]
    }))
    if (errors.permissions) {
      setErrors(prev => ({ ...prev, permissions: '' }))
    }
  }, [errors.permissions])

  const handleSave = useCallback(async () => {
    if (!validateForm()) return
    
    setLoading(true)
    try {
      const requestData = {
        name: formData.name.trim(),
        description: formData.description.trim() || undefined,
        permissions: formData.permissions,
        priority_level: formData.priority_level,
        default_expiry_days: formData.default_expiry_days || undefined
      }
      
      const savedGroup = group 
        ? await updateGroup(group.id, requestData as UpdateGroupRequest)
        : await createGroup(requestData as CreateGroupRequest)
      
      onSave(savedGroup)
    } catch (error) {
      toast({
        title: 'Save Failed',
        description: error instanceof Error ? error.message : 'Failed to save group',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [formData, group, validateForm, createGroup, updateGroup, onSave, toast])

  const getPriorityLabel = (priority: number) => {
    if (priority >= 8) return 'Critical'
    if (priority >= 6) return 'High'
    if (priority >= 4) return 'Medium'
    return 'Low'
  }

  const getPriorityColor = (priority: number) => {
    if (priority >= 8) return 'text-red-600'
    if (priority >= 6) return 'text-orange-600'
    if (priority >= 4) return 'text-yellow-600'
    return 'text-green-600'
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* System Group Warning */}
      {group?.is_system_group && (
        <Alert>
          <Star className="h-4 w-4" />
          <AlertDescription>
            This is a system group. Only certain properties can be modified.
          </AlertDescription>
        </Alert>
      )}

      {/* Basic Information */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Group Information
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="name">Group Name *</Label>
            <Input
              id="name"
              value={formData.name}
              onChange={(e) => handleInputChange('name', e.target.value)}
              placeholder="Enter group name"
              disabled={group?.is_system_group}
              className={errors.name ? 'border-red-300' : ''}
            />
            {errors.name && (
              <p className="text-sm text-red-600 mt-1">{errors.name}</p>
            )}
          </div>

          <div>
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={formData.description}
              onChange={(e) => handleInputChange('description', e.target.value)}
              placeholder="Optional description for this group"
              rows={3}
              disabled={group?.is_system_group}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label htmlFor="priority">Priority Level *</Label>
              <Select
                value={formData.priority_level.toString()}
                onValueChange={(value) => handleInputChange('priority_level', parseInt(value))}
              >
                <SelectTrigger className={errors.priority_level ? 'border-red-300' : ''}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {[...Array(11)].map((_, i) => (
                    <SelectItem key={i} value={i.toString()}>
                      {i} - {getPriorityLabel(i)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.priority_level && (
                <p className="text-sm text-red-600 mt-1">{errors.priority_level}</p>
              )}
            </div>

            <div>
              <Label htmlFor="expiry">Default Expiry (days)</Label>
              <Input
                id="expiry"
                type="number"
                value={formData.default_expiry_days || ''}
                onChange={(e) => handleInputChange('default_expiry_days', 
                  e.target.value ? parseInt(e.target.value) : null
                )}
                placeholder="Optional expiry in days"
                min="1"
                className={errors.default_expiry_days ? 'border-red-300' : ''}
              />
              {errors.default_expiry_days && (
                <p className="text-sm text-red-600 mt-1">{errors.default_expiry_days}</p>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Permission Selection */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            Permissions ({formData.permissions.length} selected)
          </CardTitle>
          {errors.permissions && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{errors.permissions}</AlertDescription>
            </Alert>
          )}
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Search and Filter */}
          <div className="flex flex-col sm:flex-row gap-4">
            <div className="relative flex-1">
              <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
              <Input
                placeholder="Search permissions..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select value={selectedCategory} onValueChange={setSelectedCategory}>
              <SelectTrigger className="w-full sm:w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {permissionCategories.map((category) => (
                  <SelectItem key={category.name.toLowerCase()} value={category.name.toLowerCase()}>
                    {category.name} ({category.permissions.length})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Selected Permissions Summary */}
          {formData.permissions.length > 0 && (
            <div className="p-3 bg-blue-50 rounded-lg border border-blue-200">
              <h4 className="font-medium text-blue-900 mb-2">Selected Permissions</h4>
              <div className="flex flex-wrap gap-1">
                {formData.permissions.map((permission) => (
                  <Badge 
                    key={permission}
                    variant="secondary"
                    className="cursor-pointer hover:bg-red-100"
                    onClick={() => handlePermissionToggle(permission)}
                  >
                    {permission}
                    <X className="h-3 w-3 ml-1" />
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Permission List */}
          {loadingPermissions ? (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
              <p className="text-sm text-gray-600 mt-2">Loading permissions...</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-96 overflow-y-auto">
              {filteredPermissions.map((permission) => {
                const isSelected = formData.permissions.includes(permission)
                const parts = permission.split(':')
                const platform = parts[0] || ''
                const resource = parts[1] || ''
                const action = parts[2] || ''
                
                return (
                  <div
                    key={permission}
                    className={cn(
                      'flex items-center p-3 rounded-lg border cursor-pointer transition-colors',
                      isSelected 
                        ? 'bg-blue-50 border-blue-200 text-blue-900'
                        : 'bg-white border-gray-200 hover:bg-gray-50'
                    )}
                    onClick={() => handlePermissionToggle(permission)}
                  >
                    <Checkbox 
                      checked={isSelected}
                      onChange={() => handlePermissionToggle(permission)}
                      className="mr-3"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium truncate">
                        {permission}
                      </div>
                      {resource && action && (
                        <div className="text-xs text-gray-500">
                          {platform} → {resource} → {action}
                        </div>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          )}

          {filteredPermissions.length === 0 && !loadingPermissions && (
            <div className="text-center py-8 text-gray-500">
              <Key className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No permissions found matching your search.</p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Actions */}
      <div className="flex justify-end gap-3">
        <Button variant="outline" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
        <Button 
          onClick={handleSave} 
          disabled={loading}
          className={adminButtonVariants({ variant: 'primary' })}
        >
          {loading ? 'Saving...' : (group ? 'Update Group' : 'Create Group')}
        </Button>
      </div>
    </div>
  )
}

export default GroupEditor