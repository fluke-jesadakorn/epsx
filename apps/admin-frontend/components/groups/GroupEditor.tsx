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

import {
  AlertTriangle,
  Key,
  Shield,
  Star
} from 'lucide-react'
import React, { useCallback, useEffect, useState } from 'react'
import { PermissionTransferList } from './PermissionTransferList'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { useToast } from '@/components/ui/use-toast'
import { adminButtonVariants, adminCardVariants } from '@/design-system'
import {
  useAvailablePermissions,
  useGroups
} from '@/hooks/useGroupPermissions'
import {
  CreateGroupRequest,
  Group,
  UpdateGroupRequest
} from '@/lib/api/group-management-client'

interface GroupEditorProps {
  group?: Group | null
  onSave: (group: Group) => void
  onCancel: () => void
  className?: string
}

interface PermissionCategory {
  name: string
  icon: React.ReactNode
  permissions: string[]
  description: string
}

/**
 *
 * @param root0
 * @param root0.group
 * @param root0.onSave
 * @param root0.onCancel
 * @param root0.className
 */
interface GroupFormData {
  name: string;
  description: string;
  permissions: string[];
  priority_level: number;
  default_expiry_days: number | null;
}

export function GroupEditor({ group, onSave, onCancel, className }: GroupEditorProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)

  // Form state
  const [formData, setFormData] = useState<GroupFormData>({
    name: '',
    description: '',
    permissions: [] as string[],
    priority_level: 1,
    default_expiry_days: null
  })

  const [errors, setErrors] = useState<Record<string, string>>({})

  // Hooks
  const { createGroup, updateGroup } = useGroups()
  const { permissions: availablePermissions, isLoading: loadingPermissions } = useAvailablePermissions()

  // Initialize form data when group prop changes
  useEffect(() => {
    if (group) {
      setFormData({
        name: group.name,
        description: group.description || '',
        permissions: [...group.permissions],
        priority_level: group.display_order || 0,
        default_expiry_days: null // default_expiry_days removed from backend response
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
  const handleInputChange = useCallback(<K extends keyof GroupFormData>(field: K, value: GroupFormData[K]) => {
    setFormData(prev => ({ ...prev, [field]: value }))
    if (errors[field as string]) {
      setErrors(prev => ({ ...prev, [field as string]: '' }))
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
    if (!validateForm()) { return }

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
    } catch (_error) {
      toast({
        title: 'Save Failed',
        description: _error instanceof Error ? _error.message : 'Failed to save group',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [formData, group, validateForm, createGroup, updateGroup, onSave, toast])

  const getPriorityLabel = (priority: number) => {
    if (priority >= 8) { return 'Critical' }
    if (priority >= 6) { return 'High' }
    if (priority >= 4) { return 'Medium' }
    return 'Low'
  }

  return (
    <div className={`space-y-6 ${className || ''}`}>
      {/* System Group Warning */}
      {(group?.group_type === 'system' || group?.group_type === 'admin') && (
        <Alert>
          <Star className="h-4 w-4" />
          <AlertDescription>
            This is a system-managed group. Only certain properties can be modified.
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
              disabled={group?.group_type === 'system' || group?.group_type === 'admin'}
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
              disabled={group?.group_type === 'system' || group?.group_type === 'admin'}
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
            {/* Note: default_expiry_days is being deprecated in favor of dynamic rules */}
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
          <CardContent className="space-y-4">
            <PermissionTransferList
              available={availablePermissions}
              selected={formData.permissions}
              onChange={(newSelected: string[]) => {
                setFormData(prev => ({ ...prev, permissions: newSelected }))
                if (errors.permissions) {
                  setErrors(prev => ({ ...prev, permissions: '' }))
                }
              }}
              isLoading={loadingPermissions}
              systemPermissions={new Set(
                availablePermissions.filter(p => p.startsWith('system:') || p.startsWith('admin:'))
              )}
            />
          </CardContent>
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