/**
 * Permission Profile Assignment Modal Component
 * Allows assigning permission profiles to users
 */

'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
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
import { assignPermissionProfile } from '@/lib/actions/unified-user-actions'

interface PermissionProfileModalProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  userId: string
  userEmail: string
  existingProfiles: string[]
  onProfileAssigned?: () => void
}

const AVAILABLE_PROFILES = [
  {
    id: 'user-basic-001',
    name: 'Basic User',
    description: 'Basic trading features and market data access',
    permissions: ['market-data-read', 'portfolio-view', 'basic-alerts']
  },
  {
    id: 'user-premium-002', 
    name: 'Premium User',
    description: 'Premium features with advanced analytics and tools',
    permissions: ['market-data-read', 'portfolio-manage', 'advanced-analytics', 'premium-alerts', 'export-data']
  },
  {
    id: 'moderator-standard-003',
    name: 'Moderator',
    description: 'User management capabilities and content moderation',
    permissions: ['user-view', 'user-manage', 'content-moderate', 'reports-view']
  },
  {
    id: 'admin-full-004',
    name: 'Administrator',
    description: 'Full system access and administrative privileges',
    permissions: ['admin-full-access', 'user-manage', 'system-settings', 'audit-logs', 'billing-manage']
  }
]

export function PermissionProfileModal({
  isOpen,
  onOpenChange,
  userId,
  userEmail,
  existingProfiles,
  onProfileAssigned
}: PermissionProfileModalProps) {
  const [selectedProfile, setSelectedProfile] = useState('')
  const [reason, setReason] = useState('')
  const [isAssigning, setIsAssigning] = useState(false)

  // Get available profiles (exclude existing ones)
  const availableProfiles = AVAILABLE_PROFILES.filter(profile => 
    !existingProfiles.includes(profile.id)
  )

  // Reset form when modal opens/closes
  useEffect(() => {
    if (!isOpen) {
      setSelectedProfile('')
      setReason('')
    }
  }, [isOpen])

  const handleAssignProfile = async () => {
    if (!selectedProfile) {
      toast.error('Please select a permission profile to assign')
      return
    }

    setIsAssigning(true)
    try {
      const result = await assignPermissionProfile({
        userId,
        profileId: selectedProfile,
        reason: reason || `Permission profile assigned to ${userEmail}`
      })
      
      if (result.success) {
        const profileName = AVAILABLE_PROFILES.find(p => p.id === selectedProfile)?.name
        toast.success(`Permission profile "${profileName}" assigned successfully`)
        onProfileAssigned?.()
        onOpenChange(false)
      } else {
        toast.error(result.error?.message || 'Failed to assign permission profile')
      }
    } catch (error) {
      toast.error('Failed to assign permission profile')
    } finally {
      setIsAssigning(false)
    }
  }

  const selectedProfileDetails = AVAILABLE_PROFILES.find(p => p.id === selectedProfile)

  if (availableProfiles.length === 0) {
    return (
      <Dialog open={isOpen} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Assign Permission Profile</DialogTitle>
            <DialogDescription>
              All available permission profiles have already been assigned to {userEmail}.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    )
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Assign Permission Profile</DialogTitle>
          <DialogDescription>
            Assign a predefined permission profile to {userEmail}. This will grant a comprehensive set of permissions based on the user's role.
          </DialogDescription>
        </DialogHeader>
        
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="profile">Permission Profile</Label>
            <Select value={selectedProfile} onValueChange={setSelectedProfile}>
              <SelectTrigger>
                <SelectValue placeholder="Select a permission profile" />
              </SelectTrigger>
              <SelectContent>
                {availableProfiles.map((profile) => (
                  <SelectItem key={profile.id} value={profile.id}>
                    <div className="flex flex-col">
                      <span className="font-medium">{profile.name}</span>
                      <span className="text-xs text-muted-foreground">{profile.description}</span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedProfileDetails && (
            <div className="rounded-md bg-muted p-3">
              <h4 className="text-sm font-medium mb-2">Included Permissions:</h4>
              <div className="flex flex-wrap gap-1">
                {selectedProfileDetails.permissions.map((permission) => (
                  <Badge key={permission} variant="outline" className="text-xs">
                    {permission}
                  </Badge>
                ))}
              </div>
            </div>
          )}
          
          <div className="grid gap-2">
            <Label htmlFor="reason">Reason (Optional)</Label>
            <Textarea
              id="reason"
              placeholder="Enter reason for profile assignment..."
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="min-h-[80px]"
            />
          </div>

          {existingProfiles.length > 0 && (
            <div className="rounded-md bg-muted p-3">
              <h4 className="text-sm font-medium mb-2">Current Profiles:</h4>
              <div className="flex flex-wrap gap-1">
                {existingProfiles.map((profileId) => {
                  const profileInfo = AVAILABLE_PROFILES.find(p => p.id === profileId)
                  return (
                    <Badge 
                      key={profileId}
                      variant="secondary" 
                      className="text-xs"
                    >
                      {profileInfo?.name || profileId}
                    </Badge>
                  )
                })}
              </div>
            </div>
          )}
        </div>
        
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button 
            onClick={handleAssignProfile}
            disabled={!selectedProfile || isAssigning}
            className="min-w-[100px]"
          >
            {isAssigning ? 'Assigning...' : 'Assign Profile'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}