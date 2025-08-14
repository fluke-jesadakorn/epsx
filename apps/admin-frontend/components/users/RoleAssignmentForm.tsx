/**
 * Role Assignment Inline Form Component
 * Allows assigning/removing roles without modals
 */

'use client'

import { useState } from 'react'
import { Shield, Plus, Loader2, X } from 'lucide-react'
import { assignUserRole, removeUserRole } from '@/lib/actions/users'

interface RoleAssignmentFormProps {
  userId: string
  existingRoles: string[]
  onRoleUpdated?: () => void
}

const AVAILABLE_ROLES = [
  { id: 'user-basic-001', name: 'Basic User', description: 'Basic trading features' },
  { id: 'user-premium-002', name: 'Premium User', description: 'Premium features + analytics' },
  { id: 'moderator-standard-003', name: 'Moderator', description: 'User management capabilities' },
  { id: 'admin-full-004', name: 'Admin', description: 'Full system access' }
]

export function RoleAssignmentForm({ userId, existingRoles, onRoleUpdated }: RoleAssignmentFormProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showAddForm, setShowAddForm] = useState(false)
  const [selectedRole, setSelectedRole] = useState('')

  const availableRoles = AVAILABLE_ROLES.filter(role => !existingRoles.includes(role.id))

  const handleAddRole = async () => {
    if (!selectedRole) return

    setIsLoading(true)
    setError(null)

    try {
      const result = await assignUserRole({
        userId,
        roleId: selectedRole,
        assignedBy: 'current-admin' // This should come from auth context
      })

      if (!result.success) {
        setError(result.error?.message || 'Failed to assign role')
        return
      }

      setSelectedRole('')
      setShowAddForm(false)
      onRoleUpdated?.()
    } catch (err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  const handleRemoveRole = async (roleId: string) => {
    setIsLoading(true)
    setError(null)

    try {
      const result = await removeUserRole({
        userId,
        roleId,
        removedBy: 'current-admin' // This should come from auth context
      })

      if (!result.success) {
        setError(result.error?.message || 'Failed to remove role')
        return
      }

      onRoleUpdated?.()
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
          <Shield className="h-4 w-4" />
          Role Assignment
        </h4>
        {availableRoles.length > 0 && !showAddForm && (
          <button
            onClick={() => setShowAddForm(true)}
            className="text-sm text-blue-600 hover:text-blue-700 flex items-center gap-1"
          >
            <Plus className="h-3 w-3" />
            Add Role
          </button>
        )}
      </div>

      {error && (
        <div className="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-800 dark:text-red-200 text-sm">
          {error}
        </div>
      )}

      {/* Current Roles */}
      <div className="space-y-2">
        {existingRoles.map(roleId => {
          const role = AVAILABLE_ROLES.find(r => r.id === roleId)
          if (!role) return null

          return (
            <div key={roleId} className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-700 rounded">
              <div>
                <span className="text-sm font-medium">{role.name}</span>
                <p className="text-xs text-gray-500">{role.description}</p>
              </div>
              <button
                onClick={() => handleRemoveRole(roleId)}
                disabled={isLoading}
                className="text-red-600 hover:text-red-700 disabled:opacity-50"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          )
        })}
      </div>

      {/* Add Role Form */}
      {showAddForm && (
        <div className="p-3 border border-gray-200 dark:border-gray-600 rounded-md space-y-3">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Select Role
            </label>
            <select
              value={selectedRole}
              onChange={(e) => setSelectedRole(e.target.value)}
              className="w-full mt-1 px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Choose a role...</option>
              {availableRoles.map(role => (
                <option key={role.id} value={role.id}>
                  {role.name} - {role.description}
                </option>
              ))}
            </select>
          </div>
          
          <div className="flex gap-2">
            <button
              onClick={handleAddRole}
              disabled={!selectedRole || isLoading}
              className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50 flex items-center gap-1"
            >
              {isLoading && <Loader2 className="h-3 w-3 animate-spin" />}
              Assign
            </button>
            <button
              onClick={() => {
                setShowAddForm(false)
                setSelectedRole('')
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