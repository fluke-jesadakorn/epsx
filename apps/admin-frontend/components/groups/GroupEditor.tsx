/**
 * Group Editor Component
 * Interface for creating and editing permission groups
 *
 * Features:
 * - Create new permission groups
 * - Edit existing groups (except system groups)
 * - Permissions management via TransferList (Two-column drag-and-drop)
 * - Human-readable permission display
 * - Configure group settings (priority, expiry, description)
 * - Validation and error handling
 */

'use client'

import {
  Check,
  Key,
  Shield,
  Star
} from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { useToast } from '@/components/ui/use-toast'
import { adminCardVariants } from '@/design-system'
import {
  useAvailablePermissions,
  useGroups
} from '@/hooks/useGroupPermissions'
import {
  CreateGroupRequest,
  Group,
  UpdateGroupRequest
} from '@/lib/api/group-management-client'
import { cn } from '@/lib/utils'
import { TransferList } from '@/shared/components/ui/transfer-list/TransferList'

interface GroupEditorProps {
  group?: Group | null
  onSave: (group: Group) => void
  onCancel: () => void
  className?: string
}

interface GroupFormData {
  name: string;
  description: string;
  permissions: string[];
  priority_level: number;
  default_expiry_days: number | null;
}

interface PermissionItem {
  id: string
  title: string
  description: string
  platform: string
}

// Helper to format permission string into human-readable object
const parsePermission = (permId: string): PermissionItem => {
  const parts = permId.split(':')
  // Format: platform:resource:action -> Action Resource (Platform)
  // Example: epsx:admin:write -> Write Admin (epsx)

  let platform = parts[0] || 'Unknown'
  let resource = parts[1] || ''
  let action = parts[2] || ''

  // Fallback for wildcards or non-standard
  if (permId === '*') {
    return {
      id: permId,
      title: 'Full Admin Access',
      description: 'Super admin privileges',
      platform: 'All'
    }
  }

  // Capitalize helper
  const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1)

  const title = action && resource
    ? `${cap(action)} ${cap(resource)}`
    : permId

  return {
    id: permId,
    title,
    description: `Platform: ${cap(platform)}`,
    platform: cap(platform)
  }
}

/**
 *
 * @param root0
 * @param root0.group
 * @param root0.onSave
 * @param root0.onCancel
 * @param root0.className
 */
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
  const { permissions: availablePermissionsRaw, isLoading: loadingPermissions } = useAvailablePermissions()

  // Prepare TransferList Data
  const allPermissions: PermissionItem[] = useMemo(() => {
    return availablePermissionsRaw.map(parsePermission)
  }, [availablePermissionsRaw])

  const selectedPermissions: PermissionItem[] = useMemo(() => {
    return formData.permissions.map(parsePermission)
  }, [formData.permissions])

  const availablePermissionsForList: PermissionItem[] = useMemo(() => {
    // Filter out already selected items from the "Available" list
    return allPermissions.filter(p => !formData.permissions.includes(p.id))
  }, [allPermissions, formData.permissions])


  // Initialize form data when group prop changes
  useEffect(() => {
    if (group) {
      setFormData({
        name: group.name || '', // Ensure never undefined
        description: group.description || '',
        permissions: [...(group.permissions || [])], // Safe fallback
        priority_level: group.priority_level ?? group.display_order ?? 0,
        // Backend might return expiry in different fields, prioritizing default_expiry_days
        default_expiry_days: group.default_expiry_days ?? null
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

    if (formData.priority_level < 0 || formData.priority_level > 100) { // Relaxed validation
      newErrors.priority_level = 'Priority level must be between 0 and 100'
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

  const handlePermissionsChange = useCallback((newSelected: PermissionItem[]) => {
    setFormData(prev => ({
      ...prev,
      permissions: newSelected.map(p => p.id)
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
        default_expiry_days: formData.default_expiry_days === null ? undefined : formData.default_expiry_days
      }

      console.log('Saving Group:', requestData)

      const savedGroup = group
        ? await updateGroup(group.id, requestData as UpdateGroupRequest)
        : await createGroup(requestData as CreateGroupRequest)

      onSave(savedGroup)
    } catch (_error) {
      console.error('Save failed:', _error)
      toast({
        title: 'Save Failed',
        description: _error instanceof Error ? _error.message : 'Failed to save group',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [formData, group, validateForm, createGroup, updateGroup, onSave, toast])


  // Renderer for TransferList Items
  const renderPermissionItem = useCallback((item: PermissionItem, type: 'available' | 'selected') => {
    const isSelected = type === 'selected'
    return (
      <div className={cn(
        "flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-all",
        "border hover:shadow-sm select-none overflow-hidden",
        isSelected
          ? "bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-red-300 dark:hover:border-red-700"
          : "bg-white dark:bg-gray-900/50 border-gray-200 dark:border-gray-800 hover:border-green-300 dark:hover:border-green-700"
      )}>
        <span className={cn(
          "flex-shrink-0 p-1.5 rounded-md",
          isSelected ? "bg-red-50 text-red-500" : "bg-blue-50 text-blue-500 delay-75"
        )}>
          <Key className="h-4 w-4" />
        </span>

        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-gray-900 dark:text-white truncate">
            {item.title}
          </p>
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-500 dark:text-gray-400 font-mono bg-gray-100 dark:bg-gray-800 px-1 rounded">
              {item.id}
            </span>
          </div>
        </div>

        <span className={cn(
          "flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity",
          isSelected ? "text-red-400" : "text-green-400"
        )}>
          {isSelected ? <Shield className="h-4 w-4" /> : <Check className="h-4 w-4" />}
        </span>
      </div>
    )
  }, [])


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

      {/* Main Form Card */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardContent className="space-y-6 pt-6">

          {/* Group Header Info */}
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <Label htmlFor="name">Group Name</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => handleInputChange('name', e.target.value)}
                placeholder="e.g. Starter Plan"
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
                placeholder="Ideal for individual investors"
                rows={1}
                disabled={group?.group_type === 'system' || group?.group_type === 'admin'}
                className="resize-none"
              />
            </div>
          </div>

          {/* Permissions Transfer List */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label className="text-base">Permissions</Label>
              {errors.permissions && (
                <span className="text-sm text-red-600">{errors.permissions}</span>
              )}
            </div>

            <div className="bg-gray-50/50 dark:bg-gray-900/20 p-4 rounded-xl border border-gray-100 dark:border-gray-800">
              <TransferList
                available={availablePermissionsForList}
                selected={selectedPermissions}
                onChange={handlePermissionsChange}
                renderItem={renderPermissionItem}
                keyExtractor={(item) => item.id}
                availableTitle="Available Permissions"
                selectedTitle="Authorized Permissions"
                className="h-[500px]"
                isLoading={loadingPermissions}
                showSelection={true}
                emptyStateAvailable={
                  <div className="flex flex-col items-center justify-center h-full text-gray-400 space-y-2">
                    <Shield className="w-8 h-8 opacity-20" />
                    <p className="text-sm">No more permissions available</p>
                  </div>
                }
              />
            </div>
          </div>

          {/* Settings Row */}
          <div className="grid grid-cols-2 gap-4 pt-2">
            <div>
              <Label htmlFor="expiry">Default Expiry (Days)</Label>
              <Input
                id="expiry"
                type="number"
                value={formData.default_expiry_days || ''}
                onChange={(e) => handleInputChange('default_expiry_days',
                  e.target.value ? parseInt(e.target.value) : null
                )}
                placeholder="30"
                min="0"
                className={errors.default_expiry_days ? 'border-red-300' : ''}
              />
              <p className="text-xs text-gray-500 mt-1">Leave empty for no expiry</p>
              {errors.default_expiry_days && (
                <p className="text-sm text-red-600 mt-1">{errors.default_expiry_days}</p>
              )}
            </div>

            <div>
              <Label htmlFor="priority">Priority Level</Label>
              <Input
                id="priority"
                type="number"
                value={formData.priority_level}
                onChange={(e) => handleInputChange('priority_level', parseInt(e.target.value) || 0)}
                placeholder="0"
                min="0"
                className={errors.priority_level ? 'border-red-300' : ''}
              />
              <p className="text-xs text-gray-500 mt-1">Higher number = higher precedence</p>
              {errors.priority_level && (
                <p className="text-sm text-red-600 mt-1">{errors.priority_level}</p>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Action Buttons */}
      <div className="flex flex-col sm:flex-row justify-end gap-3 pt-2">
        <Button
          variant="ghost"
          onClick={onCancel}
          disabled={loading}
        >
          Cancel
        </Button>
        <Button
          onClick={handleSave}
          disabled={loading}
          className="bg-orange-500 hover:bg-orange-600 text-white font-semibold min-w-[150px] shadow-sm"
        >
          {loading ? 'Saving...' : (group ? 'Update Group' : 'Create Group')}
        </Button>
      </div>
    </div>
  )
}

export default GroupEditor