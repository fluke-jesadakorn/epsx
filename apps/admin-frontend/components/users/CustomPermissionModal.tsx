/**
 * Custom Permission Modal Component
 * Allows adding/removing custom permissions for users
 */

'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/toast'
import { addCustomPermission } from '@/lib/actions/unified-user-actions'

interface CustomPermissionModalProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  userId: string
  userEmail: string
  onPermissionAdded?: () => void
}

const PERMISSION_TEMPLATES = [
  {
    category: 'API Access',
    permissions: [
      { resource: '/api/v1/users', action: 'read', description: 'Read user data' },
      { resource: '/api/v1/users', action: 'write', description: 'Modify user data' },
      { resource: '/api/v1/analytics', action: 'read', description: 'View analytics' },
      { resource: '/api/v1/admin', action: 'read', description: 'Admin read access' },
    ]
  },
  {
    category: 'Modules',
    permissions: [
      { resource: '/api/v1/modules/portfolio-tracker', action: 'read', description: 'Portfolio Tracker access' },
      { resource: '/api/v1/modules/trading-signals', action: 'read', description: 'Trading Signals access' },
      { resource: '/api/v1/modules/risk-management', action: 'read', description: 'Risk Management access' },
      { resource: '/api/v1/modules/backtesting', action: 'read', description: 'Strategy Backtesting access' },
    ]
  },
  {
    category: 'System',
    permissions: [
      { resource: '/system/logs', action: 'read', description: 'View system logs' },
      { resource: '/system/settings', action: 'read', description: 'View system settings' },
      { resource: '/system/cache', action: 'write', description: 'Clear system cache' },
    ]
  }
]

const ACTIONS = [
  { value: 'read', label: 'Read', description: 'View/access permission' },
  { value: 'write', label: 'Write', description: 'Modify/update permission' },
  { value: 'delete', label: 'Delete', description: 'Remove/delete permission' },
  { value: 'admin', label: 'Admin', description: 'Full administrative permission' }
]

export function CustomPermissionModal({
  isOpen,
  onOpenChange,
  userId,
  userEmail,
  onPermissionAdded
}: CustomPermissionModalProps) {
  const [mode, setMode] = useState<'template' | 'custom'>('template')
  const [selectedTemplate, setSelectedTemplate] = useState('')
  const [customResource, setCustomResource] = useState('')
  const [selectedAction, setSelectedAction] = useState('')
  const [reason, setReason] = useState('')
  const [isAdding, setIsAdding] = useState(false)

  // Reset form when modal opens/closes
  useEffect(() => {
    if (!isOpen) {
      setMode('template')
      setSelectedTemplate('')
      setCustomResource('')
      setSelectedAction('')
      setReason('')
    }
  }, [isOpen])

  const handleAddPermission = async () => {
    let resource: string
    let action: string

    if (mode === 'template') {
      if (!selectedTemplate || !selectedAction) {
        toast.error('Please select a permission template and action')
        return
      }
      
      // Find the selected template permission
      const allPermissions = PERMISSION_TEMPLATES.flatMap(cat => cat.permissions)
      const template = allPermissions.find(p => `${p.resource}:${p.action}` === selectedTemplate)
      
      if (!template) {
        toast.error('Invalid template selection')
        return
      }
      
      resource = template.resource
      action = selectedAction
    } else {
      if (!customResource || !selectedAction) {
        toast.error('Please enter a resource path and select an action')
        return
      }
      
      resource = customResource
      action = selectedAction
    }

    setIsAdding(true)
    try {
      const result = await addCustomPermission({
        userId,
        resource,
        action,
        reason: reason || `Custom permission added for ${userEmail}`
      })
      
      if (result.success) {
        toast.success(`Permission "${action}" on "${resource}" added successfully`)
        onPermissionAdded?.()
        onOpenChange(false)
      } else {
        toast.error(result.error?.message || 'Failed to add permission')
      }
    } catch (error) {
      toast.error('Failed to add permission')
    } finally {
      setIsAdding(false)
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Add Custom Permission</DialogTitle>
          <DialogDescription>
            Grant {userEmail} a specific permission for resources or modules.
          </DialogDescription>
        </DialogHeader>
        
        <div className="grid gap-4 py-4">
          <div className="flex gap-2">
            <Button
              type="button"
              variant={mode === 'template' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setMode('template')}
            >
              Use Template
            </Button>
            <Button
              type="button"
              variant={mode === 'custom' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setMode('custom')}
            >
              Custom Permission
            </Button>
          </div>

          {mode === 'template' ? (
            <div className="grid gap-3">
              <Label>Permission Template</Label>
              <Select value={selectedTemplate} onValueChange={setSelectedTemplate}>
                <SelectTrigger>
                  <SelectValue placeholder="Select a permission template" />
                </SelectTrigger>
                <SelectContent>
                  {PERMISSION_TEMPLATES.map((category) => (
                    <div key={category.category}>
                      <div className="px-2 py-1.5 text-sm font-semibold text-muted-foreground">
                        {category.category}
                      </div>
                      {category.permissions.map((perm) => {
                        const key = `${perm.resource}:${perm.action}`
                        return (
                          <SelectItem key={key} value={key}>
                            <div className="flex flex-col">
                              <span className="font-medium">{perm.resource}</span>
                              <span className="text-xs text-muted-foreground">
                                {perm.description} ({perm.action})
                              </span>
                            </div>
                          </SelectItem>
                        )
                      })}
                    </div>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : (
            <div className="grid gap-3">
              <div className="grid gap-2">
                <Label htmlFor="resource">Resource Path</Label>
                <Input
                  id="resource"
                  placeholder="/api/v1/custom/resource"
                  value={customResource}
                  onChange={(e) => setCustomResource(e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  Enter the full resource path (e.g., /api/v1/users, /modules/custom-tool)
                </p>
              </div>
            </div>
          )}

          <div className="grid gap-2">
            <Label>Action</Label>
            <Select value={selectedAction} onValueChange={setSelectedAction}>
              <SelectTrigger>
                <SelectValue placeholder="Select an action" />
              </SelectTrigger>
              <SelectContent>
                {ACTIONS.map((action) => (
                  <SelectItem key={action.value} value={action.value}>
                    <div className="flex flex-col">
                      <span className="font-medium">{action.label}</span>
                      <span className="text-xs text-muted-foreground">{action.description}</span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          
          <div className="grid gap-2">
            <Label htmlFor="reason">Reason (Optional)</Label>
            <Textarea
              id="reason"
              placeholder="Enter reason for granting this permission..."
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="min-h-[80px]"
            />
          </div>

          {selectedTemplate && mode === 'template' && (
            <div className="rounded-md bg-muted p-3">
              <h4 className="text-sm font-medium mb-2">Permission Preview:</h4>
              <div className="flex items-center gap-2">
                <Badge variant="outline">
                  {selectedTemplate.split(':')[0]} : {selectedAction || 'ACTION'}
                </Badge>
              </div>
            </div>
          )}

          {customResource && selectedAction && mode === 'custom' && (
            <div className="rounded-md bg-muted p-3">
              <h4 className="text-sm font-medium mb-2">Permission Preview:</h4>
              <div className="flex items-center gap-2">
                <Badge variant="outline">
                  {customResource} : {selectedAction}
                </Badge>
              </div>
            </div>
          )}
        </div>
        
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button 
            onClick={handleAddPermission}
            disabled={isAdding || !selectedAction || (mode === 'template' ? !selectedTemplate : !customResource)}
            className="min-w-[100px]"
          >
            {isAdding ? 'Adding...' : 'Add Permission'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}