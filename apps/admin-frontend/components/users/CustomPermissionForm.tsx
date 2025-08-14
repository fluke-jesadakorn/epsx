/**
 * Custom Permission Assignment Inline Form Component
 * Allows adding/removing custom permissions without modals
 */

'use client'

import { useState } from 'react'
import { Settings, Plus, Loader2, X } from 'lucide-react'
import { addCustomPermission, removeCustomPermission } from '@/lib/actions/users'

interface CustomPermissionFormProps {
  userId: string
  existingPermissions: string[]
  onPermissionUpdated?: () => void
}

const AVAILABLE_PERMISSIONS = [
  { id: 'admin.users.create', name: 'Create Users', category: 'Admin' },
  { id: 'admin.users.delete', name: 'Delete Users', category: 'Admin' },
  { id: 'admin.system.settings', name: 'System Settings', category: 'Admin' },
  { id: 'trading.advanced.api', name: 'Advanced API Access', category: 'Trading' },
  { id: 'trading.margin.access', name: 'Margin Trading', category: 'Trading' },
  { id: 'analytics.premium.data', name: 'Premium Data Access', category: 'Analytics' },
  { id: 'reports.export.unlimited', name: 'Unlimited Exports', category: 'Reports' }
]

export function CustomPermissionForm({ userId, existingPermissions, onPermissionUpdated }: CustomPermissionFormProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showAddForm, setShowAddForm] = useState(false)
  const [selectedPermission, setSelectedPermission] = useState('')

  const availablePermissions = AVAILABLE_PERMISSIONS.filter(perm => !existingPermissions.includes(perm.id))

  const handleAddPermission = async () => {
    if (!selectedPermission) return

    setIsLoading(true)
    setError(null)

    try {
      const result = await addCustomPermission({
        userId,
        permission: selectedPermission,
        grantedBy: 'current-admin' // This should come from auth context
      })

      if (!result.success) {
        setError(result.error?.message || 'Failed to add permission')
        return
      }

      setSelectedPermission('')
      setShowAddForm(false)
      onPermissionUpdated?.()
    } catch (err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  const handleRemovePermission = async (permission: string) => {
    setIsLoading(true)
    setError(null)

    try {
      const result = await removeCustomPermission({
        userId,
        permission,
        removedBy: 'current-admin' // This should come from auth context
      })

      if (!result.success) {
        setError(result.error?.message || 'Failed to remove permission')
        return
      }

      onPermissionUpdated?.()
    } catch (err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  const getPermissionDisplay = (permissionId: string) => {
    const perm = AVAILABLE_PERMISSIONS.find(p => p.id === permissionId)
    return perm ? `${perm.name} (${perm.category})` : permissionId
  }

  // Group permissions by category for the select
  const groupedPermissions = AVAILABLE_PERMISSIONS.reduce((acc, perm) => {
    if (!availablePermissions.find(ap => ap.id === perm.id)) return acc
    
    if (!acc[perm.category]) {
      acc[perm.category] = []
    }
    acc[perm.category].push(perm)
    return acc
  }, {} as Record<string, typeof AVAILABLE_PERMISSIONS>)

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 flex items-center gap-2">
          <Settings className="h-4 w-4" />
          Custom Permissions
        </h4>
        {availablePermissions.length > 0 && !showAddForm && (
          <button
            onClick={() => setShowAddForm(true)}
            className="text-sm text-blue-600 hover:text-blue-700 flex items-center gap-1"
          >
            <Plus className="h-3 w-3" />
            Add Permission
          </button>
        )}
      </div>

      {error && (
        <div className="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-800 dark:text-red-200 text-sm">
          {error}
        </div>
      )}

      {/* Current Permissions */}
      <div className="space-y-2">
        {existingPermissions.map(permissionId => (
          <div key={permissionId} className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-700 rounded">
            <div>
              <span className="text-sm font-medium">{getPermissionDisplay(permissionId)}</span>
              <p className="text-xs text-gray-500 font-mono">{permissionId}</p>
            </div>
            <button
              onClick={() => handleRemovePermission(permissionId)}
              disabled={isLoading}
              className="text-red-600 hover:text-red-700 disabled:opacity-50"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        ))}
      </div>

      {/* Add Permission Form */}
      {showAddForm && (
        <div className="p-3 border border-gray-200 dark:border-gray-600 rounded-md space-y-3">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Select Permission
            </label>
            <select
              value={selectedPermission}
              onChange={(e) => setSelectedPermission(e.target.value)}
              className="w-full mt-1 px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Choose a permission...</option>
              {Object.entries(groupedPermissions).map(([category, perms]) => (
                <optgroup key={category} label={category}>
                  {perms.map(perm => (
                    <option key={perm.id} value={perm.id}>
                      {perm.name}
                    </option>
                  ))}
                </optgroup>
              ))}
            </select>
          </div>
          
          <div className="flex gap-2">
            <button
              onClick={handleAddPermission}
              disabled={!selectedPermission || isLoading}
              className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50 flex items-center gap-1"
            >
              {isLoading && <Loader2 className="h-3 w-3 animate-spin" />}
              Grant
            </button>
            <button
              onClick={() => {
                setShowAddForm(false)
                setSelectedPermission('')
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