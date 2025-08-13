/**
 * Permission Profile Modal - Create/Edit permission profiles
 */

'use client'

import { useState, useEffect } from 'react'
import { X, Plus, Trash2, Shield, Key } from 'lucide-react'
import { Button } from '@epsx/ui'
import { Input } from '@epsx/ui'
import { Label } from '@epsx/ui'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui'
import { Switch } from '@/components/ui/switch'
import { Separator } from '@/components/ui/separator'
import { Badge } from '@epsx/ui'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@epsx/ui'
import type { PermissionProfile, Permission } from '@/lib/types/permission-profiles'

interface PermissionProfileModalProps {
  open: boolean
  onClose: () => void
  onSubmit: (data: Partial<PermissionProfile>) => Promise<{ success: boolean; error?: string }>
  initialData?: PermissionProfile
  title: string
  submitText: string
}

const CATEGORIES = [
  { id: 'user', name: 'User', description: 'Standard user permissions' },
  { id: 'moderator', name: 'Moderator', description: 'Content moderation permissions' },
  { id: 'admin', name: 'Administrator', description: 'Administrative permissions' },
  { id: 'custom', name: 'Custom', description: 'Custom permission sets' },
  { id: 'system', name: 'System', description: 'System-level permissions' },
  { id: 'business', name: 'Business', description: 'Business-related permissions' },
  { id: 'technical', name: 'Technical', description: 'Technical permissions' },
  { id: 'administrative', name: 'Administrative', description: 'Administrative permissions' },
  { id: 'compliance', name: 'Compliance', description: 'Compliance-related permissions' }
]

const TIERS = [
  { id: 'free', name: 'Free', description: 'Free tier permissions' },
  { id: 'bronze', name: 'Bronze', description: 'Basic tier permissions' },
  { id: 'silver', name: 'Silver', description: 'Silver tier permissions' },
  { id: 'gold', name: 'Gold', description: 'Gold tier permissions' },
  { id: 'platinum', name: 'Platinum', description: 'Platinum tier permissions' },
  { id: 'admin', name: 'Admin', description: 'Administrative tier permissions' },
  { id: 'superadmin', name: 'Super Admin', description: 'Super administrative tier permissions' }
]

const COMMON_RESOURCES = [
  '/api/v1/users',
  '/api/v1/admin',
  '/api/v1/trading',
  '/api/v1/portfolio',
  '/api/v1/market-data',
  '/api/v1/analytics',
  '/api/v1/billing',
  '/api/v1/settings'
]

const COMMON_ACTIONS = [
  'read',
  'write',
  'create',
  'update',
  'delete',
  'manage',
  'access',
  'execute'
]

export function PermissionProfileModal({
  open,
  onClose,
  onSubmit,
  initialData,
  title,
  submitText
}: PermissionProfileModalProps) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    category: 'custom',
    targetTier: 'free',
    isActive: true,
    permissions: [] as Permission[]
  })
  
  const [newPermission, setNewPermission] = useState({ resource: '', action: '' })
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [errors, setErrors] = useState<Record<string, string>>({})

  useEffect(() => {
    if (initialData) {
      setFormData({
        name: initialData.name,
        description: initialData.description,
        category: initialData.category,
        targetTier: initialData.targetTier,
        isActive: initialData.isActive,
        permissions: [...initialData.permissions]
      })
    } else {
      setFormData({
        name: '',
        description: '',
        category: 'custom',
        targetTier: 'free',
        isActive: true,
        permissions: []
      })
    }
    setErrors({})
  }, [initialData, open])

  const validateForm = () => {
    const newErrors: Record<string, string> = {}

    if (!formData.name.trim()) {
      newErrors.name = 'Profile name is required'
    }

    if (!formData.description.trim()) {
      newErrors.description = 'Description is required'
    }

    if (formData.permissions.length === 0) {
      newErrors.permissions = 'At least one permission is required'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) {
      return
    }

    setIsSubmitting(true)
    
    try {
      const result = await onSubmit(formData)
      if (result.success) {
        onClose()
      } else {
        setErrors({ general: result.error || 'Failed to save profile' })
      }
    } catch (error) {
      setErrors({ general: 'An unexpected error occurred' })
    } finally {
      setIsSubmitting(false)
    }
  }

  const addPermission = () => {
    if (!newPermission.resource.trim() || !newPermission.action.trim()) {
      return
    }

    const exists = formData.permissions.some(
      p => p.resource === newPermission.resource && p.action === newPermission.action
    )

    if (exists) {
      return
    }

    setFormData(prev => ({
      ...prev,
      permissions: [...prev.permissions, { ...newPermission }]
    }))
    
    setNewPermission({ resource: '', action: '' })
  }

  const removePermission = (index: number) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.filter((_, i) => i !== index)
    }))
  }

  const addCommonPermissions = (resourcePrefix: string) => {
    const commonPermissions = ['read', 'write', 'create', 'update', 'delete']
    const newPermissions = commonPermissions.map(action => ({
      resource: resourcePrefix,
      action
    })).filter(newPerm => 
      !formData.permissions.some(existing => 
        existing.resource === newPerm.resource && existing.action === newPerm.action
      )
    )

    setFormData(prev => ({
      ...prev,
      permissions: [...prev.permissions, ...newPermissions]
    }))
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-purple-500" />
            {title}
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Basic Information */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="name">Profile Name *</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                placeholder="e.g., Advanced Trader"
                className={errors.name ? 'border-red-500' : ''}
              />
              {errors.name && (
                <p className="text-sm text-red-600 mt-1">{errors.name}</p>
              )}
            </div>

            <div>
              <Label htmlFor="category">Category *</Label>
              <Select
                value={formData.category}
                onValueChange={(value) => setFormData(prev => ({ ...prev, category: value }))}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CATEGORIES.map((category) => (
                    <SelectItem key={category.id} value={category.id}>
                      <div>
                        <div className="font-medium">{category.name}</div>
                        <div className="text-sm text-muted-foreground">{category.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="targetTier">Target Tier *</Label>
              <Select
                value={formData.targetTier}
                onValueChange={(value) => setFormData(prev => ({ ...prev, targetTier: value }))}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {TIERS.map((tier) => (
                    <SelectItem key={tier.id} value={tier.id}>
                      <div>
                        <div className="font-medium">{tier.name}</div>
                        <div className="text-sm text-muted-foreground">{tier.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center space-x-2">
              <Switch
                id="isActive"
                checked={formData.isActive}
                onCheckedChange={(checked) => setFormData(prev => ({ ...prev, isActive: checked }))}
              />
              <Label htmlFor="isActive">Profile is active</Label>
            </div>
          </div>

          <div>
            <Label htmlFor="description">Description *</Label>
            <Textarea
              id="description"
              value={formData.description}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
              placeholder="Describe what this profile provides..."
              rows={3}
              className={errors.description ? 'border-red-500' : ''}
            />
            {errors.description && (
              <p className="text-sm text-red-600 mt-1">{errors.description}</p>
            )}
          </div>

          <Separator />

          {/* Permissions Section */}
          <div>
            <div className="flex items-center justify-between mb-4">
              <div>
                <Label className="text-base font-semibold flex items-center gap-2">
                  <Key className="h-4 w-4" />
                  Permissions ({formData.permissions.length})
                </Label>
                <p className="text-sm text-muted-foreground">
                  Define what resources and actions this profile grants access to
                </p>
              </div>
            </div>

            {/* Quick Add Common Permissions */}
            <div className="mb-4">
              <Label className="text-sm">Quick Add Common Permissions:</Label>
              <div className="flex flex-wrap gap-2 mt-2">
                {COMMON_RESOURCES.map((resource) => (
                  <Button
                    key={resource}
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => addCommonPermissions(resource)}
                  >
                    {resource.split('/').pop()}
                  </Button>
                ))}
              </div>
            </div>

            {/* Add New Permission */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-2 mb-4 p-4 border rounded-lg bg-gray-50">
              <div>
                <Label htmlFor="resource">Resource</Label>
                <Input
                  id="resource"
                  value={newPermission.resource}
                  onChange={(e) => setNewPermission(prev => ({ ...prev, resource: e.target.value }))}
                  placeholder="/api/v1/users"
                  list="common-resources"
                />
                <datalist id="common-resources">
                  {COMMON_RESOURCES.map(resource => (
                    <option key={resource} value={resource} />
                  ))}
                </datalist>
              </div>
              
              <div>
                <Label htmlFor="action">Action</Label>
                <Input
                  id="action"
                  value={newPermission.action}
                  onChange={(e) => setNewPermission(prev => ({ ...prev, action: e.target.value }))}
                  placeholder="read"
                  list="common-actions"
                />
                <datalist id="common-actions">
                  {COMMON_ACTIONS.map(action => (
                    <option key={action} value={action} />
                  ))}
                </datalist>
              </div>
              
              <div className="flex items-end">
                <Button
                  type="button"
                  onClick={addPermission}
                  disabled={!newPermission.resource.trim() || !newPermission.action.trim()}
                  className="w-full"
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Add
                </Button>
              </div>
            </div>

            {/* Permissions List */}
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {formData.permissions.map((permission, index) => (
                <div
                  key={`${permission.resource}-${permission.action}-${index}`}
                  className="flex items-center justify-between p-3 border rounded-lg bg-white"
                >
                  <div className="flex items-center gap-4">
                    <Badge variant="outline" className="font-mono">
                      {permission.action}
                    </Badge>
                    <span className="text-sm text-muted-foreground">
                      {permission.resource}
                    </span>
                  </div>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => removePermission(index)}
                    className="text-red-600 hover:text-red-700"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              ))}
              
              {formData.permissions.length === 0 && (
                <p className="text-sm text-muted-foreground text-center py-4">
                  No permissions added yet. Use the form above to add permissions.
                </p>
              )}
            </div>

            {errors.permissions && (
              <p className="text-sm text-red-600 mt-2">{errors.permissions}</p>
            )}
          </div>

          {/* General Error */}
          {errors.general && (
            <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
              {errors.general}
            </div>
          )}
        </form>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button 
            onClick={handleSubmit}
            disabled={isSubmitting}
            className="gap-2"
          >
            {isSubmitting && <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />}
            {submitText}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}