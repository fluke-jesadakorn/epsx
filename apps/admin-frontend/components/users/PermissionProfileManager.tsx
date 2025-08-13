/**
 * Enhanced Permission Profile Manager for User Permissions Page
 * Provides CRUD operations for managing permission profiles on individual users
 */

'use client'

import { useState, useEffect } from 'react'
import { Shield, Plus, Settings, AlertCircle, CheckCircle, X } from 'lucide-react'
import { Button } from '@epsx/ui'
import { Badge } from '@epsx/ui'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@epsx/ui'
import { useToast } from '@/components/ui/use-toast'
import { 
  assignPermissionProfile, 
  unassignPermissionProfile,
  validatePermissionProfileAssignment,
  getPermissionProfileCategories 
} from '@/lib/actions/permission-profile-actions'
import { listPermissionProfiles } from '@/lib/actions/permission-profile-actions'
import type { PermissionProfile as UnifiedPermissionProfile } from '@/lib/types/unified-user'
import type { PermissionProfile, ValidateAssignmentResponse } from '@/lib/types/permission-profiles'

interface PermissionProfileManagerProps {
  userId: string
  currentProfiles: UnifiedPermissionProfile[]
  canManage: boolean
  onProfileUpdated?: () => void
}

interface AssignmentValidation {
  isValid: boolean
  errors: string[]
  warnings: string[]
  recommendations: string[]
}

export function PermissionProfileManager({
  userId,
  currentProfiles,
  canManage,
  onProfileUpdated
}: PermissionProfileManagerProps) {
  const [availableProfiles, setAvailableProfiles] = useState<PermissionProfile[]>([])
  const [showAssignModal, setShowAssignModal] = useState(false)
  const [showUnassignModal, setShowUnassignModal] = useState(false)
  const [selectedProfile, setSelectedProfile] = useState<string>('')
  const [profileToUnassign, setProfileToUnassign] = useState<UnifiedPermissionProfile | null>(null)
  const [validation, setValidation] = useState<AssignmentValidation | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [categories, setCategories] = useState<Array<{ id: string, name: string }>>([])
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  
  const { toast } = useToast()

  // Load available profiles and categories
  useEffect(() => {
    loadAvailableProfiles()
    loadCategories()
  }, [categoryFilter])

  const loadAvailableProfiles = async () => {
    try {
      const result = await listPermissionProfiles({
        activeOnly: true,
        category: categoryFilter === 'all' ? undefined : categoryFilter
      })

      if (result.success && result.data) {
        // Filter out profiles already assigned to this user
        const currentProfileIds = currentProfiles.map(p => p.id)
        const filtered = result.data.profiles.filter(p => !currentProfileIds.includes(p.id))
        setAvailableProfiles(filtered)
      }
    } catch (error) {
      console.error('Failed to load available profiles:', error)
    }
  }

  const loadCategories = async () => {
    try {
      const result = await getPermissionProfileCategories()
      if (result.success && result.data) {
        setCategories(result.data.categories)
      }
    } catch (error) {
      console.error('Failed to load categories:', error)
    }
  }

  const handleValidateAssignment = async (profileId: string) => {
    if (!profileId) {
      setValidation(null)
      return
    }

    setIsLoading(true)
    try {
      const result = await validatePermissionProfileAssignment({
        userId,
        profileId
      })

      if (result.success && result.data) {
        setValidation({
          isValid: result.data.isValid,
          errors: result.data.errors,
          warnings: result.data.warnings,
          recommendations: result.data.recommendations
        })
      }
    } catch (error) {
      toast({
        title: 'Validation Failed',
        description: 'Could not validate profile assignment',
        variant: 'destructive'
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleAssignProfile = async () => {
    if (!selectedProfile || !validation?.isValid) return

    setIsLoading(true)
    try {
      const result = await assignPermissionProfile({
        userId,
        profileId: selectedProfile,
        assignedBy: 'current-admin'
      })

      if (result.success) {
        toast({
          title: 'Profile Assigned',
          description: 'Permission profile has been successfully assigned'
        })
        setShowAssignModal(false)
        setSelectedProfile('')
        setValidation(null)
        onProfileUpdated?.()
      } else {
        toast({
          title: 'Assignment Failed',
          description: result.error?.message || 'Failed to assign profile',
          variant: 'destructive'
        })
      }
    } catch (error) {
      toast({
        title: 'Assignment Failed',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleUnassignProfile = async () => {
    if (!profileToUnassign) return

    setIsLoading(true)
    try {
      const result = await unassignPermissionProfile({
        userId,
        profileId: profileToUnassign.id,
        reason: 'Manually removed by admin'
      })

      if (result.success) {
        toast({
          title: 'Profile Unassigned',
          description: 'Permission profile has been successfully removed'
        })
        setShowUnassignModal(false)
        setProfileToUnassign(null)
        onProfileUpdated?.()
      } else {
        toast({
          title: 'Removal Failed',
          description: result.error?.message || 'Failed to remove profile',
          variant: 'destructive'
        })
      }
    } catch (error) {
      toast({
        title: 'Removal Failed',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      })
    } finally {
      setIsLoading(false)
    }
  }

  const openUnassignModal = (profile: UnifiedPermissionProfile) => {
    setProfileToUnassign(profile)
    setShowUnassignModal(true)
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Shield className="h-5 w-5 text-purple-500" />
          <h3 className="text-lg font-semibold">Permission Profiles ({currentProfiles.length})</h3>
        </div>
        {canManage && (
          <Button 
            onClick={() => setShowAssignModal(true)} 
            variant="outline" 
            size="sm"
            className="gap-2"
          >
            <Plus className="h-4 w-4" />
            Assign Profile
          </Button>
        )}
      </div>

      {/* Current Profiles */}
      <div className="space-y-3">
        {currentProfiles.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <Shield className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p>No permission profiles assigned</p>
            {canManage && (
              <Button
                variant="link"
                onClick={() => setShowAssignModal(true)}
                className="mt-2"
              >
                Assign your first profile
              </Button>
            )}
          </div>
        ) : (
          currentProfiles.map((profile) => (
            <div
              key={profile.id}
              className="flex items-center justify-between p-4 border rounded-lg bg-white"
            >
              <div className="flex items-center gap-3">
                <Shield className={`h-5 w-5 ${
                  profile.isActive ? 'text-purple-500' : 'text-gray-400'
                }`} />
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{profile.name}</span>
                    <Badge variant={profile.isActive ? 'default' : 'secondary'}>
                      {profile.isActive ? 'Active' : 'Inactive'}
                    </Badge>
                  </div>
                  <p className="text-sm text-muted-foreground">{profile.description}</p>
                  {profile.permissions && (
                    <p className="text-xs text-muted-foreground">
                      {profile.permissions.length} permissions included
                    </p>
                  )}
                </div>
              </div>
              
              {canManage && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => openUnassignModal(profile)}
                  className="text-red-600 hover:text-red-700"
                >
                  <X className="h-4 w-4" />
                </Button>
              )}
            </div>
          ))
        )}
      </div>

      {/* Assign Profile Modal */}
      <Dialog open={showAssignModal} onOpenChange={setShowAssignModal}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-purple-500" />
              Assign Permission Profile
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            {/* Category Filter */}
            <div>
              <label className="text-sm font-medium">Filter by Category</label>
              <Select value={categoryFilter} onValueChange={setCategoryFilter}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Categories</SelectItem>
                  {categories.map(category => (
                    <SelectItem key={category.id} value={category.id}>
                      {category.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Profile Selection */}
            <div>
              <label className="text-sm font-medium">Select Profile</label>
              <Select 
                value={selectedProfile} 
                onValueChange={(value) => {
                  setSelectedProfile(value)
                  handleValidateAssignment(value)
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Choose a permission profile" />
                </SelectTrigger>
                <SelectContent>
                  {availableProfiles.map(profile => (
                    <SelectItem key={profile.id} value={profile.id}>
                      <div>
                        <div className="font-medium">{profile.name}</div>
                        <div className="text-sm text-muted-foreground">
                          {profile.permissions.length} permissions • {profile.category}
                        </div>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Validation Results */}
            {isLoading && (
              <div className="flex items-center gap-2 p-3 bg-blue-50 border border-blue-200 rounded">
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />
                <span className="text-blue-700">Validating assignment...</span>
              </div>
            )}

            {validation && !isLoading && (
              <div className="space-y-2">
                {validation.isValid ? (
                  <div className="flex items-center gap-2 p-3 bg-green-50 border border-green-200 rounded">
                    <CheckCircle className="h-4 w-4 text-green-600" />
                    <span className="text-green-700 font-medium">Assignment is valid</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded">
                    <AlertCircle className="h-4 w-4 text-red-600" />
                    <span className="text-red-700 font-medium">Assignment validation failed</span>
                  </div>
                )}

                {validation.errors.length > 0 && (
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-red-700">Errors:</p>
                    {validation.errors.map((error, index) => (
                      <p key={index} className="text-sm text-red-600">• {error}</p>
                    ))}
                  </div>
                )}

                {validation.warnings.length > 0 && (
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-yellow-700">Warnings:</p>
                    {validation.warnings.map((warning, index) => (
                      <p key={index} className="text-sm text-yellow-600">• {warning}</p>
                    ))}
                  </div>
                )}

                {validation.recommendations.length > 0 && (
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-blue-700">Recommendations:</p>
                    {validation.recommendations.map((rec, index) => (
                      <p key={index} className="text-sm text-blue-600">• {rec}</p>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAssignModal(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleAssignProfile}
              disabled={!selectedProfile || !validation?.isValid || isLoading}
              className="gap-2"
            >
              {isLoading && <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />}
              Assign Profile
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Unassign Profile Modal */}
      <Dialog open={showUnassignModal} onOpenChange={setShowUnassignModal}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertCircle className="h-5 w-5 text-red-500" />
              Remove Permission Profile
            </DialogTitle>
          </DialogHeader>

          {profileToUnassign && (
            <div className="space-y-4">
              <p>Are you sure you want to remove the following permission profile?</p>
              
              <div className="p-3 border rounded-lg bg-gray-50">
                <div className="font-medium">{profileToUnassign.name}</div>
                <div className="text-sm text-muted-foreground">{profileToUnassign.description}</div>
              </div>

              <div className="text-sm text-muted-foreground bg-yellow-50 border border-yellow-200 p-3 rounded">
                <strong>Note:</strong> This will remove all permissions granted by this profile. 
                The user may lose access to certain features.
              </div>
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUnassignModal(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleUnassignProfile}
              disabled={isLoading}
              className="gap-2"
            >
              {isLoading && <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />}
              Remove Profile
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}