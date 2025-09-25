/**
 * Bulk Permission Manager Component
 * Manage permissions for multiple users at once
 */

'use client'

import { useState } from 'react'
import { Users, Plus, Minus, Upload, Download, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { bulkAssignPermissions, bulkRemovePermissions } from '@/lib/actions/consolidated-user-actions'
import { useToast } from '@/components/ui/use-toast'

interface BulkPermissionManagerProps {
  selectedUserIds: string[]
  onBulkActionComplete?: () => void
  className?: string
}

const PERMISSION_TEMPLATES = [
  {
    id: 'free-template',
    name: 'Free Template',
    description: 'Basic free tier permissions',
    permissions: [
      { resource: 'epsx', action: 'rankings:view:3' },
      { resource: 'epsx', action: 'trading:basic' },
      { resource: 'epsx', action: 'portfolio:view' }
    ]
  },
  {
    id: 'bronze-template',
    name: 'Bronze Template',
    description: 'Enhanced access with basic features',
    permissions: [
      { resource: 'epsx', action: 'rankings:view:5' },
      { resource: 'epsx', action: 'trading:basic' },
      { resource: 'epsx', action: 'portfolio:view' },
      { resource: 'epsx', action: 'portfolio:history' }
    ]
  },
  {
    id: 'silver-template',
    name: 'Silver Template',
    description: 'Premium access with advanced analytics',
    permissions: [
      { resource: 'epsx', action: 'rankings:view:25' },
      { resource: 'epsx', action: 'trading:basic' },
      { resource: 'epsx', action: 'trading:advanced' },
      { resource: 'epsx', action: 'portfolio:view' },
      { resource: 'epsx', action: 'analytics:basic' }
    ]
  },
  {
    id: 'gold-template',
    name: 'Gold Template',
    description: 'Professional access with premium tools',
    permissions: [
      { resource: 'epsx', action: 'rankings:view:50' },
      { resource: 'epsx', action: 'trading:premium' },
      { resource: 'epsx', action: 'portfolio:tools' },
      { resource: 'epsx', action: 'analytics:advanced' }
    ]
  },
  {
    id: 'platinum-template',
    name: 'Platinum Template',
    description: 'VIP access with advanced features',
    permissions: [
      { resource: 'epsx', action: 'rankings:view:100' },
      { resource: 'epsx', action: 'trading:premium' },
      { resource: 'epsx', action: 'analytics:premium' },
      { resource: 'epsx', action: 'research:reports' },
      { resource: 'epsx', action: 'dashboards:custom' }
    ]
  },
  {
    id: 'enterprise-template',
    name: 'Enterprise Template',
    description: 'Unlimited access with all platform features',
    permissions: [
      { resource: 'epsx', action: 'rankings:view:unlimited' },
      { resource: 'epsx', action: '*:*' },
      { resource: 'epsx-pay', action: '*:*' },
      { resource: 'epsx-token', action: '*:*' }
    ]
  },
  {
    id: 'admin-template',
    name: 'Admin Template',
    description: 'Full administrative access',
    permissions: [
      { resource: 'admin', action: '*:*' },
      { resource: 'epsx', action: '*:*' },
      { resource: 'epsx-pay', action: '*:*' },
      { resource: 'epsx-token', action: '*:*' }
    ]
  }
]

export function BulkPermissionManager({ 
  selectedUserIds, 
  onBulkActionComplete,
  className = ''
}: BulkPermissionManagerProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)
  const [activeTab, setActiveTab] = useState('assign')
  
  // Form state
  const [assignForm, setAssignForm] = useState({
    permissions: [{ resource: '', action: '' }],
    template: '',
    reason: ''
  })
  
  const [removeForm, setRemoveForm] = useState({
    permissions: [{ resource: '', action: '' }],
    reason: ''
  })

  const handleAddPermission = (formType: 'assign' | 'remove') => {
    if (formType === 'assign') {
      setAssignForm(prev => ({
        ...prev,
        permissions: [...prev.permissions, { resource: '', action: '' }]
      }))
    } else {
      setRemoveForm(prev => ({
        ...prev,
        permissions: [...prev.permissions, { resource: '', action: '' }]
      }))
    }
  }

  const handleRemovePermission = (formType: 'assign' | 'remove', index: number) => {
    if (formType === 'assign') {
      setAssignForm(prev => ({
        ...prev,
        permissions: prev.permissions.filter((_, i) => i !== index)
      }))
    } else {
      setRemoveForm(prev => ({
        ...prev,
        permissions: prev.permissions.filter((_, i) => i !== index)
      }))
    }
  }

  const handlePermissionChange = (
    formType: 'assign' | 'remove',
    index: number,
    field: 'resource' | 'action',
    value: string
  ) => {
    if (formType === 'assign') {
      setAssignForm(prev => ({
        ...prev,
        permissions: prev.permissions.map((perm, i) =>
          i === index ? { ...perm, [field]: value } : perm
        )
      }))
    } else {
      setRemoveForm(prev => ({
        ...prev,
        permissions: prev.permissions.map((perm, i) =>
          i === index ? { ...perm, [field]: value } : perm
        )
      }))
    }
  }

  const applyTemplate = (templateId: string) => {
    const template = PERMISSION_TEMPLATES.find(t => t.id === templateId)
    if (template) {
      setAssignForm(prev => ({
        ...prev,
        template: templateId,
        permissions: template.permissions
      }))
    }
  }

  const handleBulkAssign = async (e: React.FormEvent) => {
    e.preventDefault()
    
    const validPermissions = assignForm.permissions.filter(p => p.resource && p.action)
    if (validPermissions.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please specify at least one permission to assign',
        variant: 'destructive'
      })
      return
    }

    try {
      setLoading(true)
      
      const result = await bulkAssignPermissions({
        userIds: selectedUserIds,
        permissions: validPermissions,
        reason: assignForm.reason || undefined
      })

      if (result.success) {
        const { succeeded, failed } = result.data || { succeeded: [], failed: [] }
        
        let message = `Successfully assigned permissions to ${succeeded.length} user(s)`
        if (failed.length > 0) {
          message += `. Failed for ${failed.length} user(s).`
        }
        
        toast({
          title: 'Bulk Assignment Complete',
          description: message,
          variant: failed.length > 0 ? 'destructive' : 'default'
        })
        
        // Reset form
        setAssignForm({
          permissions: [{ resource: '', action: '' }],
          template: '',
          reason: ''
        })
        
        onBulkActionComplete?.()
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to assign permissions',
          variant: 'destructive'
        })
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      })
      console.error('Bulk assign error:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleBulkRemove = async (e: React.FormEvent) => {
    e.preventDefault()
    
    const validPermissions = removeForm.permissions.filter(p => p.resource && p.action)
    if (validPermissions.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please specify at least one permission to remove',
        variant: 'destructive'
      })
      return
    }

    try {
      setLoading(true)
      
      const result = await bulkRemovePermissions({
        userIds: selectedUserIds,
        permissions: validPermissions,
        reason: removeForm.reason || undefined
      })

      if (result.success) {
        const { succeeded, failed } = result.data || { succeeded: [], failed: [] }
        
        let message = `Successfully removed permissions from ${succeeded.length} user(s)`
        if (failed.length > 0) {
          message += `. Failed for ${failed.length} user(s).`
        }
        
        toast({
          title: 'Bulk Removal Complete',
          description: message,
          variant: failed.length > 0 ? 'destructive' : 'default'
        })
        
        // Reset form
        setRemoveForm({
          permissions: [{ resource: '', action: '' }],
          reason: ''
        })
        
        onBulkActionComplete?.()
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to remove permissions',
          variant: 'destructive'
        })
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      })
      console.error('Bulk remove error:', error)
    } finally {
      setLoading(false)
    }
  }

  if (selectedUserIds.length === 0) {
    return (
      <div className={`text-center text-muted-foreground py-8 ${className}`}>
        <Users className="h-8 w-8 mx-auto mb-2 opacity-50" />
        <p>No users selected</p>
        <p className="text-xs">Select users to perform bulk operations</p>
      </div>
    )
  }

  return (
    <div className={className}>
      <div className="mb-4">
        <Badge variant="secondary" className="mb-2">
          {selectedUserIds.length} user(s) selected
        </Badge>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="assign" className="flex items-center gap-2">
            <Plus className="h-4 w-4" />
            Assign Permissions
          </TabsTrigger>
          <TabsTrigger value="remove" className="flex items-center gap-2">
            <Minus className="h-4 w-4" />
            Remove Permissions
          </TabsTrigger>
        </TabsList>

        <TabsContent value="assign" className="space-y-4">
          <form onSubmit={handleBulkAssign} className="space-y-4">
            {/* Template Selection */}
            <div>
              <Label htmlFor="template">Quick Templates (Optional)</Label>
              <Select
                value={assignForm.template}
                onValueChange={applyTemplate}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Choose a permission template..." />
                </SelectTrigger>
                <SelectContent>
                  {PERMISSION_TEMPLATES.map((template) => (
                    <SelectItem key={template.id} value={template.id}>
                      <div>
                        <div className="font-medium">{template.name}</div>
                        <div className="text-xs text-muted-foreground">{template.description}</div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Custom Permissions */}
            <div>
              <Label>Permissions to Assign</Label>
              <div className="space-y-2">
                {assignForm.permissions.map((permission, index) => (
                  <div key={index} className="flex gap-2 items-end">
                    <div className="flex-1">
                      <Input
                        placeholder="Resource (e.g., admin, users)"
                        value={permission.resource}
                        onChange={(e) => handlePermissionChange('assign', index, 'resource', e.target.value)}
                      />
                    </div>
                    <div className="flex-1">
                      <Input
                        placeholder="Action (e.g., read, write)"
                        value={permission.action}
                        onChange={(e) => handlePermissionChange('assign', index, 'action', e.target.value)}
                      />
                    </div>
                    {assignForm.permissions.length > 1 && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => handleRemovePermission('assign', index)}
                      >
                        <Minus className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                ))}
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => handleAddPermission('assign')}
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Add Permission
                </Button>
              </div>
            </div>

            {/* Reason */}
            <div>
              <Label htmlFor="assignReason">Reason (Optional)</Label>
              <Textarea
                id="assignReason"
                value={assignForm.reason}
                onChange={(e) => setAssignForm(prev => ({ ...prev, reason: e.target.value }))}
                placeholder="Explain why these permissions are being assigned..."
                rows={2}
              />
            </div>

            <Button type="submit" disabled={loading} className="w-full">
              {loading ? (
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
              ) : (
                <Plus className="h-4 w-4 mr-2" />
              )}
              Assign to {selectedUserIds.length} User(s)
            </Button>
          </form>
        </TabsContent>

        <TabsContent value="remove" className="space-y-4">
          <form onSubmit={handleBulkRemove} className="space-y-4">
            {/* Permissions to Remove */}
            <div>
              <Label>Permissions to Remove</Label>
              <div className="space-y-2">
                {removeForm.permissions.map((permission, index) => (
                  <div key={index} className="flex gap-2 items-end">
                    <div className="flex-1">
                      <Input
                        placeholder="Resource (e.g., admin, users)"
                        value={permission.resource}
                        onChange={(e) => handlePermissionChange('remove', index, 'resource', e.target.value)}
                      />
                    </div>
                    <div className="flex-1">
                      <Input
                        placeholder="Action (e.g., read, write)"
                        value={permission.action}
                        onChange={(e) => handlePermissionChange('remove', index, 'action', e.target.value)}
                      />
                    </div>
                    {removeForm.permissions.length > 1 && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => handleRemovePermission('remove', index)}
                      >
                        <Minus className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                ))}
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => handleAddPermission('remove')}
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Add Permission
                </Button>
              </div>
            </div>

            {/* Reason */}
            <div>
              <Label htmlFor="removeReason">Reason (Optional)</Label>
              <Textarea
                id="removeReason"
                value={removeForm.reason}
                onChange={(e) => setRemoveForm(prev => ({ ...prev, reason: e.target.value }))}
                placeholder="Explain why these permissions are being removed..."
                rows={2}
              />
            </div>

            <Button type="submit" disabled={loading} variant="destructive" className="w-full">
              {loading ? (
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
              ) : (
                <Minus className="h-4 w-4 mr-2" />
              )}
              Remove from {selectedUserIds.length} User(s)
            </Button>
          </form>
        </TabsContent>
      </Tabs>
    </div>
  )
}