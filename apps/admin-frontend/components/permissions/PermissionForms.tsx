/**
 * Permission Forms Component
 * Forms for permission management and assignment
 */

'use client'

import { 
  Shield, User, Clock, AlertTriangle, CheckCircle, Plus, Minus,
  Search, Filter, Save, X, Info, Calendar
} from 'lucide-react'
import React, { useState, useCallback, useMemo } from 'react'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/FormComponents'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { useToast } from '@/components/ui/use-toast'
import { adminCardVariants, adminButtonVariants } from '@/design-system'
import { cn } from '@/lib/shared'

export interface PermissionAssignmentRequest {
  userId: string;
  permissions: string[];
  expiryDate?: string;
  reason?: string;
  priority: number;
}

export interface BulkPermissionRequest {
  userIds: string[];
  permissions: string[];
  operation: 'grant' | 'revoke';
  expiryDate?: string;
  reason?: string;
}

interface PermissionFormProps {
  userId?: string;
  currentPermissions?: string[];
  onSubmit: (request: PermissionAssignmentRequest) => Promise<void>;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.userId
 * @param root0.currentPermissions
 * @param root0.onSubmit
 * @param root0.className
 */
export function PermissionAssignmentForm({ 
  userId, 
  currentPermissions = [], 
  onSubmit,
  className 
}: PermissionFormProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([])
  const [expiryDate, setExpiryDate] = useState('')
  const [reason, setReason] = useState('')
  const [priority, setPriority] = useState(1)
  const [searchTerm, setSearchTerm] = useState('')

  // Available permissions
  const availablePermissions = useMemo(() => [
    'admin:*:*',
    'admin:users:view',
    'admin:users:manage',
    'admin:users:create',
    'admin:users:delete',
    'admin:permissions:view',
    'admin:permissions:manage',
    'admin:analytics:view',
    'admin:analytics:manage',
    'admin:system:view',
    'admin:system:manage',
    'epsx:*:*',
    'epsx:analytics:view',
    'epsx:analytics:premium',
    'epsx:analytics:professional',
    'epsx:api:access',
    'epsx:data:export',
    'web3:*:*',
    'web3:wallet:connect',
    'web3:transactions:view',
    'web3:permissions:manage'
  ], [])

  // Filter permissions based on search
  const filteredPermissions = useMemo(() => {
    if (!searchTerm) {return availablePermissions}
    return availablePermissions.filter(p => 
      p.toLowerCase().includes(searchTerm.toLowerCase())
    )
  }, [availablePermissions, searchTerm])

  const handlePermissionToggle = useCallback((permission: string) => {
    setSelectedPermissions(prev => 
      prev.includes(permission)
        ? prev.filter(p => p !== permission)
        : [...prev, permission]
    )
  }, [])

  const handleSubmit = useCallback(async () => {
    if (!userId || selectedPermissions.length === 0) {
      toast({
        title: 'Invalid Request',
        description: 'User ID and at least one permission are required',
        variant: 'destructive'
      })
      return
    }

    setLoading(true)
    try {
      await onSubmit({
        userId,
        permissions: selectedPermissions,
        expiryDate: expiryDate || undefined,
        reason: reason || undefined,
        priority
      })

      // Reset form
      setSelectedPermissions([])
      setExpiryDate('')
      setReason('')
      setPriority(1)

      toast({
        title: 'Permissions Assigned',
        description: `Successfully assigned ${selectedPermissions.length} permissions`
      })
    } catch (_error) {
      toast({
        title: 'Assignment Failed',
        description: _error instanceof Error ? _error.message : 'Failed to assign permissions',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [userId, selectedPermissions, expiryDate, reason, priority, onSubmit, toast])

  return (
    <div className={cn('space-y-6', className)}>
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Assign Permissions
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Permission Selection */}
          <div>
            <Label>Select Permissions *</Label>
            <div className="mt-2 space-y-3">
              <div className="relative">
                <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
                <Input
                  placeholder="Search permissions..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-2 max-h-64 overflow-y-auto border rounded-lg p-3">
                {filteredPermissions.map((permission) => {
                  const isSelected = selectedPermissions.includes(permission)
                  const isCurrent = currentPermissions.includes(permission)
                  
                  return (
                    <div
                      key={permission}
                      className={cn(
                        'flex items-center p-2 rounded cursor-pointer',
                        isSelected ? 'bg-blue-50 border border-blue-200' : 'hover:bg-gray-50',
                        isCurrent ? 'bg-green-50 border border-green-200' : ''
                      )}
                      onClick={() => handlePermissionToggle(permission)}
                    >
                      <Checkbox
                        checked={isSelected}
                        onChange={() => handlePermissionToggle(permission)}
                        className="mr-3"
                      />
                      <div className="flex-1">
                        <span className="text-sm">{permission}</span>
                        {isCurrent && (
                          <Badge variant="outline" className="ml-2 text-xs">
                            Current
                          </Badge>
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>

              {selectedPermissions.length > 0 && (
                <div className="p-3 bg-blue-50 rounded-lg">
                  <h4 className="font-medium text-blue-900 mb-2">
                    Selected Permissions ({selectedPermissions.length})
                  </h4>
                  <div className="flex flex-wrap gap-1">
                    {selectedPermissions.map((permission) => (
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
            </div>
          </div>

          {/* Assignment Options */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="expiry">Expiry Date (optional)</Label>
              <Input
                id="expiry"
                type="datetime-local"
                value={expiryDate}
                onChange={(e) => setExpiryDate(e.target.value)}
              />
            </div>

            <div>
              <Label htmlFor="priority">Priority Level</Label>
              <Select value={priority.toString()} onValueChange={(value) => setPriority(parseInt(value))}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {[...Array(10)].map((_, i) => (
                    <SelectItem key={i + 1} value={(i + 1).toString()}>
                      {i + 1} - {i + 1 >= 8 ? 'Critical' : i + 1 >= 5 ? 'High' : i + 1 >= 3 ? 'Medium' : 'Low'}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div>
            <Label htmlFor="reason">Reason (optional)</Label>
            <Textarea
              id="reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Reason for permission assignment"
              rows={3}
            />
          </div>

          <div className="flex justify-end">
            <Button 
              onClick={handleSubmit}
              disabled={loading || selectedPermissions.length === 0}
              className={adminButtonVariants({ variant: 'primary' })}
            >
              {loading ? 'Assigning...' : 'Assign Permissions'}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// Bulk Permission Form
interface BulkPermissionFormProps {
  userIds: string[];
  onSubmit: (request: BulkPermissionRequest) => Promise<void>;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.userIds
 * @param root0.onSubmit
 * @param root0.className
 */
export function BulkPermissionForm({ 
  userIds, 
  onSubmit, 
  className 
}: BulkPermissionFormProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)
  const [operation, setOperation] = useState<'grant' | 'revoke'>('grant')
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([])
  const [expiryDate, setExpiryDate] = useState('')
  const [reason, setReason] = useState('')

  const availablePermissions = [
    'admin:users:view',
    'admin:analytics:view',
    'epsx:analytics:view',
    'epsx:analytics:premium',
    'web3:wallet:connect'
  ]

  const handleSubmit = useCallback(async () => {
    if (userIds.length === 0 || selectedPermissions.length === 0) {
      toast({
        title: 'Invalid Request',
        description: 'Users and permissions are required',
        variant: 'destructive'
      })
      return
    }

    setLoading(true)
    try {
      await onSubmit({
        userIds,
        permissions: selectedPermissions,
        operation,
        expiryDate: expiryDate || undefined,
        reason: reason || undefined
      })

      toast({
        title: 'Bulk Operation Complete',
        description: `Successfully ${operation === 'grant' ? 'granted' : 'revoked'} permissions for ${userIds.length} users`
      })
    } catch (_error) {
      toast({
        title: 'Bulk Operation Failed',
        description: _error instanceof Error ? _error.message : 'Failed to process bulk operation',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [userIds, selectedPermissions, operation, expiryDate, reason, onSubmit, toast])

  return (
    <div className={cn('space-y-6', className)}>
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            Bulk Permission Operation
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              This operation will affect {userIds.length} users. Please review carefully before proceeding.
            </AlertDescription>
          </Alert>

          <div>
            <Label>Operation Type</Label>
            <Select value={operation} onValueChange={(value: 'grant' | 'revoke') => setOperation(value)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="grant">Grant Permissions</SelectItem>
                <SelectItem value="revoke">Revoke Permissions</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label>Select Permissions</Label>
            <div className="space-y-2 mt-2">
              {availablePermissions.map((permission) => (
                <div key={permission} className="flex items-center space-x-2">
                  <Checkbox
                    checked={selectedPermissions.includes(permission)}
                    onChange={(checked) => {
                      if (checked) {
                        setSelectedPermissions(prev => [...prev, permission])
                      } else {
                        setSelectedPermissions(prev => prev.filter(p => p !== permission))
                      }
                    }}
                  />
                  <Label className="text-sm">{permission}</Label>
                </div>
              ))}
            </div>
          </div>

          {operation === 'grant' && (
            <div>
              <Label htmlFor="bulkExpiry">Expiry Date (optional)</Label>
              <Input
                id="bulkExpiry"
                type="datetime-local"
                value={expiryDate}
                onChange={(e) => setExpiryDate(e.target.value)}
              />
            </div>
          )}

          <div>
            <Label htmlFor="bulkReason">Reason</Label>
            <Textarea
              id="bulkReason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Reason for bulk permission operation"
              rows={3}
            />
          </div>

          <div className="flex justify-end">
            <Button 
              onClick={handleSubmit}
              disabled={loading || selectedPermissions.length === 0}
              className={adminButtonVariants({ 
                variant: operation === 'revoke' ? 'destructive' : 'primary' 
              })}
            >
              {loading ? 'Processing...' : `${operation === 'grant' ? 'Grant' : 'Revoke'} Permissions`}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// Permission Request Form (for users requesting permissions)
interface PermissionRequestFormProps {
  onSubmit: (request: { permissions: string[]; justification: string }) => Promise<void>;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.onSubmit
 * @param root0.className
 */
export function PermissionRequestForm({ onSubmit, className }: PermissionRequestFormProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)
  const [requestedPermissions, setRequestedPermissions] = useState<string[]>([])
  const [justification, setJustification] = useState('')

  const requestablePermissions = [
    'epsx:analytics:premium',
    'epsx:data:export',
    'admin:analytics:view',
    'web3:permissions:manage'
  ]

  const handleSubmit = useCallback(async () => {
    if (requestedPermissions.length === 0 || !justification.trim()) {
      toast({
        title: 'Incomplete Request',
        description: 'Please select permissions and provide justification',
        variant: 'destructive'
      })
      return
    }

    setLoading(true)
    try {
      await onSubmit({
        permissions: requestedPermissions,
        justification: justification.trim()
      })

      setRequestedPermissions([])
      setJustification('')

      toast({
        title: 'Request Submitted',
        description: 'Your permission request has been submitted for review'
      })
    } catch (_error) {
      toast({
        title: 'Request Failed',
        description: _error instanceof Error ? _error.message : 'Failed to submit request',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }, [requestedPermissions, justification, onSubmit, toast])

  return (
    <div className={cn('space-y-6', className)}>
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Request Permissions
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Select Permissions to Request</Label>
            <div className="space-y-2 mt-2">
              {requestablePermissions.map((permission) => (
                <div key={permission} className="flex items-center space-x-2">
                  <Checkbox
                    checked={requestedPermissions.includes(permission)}
                    onChange={(checked) => {
                      if (checked) {
                        setRequestedPermissions(prev => [...prev, permission])
                      } else {
                        setRequestedPermissions(prev => prev.filter(p => p !== permission))
                      }
                    }}
                  />
                  <Label className="text-sm">{permission}</Label>
                </div>
              ))}
            </div>
          </div>

          <div>
            <Label htmlFor="justification">Justification *</Label>
            <Textarea
              id="justification"
              value={justification}
              onChange={(e) => setJustification(e.target.value)}
              placeholder="Please explain why you need these permissions"
              rows={4}
            />
          </div>

          <div className="flex justify-end">
            <Button 
              onClick={handleSubmit}
              disabled={loading || requestedPermissions.length === 0 || !justification.trim()}
              className={adminButtonVariants({ variant: 'primary' })}
            >
              {loading ? 'Submitting...' : 'Submit Request'}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// Combined export for compatibility
export const PermissionForms = {
  PermissionAssignmentForm,
  BulkPermissionForm,
  PermissionRequestForm
}

export default PermissionForms