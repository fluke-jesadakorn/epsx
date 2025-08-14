/**
 * Permission Profile Assignment Inline Form Component
 * Allows assigning/removing permission profiles without modals
 */

'use client'

import { useState } from 'react'
import { Key, Plus, Loader2, X } from 'lucide-react'
import { assignPermissionProfile, removeCustomPermission } from '@/lib/actions/users'

interface PermissionProfileFormProps {
  userId: string
  existingProfileIds: string[]
  onProfileUpdated?: () => void
}

const AVAILABLE_PROFILES = [
  { id: 'basic-trading', name: 'Basic Trading', description: 'Basic market access and trading' },
  { id: 'advanced-analytics', name: 'Advanced Analytics', description: 'Advanced charting and analysis tools' },
  { id: 'premium-features', name: 'Premium Features', description: 'All premium platform features' },
  { id: 'api-access', name: 'API Access', description: 'REST and WebSocket API access' }
]

export function PermissionProfileForm({ userId, existingProfileIds, onProfileUpdated }: PermissionProfileFormProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showAddForm, setShowAddForm] = useState(false)
  const [selectedProfile, setSelectedProfile] = useState('')

  const availableProfiles = AVAILABLE_PROFILES.filter(profile => !existingProfileIds.includes(profile.id))

  const handleAddProfile = async () => {
    if (!selectedProfile) return

    setIsLoading(true)
    setError(null)

    try {
      const result = await assignPermissionProfile({
        userId,
        profileId: selectedProfile,
        assignedBy: 'current-admin' // This should come from auth context
      })

      if (!result.success) {
        setError(result.error?.message || 'Failed to assign profile')
        return
      }

      setSelectedProfile('')
      setShowAddForm(false)
      onProfileUpdated?.()
    } catch (err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  const handleRemoveProfile = async (profileId: string) => {
    setIsLoading(true)
    setError(null)

    try {
      const result = await removeCustomPermission({
        userId,
        permission: `profile:${profileId}`,
        removedBy: 'current-admin' // This should come from auth context
      })

      if (!result.success) {
        setError(result.error?.message || 'Failed to remove profile')
        return
      }

      onProfileUpdated?.()
    } catch (err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 flex items-center gap-2">
          <Key className="h-4 w-4" />
          Permission Profiles
        </h4>
        {availableProfiles.length > 0 && !showAddForm && (
          <button
            onClick={() => setShowAddForm(true)}
            className="text-sm text-blue-600 hover:text-blue-700 flex items-center gap-1"
          >
            <Plus className="h-3 w-3" />
            Add Profile
          </button>
        )}
      </div>

      {error && (
        <div className="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-800 dark:text-red-200 text-sm">
          {error}
        </div>
      )}

      {/* Current Profiles */}
      <div className="space-y-2">
        {existingProfileIds.map(profileId => {
          const profile = AVAILABLE_PROFILES.find(p => p.id === profileId)
          if (!profile) return null

          return (
            <div key={profileId} className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-700 rounded">
              <div>
                <span className="text-sm font-medium">{profile.name}</span>
                <p className="text-xs text-gray-500">{profile.description}</p>
              </div>
              <button
                onClick={() => handleRemoveProfile(profileId)}
                disabled={isLoading}
                className="text-red-600 hover:text-red-700 disabled:opacity-50"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          )
        })}
      </div>

      {/* Add Profile Form */}
      {showAddForm && (
        <div className="p-3 border border-gray-200 dark:border-gray-600 rounded-md space-y-3">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Select Permission Profile
            </label>
            <select
              value={selectedProfile}
              onChange={(e) => setSelectedProfile(e.target.value)}
              className="w-full mt-1 px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Choose a profile...</option>
              {availableProfiles.map(profile => (
                <option key={profile.id} value={profile.id}>
                  {profile.name} - {profile.description}
                </option>
              ))}
            </select>
          </div>
          
          <div className="flex gap-2">
            <button
              onClick={handleAddProfile}
              disabled={!selectedProfile || isLoading}
              className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50 flex items-center gap-1"
            >
              {isLoading && <Loader2 className="h-3 w-3 animate-spin" />}
              Assign
            </button>
            <button
              onClick={() => {
                setShowAddForm(false)
                setSelectedProfile('')
                setError(null)
              }}
              className="px-3 py-1 text-gray-600 dark:text-gray-400 text-sm hover:text-gray-800 dark:hover:text-gray-200"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  )
}