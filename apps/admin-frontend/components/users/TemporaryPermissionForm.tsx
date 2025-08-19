/**
 * Temporary Permission Form Component
 * Allows assigning time-limited permissions
 */

'use client'

import { useState } from 'react'
import { Clock, Plus, Calendar, AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { assignTemporaryPermission, validatePermissionAssignment } from '@/lib/actions/users'
import { useToast } from '@/components/ui/use-toast'
import { format, addDays, addHours, addWeeks } from 'date-fns'

interface TemporaryPermissionFormProps {
  userId: string
  onPermissionUpdated?: () => void
}

const QUICK_DURATION_OPTIONS = [
  { label: '1 Hour', value: 'hour', duration: 1 },
  { label: '4 Hours', value: 'hours_4', duration: 4 },
  { label: '1 Day', value: 'day', duration: 1 },
  { label: '3 Days', value: 'days_3', duration: 3 },
  { label: '1 Week', value: 'week', duration: 1 },
  { label: '2 Weeks', value: 'weeks_2', duration: 2 },
  { label: 'Custom', value: 'custom', duration: 0 },
]

const COMMON_PERMISSIONS = [
  { resource: 'admin', action: 'read', label: 'Admin Read Access' },
  { resource: 'admin', action: 'write', label: 'Admin Write Access' },
  { resource: 'users', action: 'manage', label: 'User Management' },
  { resource: 'reports', action: 'generate', label: 'Generate Reports' },
  { resource: 'settings', action: 'modify', label: 'Modify Settings' },
  { resource: 'billing', action: 'access', label: 'Billing Access' },
]

export function TemporaryPermissionForm({ userId, onPermissionUpdated }: TemporaryPermissionFormProps) {
  const { toast } = useToast()
  const [showForm, setShowForm] = useState(false)
  const [loading, setLoading] = useState(false)
  const [validating, setValidating] = useState(false)
  const [warnings, setWarnings] = useState<string[]>([])
  
  const [formData, setFormData] = useState({
    resource: '',
    action: '',
    duration: 'day',
    customDate: '',
    customTime: '',
    reason: ''
  })

  const calculateExpiration = () => {
    if (formData.duration === 'custom') {
      if (formData.customDate && formData.customTime) {
        return new Date(`${formData.customDate}T${formData.customTime}`)
      }
      return null
    }

    const option = QUICK_DURATION_OPTIONS.find(opt => opt.value === formData.duration)
    if (!option) return null

    const now = new Date()
    if (formData.duration.includes('hour')) {
      return addHours(now, option.duration)
    } else if (formData.duration.includes('week')) {
      return addWeeks(now, option.duration)
    } else {
      return addDays(now, option.duration)
    }
  }

  const validatePermission = async () => {
    if (!formData.resource || !formData.action) return

    try {
      setValidating(true)
      const result = await validatePermissionAssignment({
        userId,
        resource: formData.resource,
        action: formData.action
      })

      if (result.success) {
        setWarnings(result.data?.warnings || [])
      }
    } catch (error) {
      console.error('Validation error:', error)
    } finally {
      setValidating(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!formData.resource || !formData.action) {
      toast({
        title: 'Validation Error',
        description: 'Please specify both resource and action',
        variant: 'destructive'
      })
      return
    }

    const expires = calculateExpiration()
    if (!expires) {
      toast({
        title: 'Validation Error', 
        description: 'Please specify a valid expiration date',
        variant: 'destructive'
      })
      return
    }

    if (expires <= new Date()) {
      toast({
        title: 'Validation Error',
        description: 'Expiration date must be in the future',
        variant: 'destructive'
      })
      return
    }

    try {
      setLoading(true)
      
      const result = await assignTemporaryPermission({
        userId,
        resource: formData.resource,
        action: formData.action,
        expires,
        reason: formData.reason || undefined
      })

      if (result.success) {
        toast({
          title: 'Success',
          description: `Temporary permission granted until ${format(expires, 'PPpp')}`
        })
        
        // Reset form
        setFormData({
          resource: '',
          action: '',
          duration: 'day',
          customDate: '',
          customTime: '',
          reason: ''
        })
        setShowForm(false)
        setWarnings([])
        onPermissionUpdated?.()
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Failed to assign temporary permission',
          variant: 'destructive'
        })
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      })
      console.error('Assign temporary permission error:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleCommonPermissionSelect = (permission: typeof COMMON_PERMISSIONS[0]) => {
    setFormData(prev => ({
      ...prev,
      resource: permission.resource,
      action: permission.action
    }))
    validatePermission()
  }

  if (!showForm) {
    return (
      <div className="space-y-4">
        {/* Quick Access Common Permissions */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
          {COMMON_PERMISSIONS.slice(0, 4).map((permission) => (
            <Button
              key={`${permission.resource}:${permission.action}`}
              variant="outline"
              size="sm"
              onClick={() => {
                handleCommonPermissionSelect(permission)
                setShowForm(true)
              }}
              className="flex items-center gap-2 justify-start"
            >
              <Clock className="h-4 w-4" />
              {permission.label}
            </Button>
          ))}
        </div>
        
        <Button
          onClick={() => setShowForm(true)}
          variant="outline"
          size="sm"
          className="w-full"
        >
          <Plus className="h-4 w-4 mr-2" />
          Add Temporary Permission
        </Button>
      </div>
    )
  }

  const expirationPreview = calculateExpiration()

  return (
    <form onSubmit={handleSubmit} className="space-y-4 border rounded-lg p-4 bg-accent/20">
      <div className="flex items-center justify-between">
        <h4 className="font-medium">Grant Temporary Permission</h4>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => setShowForm(false)}
        >
          Cancel
        </Button>
      </div>

      {/* Permission Selection */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <Label htmlFor="resource">Resource</Label>
          <Input
            id="resource"
            value={formData.resource}
            onChange={(e) => {
              setFormData(prev => ({ ...prev, resource: e.target.value }))
              validatePermission()
            }}
            placeholder="e.g., admin, users, reports"
            required
          />
        </div>
        <div>
          <Label htmlFor="action">Action</Label>
          <Input
            id="action"
            value={formData.action}
            onChange={(e) => {
              setFormData(prev => ({ ...prev, action: e.target.value }))
              validatePermission()
            }}
            placeholder="e.g., read, write, manage"
            required
          />
        </div>
      </div>

      {/* Duration Selection */}
      <div>
        <Label htmlFor="duration">Duration</Label>
        <Select
          value={formData.duration}
          onValueChange={(value) => setFormData(prev => ({ ...prev, duration: value }))}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {QUICK_DURATION_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* Custom Date/Time */}
      {formData.duration === 'custom' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <Label htmlFor="customDate">Expiration Date</Label>
            <Input
              id="customDate"
              type="date"
              value={formData.customDate}
              onChange={(e) => setFormData(prev => ({ ...prev, customDate: e.target.value }))}
              min={format(new Date(), 'yyyy-MM-dd')}
              required
            />
          </div>
          <div>
            <Label htmlFor="customTime">Expiration Time</Label>
            <Input
              id="customTime"
              type="time"
              value={formData.customTime}
              onChange={(e) => setFormData(prev => ({ ...prev, customTime: e.target.value }))}
              required
            />
          </div>
        </div>
      )}

      {/* Expiration Preview */}
      {expirationPreview && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground bg-accent/50 p-2 rounded">
          <Calendar className="h-4 w-4" />
          <span>Expires: {format(expirationPreview, 'PPpp')}</span>
        </div>
      )}

      {/* Validation Warnings */}
      {warnings.length > 0 && (
        <div className="space-y-2">
          {warnings.map((warning, index) => (
            <div key={index} className="flex items-center gap-2 text-sm text-amber-600 bg-amber-50 p-2 rounded">
              <AlertTriangle className="h-4 w-4" />
              {warning}
            </div>
          ))}
        </div>
      )}

      {/* Reason */}
      <div>
        <Label htmlFor="reason">Reason (Optional)</Label>
        <Textarea
          id="reason"
          value={formData.reason}
          onChange={(e) => setFormData(prev => ({ ...prev, reason: e.target.value }))}
          placeholder="Explain why this temporary permission is needed..."
          rows={2}
        />
      </div>

      <div className="flex gap-2">
        <Button 
          type="submit" 
          disabled={loading || validating || !formData.resource || !formData.action}
          className="flex-1"
        >
          {loading ? (
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
          ) : (
            <Clock className="h-4 w-4 mr-2" />
          )}
          Grant Temporary Permission
        </Button>
      </div>
    </form>
  )
}