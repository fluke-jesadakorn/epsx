'use client'

import React, { useState, useTransition } from 'react'
import { X, Trash2, Check, AlertTriangle, Plus, Minus } from 'lucide-react'

interface BulkRevokePermissionsModalProps {
  isOpen: boolean
  onClose: () => void
  selectedUsers: any[]
  onConfirm: (data: { userIds: string[], permissions: string[], reason?: string }) => Promise<void>
}

/**
 * PancakeSwap x Windows Phone Bulk Revoke Permissions Modal
 * Modal for revoking permissions from multiple users
 */
export default function BulkRevokePermissionsModal({
  isOpen,
  onClose,
  selectedUsers,
  onConfirm
}: BulkRevokePermissionsModalProps) {
  const [isPending, startTransition] = useTransition()
  const [permissions, setPermissions] = useState<string[]>([''])
  const [reason, setReason] = useState('')
  const [showAdvanced, setShowAdvanced] = useState(false)
  const [validationErrors, setValidationErrors] = useState<string[]>([])

  if (!isOpen) return null

  const selectedCount = selectedUsers.length
  const userIds = selectedUsers.map(u => u.id)

  // Get common permissions across selected users
  const commonPermissions = selectedUsers.length > 0 ? 
    selectedUsers[0].permissions?.filter((perm: string) => 
      selectedUsers.every(user => user.permissions?.includes(perm))
    ) || [] : []

  // Common permission categories for quick removal
  const permissionCategories = [
    {
      name: 'Admin Access',
      permissions: ['admin:*:*', 'admin:users:manage', 'admin:system:configure'],
      description: 'Remove all admin privileges'
    },
    {
      name: 'Analytics Export',
      permissions: ['epsx:analytics:export:basic', 'epsx:analytics:export:advanced'],
      description: 'Remove data export capabilities'
    },
    {
      name: 'User Management',
      permissions: ['admin:users:view', 'admin:users:edit', 'admin:users:delete'],
      description: 'Remove user management permissions'
    },
    {
      name: 'Real-time Access',
      permissions: ['epsx:realtime:access', 'epsx:websocket:connect'],
      description: 'Remove real-time data access'
    }
  ]

  const handleAddPermission = () => {
    setPermissions([...permissions, ''])
  }

  const handleRemovePermission = (index: number) => {
    if (permissions.length > 1) {
      setPermissions(permissions.filter((_, i) => i !== index))
    }
  }

  const handlePermissionChange = (index: number, value: string) => {
    const newPermissions = [...permissions]
    newPermissions[index] = value
    setPermissions(newPermissions)
    
    // Clear validation errors when user starts typing
    setValidationErrors([])
  }

  const applyPermissionCategory = (category: typeof permissionCategories[0]) => {
    setPermissions(category.permissions)
    setValidationErrors([])
  }

  const setCommonPermissions = () => {
    setPermissions(commonPermissions)
    setValidationErrors([])
  }

  const validatePermissions = () => {
    const errors: string[] = []
    const validPermissions = permissions.filter(p => p.trim())
    
    if (validPermissions.length === 0) {
      errors.push('At least one permission is required')
    }
    
    validPermissions.forEach((perm, index) => {
      if (!perm.includes(':')) {
        errors.push(`Permission ${index + 1}: Invalid format (should be platform:resource:action)`)
      }
    })
    
    // Check for duplicates
    const duplicates = validPermissions.filter((perm, index) => 
      validPermissions.indexOf(perm) !== index
    )
    if (duplicates.length > 0) {
      errors.push('Duplicate permissions found')
    }
    
    setValidationErrors(errors)
    return errors.length === 0
  }

  const handleConfirm = async () => {
    if (!validatePermissions()) return

    const validPermissions = permissions.filter(p => p.trim())
    
    startTransition(async () => {
      try {
        await onConfirm({
          userIds,
          permissions: validPermissions,
          reason: reason.trim() || undefined
        })
        
        // Reset form
        setPermissions([''])
        setReason('')
        setValidationErrors([])
        onClose()
      } catch (error) {
        console.error('Bulk revoke permissions failed:', error)
      }
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      
      <div className="relative bg-white dark:bg-gray-900 w-full max-w-2xl mx-4 shadow-2xl border-2 border-red-400 max-h-[90vh] overflow-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700 bg-gradient-to-r from-red-500 to-red-700">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-white/10 rounded-lg flex items-center justify-center">
              <Trash2 size={18} className="text-white" />
            </div>
            <div className="text-white">
              <h2 className="text-xl font-medium">Revoke Permissions</h2>
              <p className="text-sm opacity-75">Remove permissions from {selectedCount} users</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 bg-white/10 hover:bg-white/20 rounded-lg flex items-center justify-center transition-all"
          >
            <X size={16} className="text-white" />
          </button>
        </div>

        <div className="p-6 space-y-6">
          {/* Selected users preview */}
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Selected Users</h3>
            <div className="max-h-32 overflow-auto bg-gray-50 dark:bg-gray-800 p-3 rounded-lg">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {selectedUsers.slice(0, 10).map(user => (
                  <div key={user.id} className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-2">
                    <div className="w-2 h-2 bg-red-400 rounded-full"></div>
                    {user.email}
                    <span className="text-xs opacity-75">
                      ({user.permissions?.length || 0} perms)
                    </span>
                  </div>
                ))}
                {selectedUsers.length > 10 && (
                  <div className="text-sm text-gray-500 dark:text-gray-500 col-span-2">
                    + {selectedUsers.length - 10} more users...
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Common Permissions */}
          {commonPermissions.length > 0 && (
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                Common Permissions
                <span className="text-sm text-gray-500 dark:text-gray-400 font-normal ml-2">
                  (shared by all selected users)
                </span>
              </h3>
              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <span className="text-sm text-blue-900 dark:text-blue-200">
                    {commonPermissions.length} permissions shared by all users
                  </span>
                  <button
                    onClick={setCommonPermissions}
                    className="text-sm px-3 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-all"
                  >
                    Revoke These
                  </button>
                </div>
                <div className="text-xs text-blue-700 dark:text-blue-300 space-y-1 max-h-20 overflow-auto">
                  {commonPermissions.map((perm, index) => (
                    <div key={index}>{perm}</div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Permission Categories */}
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Quick Remove Categories</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {permissionCategories.map((category, index) => (
                <button
                  key={index}
                  onClick={() => applyPermissionCategory(category)}
                  className="p-3 text-left bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-800 dark:to-gray-700 hover:from-red-50 hover:to-red-100 dark:hover:from-red-900/20 dark:hover:to-red-800/20 border border-gray-200 dark:border-gray-600 rounded-lg transition-all"
                >
                  <div className="font-medium text-gray-900 dark:text-white text-sm">{category.name}</div>
                  <div className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                    {category.description}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Custom Permissions */}
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Permissions to Revoke</h3>
            <div className="space-y-3">
              {permissions.map((permission, index) => (
                <div key={index} className="flex items-center gap-3">
                  <input
                    type="text"
                    value={permission}
                    onChange={(e) => handlePermissionChange(index, e.target.value)}
                    placeholder="e.g., admin:users:manage"
                    className="flex-1 px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:border-red-400 focus:outline-none"
                  />
                  {permissions.length > 1 && (
                    <button
                      onClick={() => handleRemovePermission(index)}
                      className="w-10 h-10 bg-red-100 dark:bg-red-900/20 hover:bg-red-200 dark:hover:bg-red-900/40 rounded-lg flex items-center justify-center transition-all"
                    >
                      <Minus size={14} className="text-red-600 dark:text-red-400" />
                    </button>
                  )}
                </div>
              ))}
              
              <button
                onClick={handleAddPermission}
                className="flex items-center gap-2 px-4 py-2 text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300 transition-all"
              >
                <Plus size={16} />
                Add another permission
              </button>
            </div>
          </div>

          {/* Validation Errors */}
          {validationErrors.length > 0 && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <div className="flex items-start gap-3">
                <AlertTriangle size={16} className="text-red-600 dark:text-red-400 mt-0.5" />
                <div>
                  <h4 className="text-sm font-medium text-red-800 dark:text-red-200">Validation Errors</h4>
                  <ul className="mt-2 text-sm text-red-700 dark:text-red-300 list-disc list-inside space-y-1">
                    {validationErrors.map((error, index) => (
                      <li key={index}>{error}</li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          )}

          {/* Advanced Options */}
          <div>
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-all"
            >
              {showAdvanced ? '▼' : '▶'} Advanced Options
            </button>
            
            {showAdvanced && (
              <div className="mt-4 space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-900 dark:text-white mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="Why are you revoking these permissions?"
                    rows={3}
                    className="w-full px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:border-red-400 focus:outline-none"
                  />
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            ⚠️ This will permanently remove permissions from {selectedCount} users
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-all"
              disabled={isPending}
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={isPending || permissions.filter(p => p.trim()).length === 0}
              className="flex items-center gap-2 px-6 py-2 bg-gradient-to-r from-red-500 to-red-700 text-white font-medium hover:from-red-600 hover:to-red-800 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isPending ? (
                <>
                  <div className="w-4 h-4 border-2 border-white/20 border-t-white/60 rounded-full animate-spin"></div>
                  Revoking...
                </>
              ) : (
                <>
                  <Check size={16} />
                  Revoke Permissions
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}