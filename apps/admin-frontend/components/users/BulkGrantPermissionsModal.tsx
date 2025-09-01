'use client'

import React, { useState, useTransition } from 'react'
import { X, UserPlus, Check, AlertTriangle, Plus, Trash2 } from 'lucide-react'

interface BulkGrantPermissionsModalProps {
  isOpen: boolean
  onClose: () => void
  selectedUsers: any[]
  onConfirm: (data: { userIds: string[], permissions: string[], reason?: string }) => Promise<void>
}

/**
 * PancakeSwap x Windows Phone Bulk Grant Permissions Modal
 * Modal for granting permissions to multiple users
 */
export default function BulkGrantPermissionsModal({
  isOpen,
  onClose,
  selectedUsers,
  onConfirm
}: BulkGrantPermissionsModalProps) {
  const [isPending, startTransition] = useTransition()
  const [permissions, setPermissions] = useState<string[]>([''])
  const [reason, setReason] = useState('')
  const [showAdvanced, setShowAdvanced] = useState(false)
  const [validationErrors, setValidationErrors] = useState<string[]>([])

  if (!isOpen) return null

  const selectedCount = selectedUsers.length
  const userIds = selectedUsers.map(u => u.id)

  // Predefined permission templates
  const permissionTemplates = [
    {
      name: 'Basic User',
      permissions: ['epsx:analytics:view', 'epsx:profile:manage']
    },
    {
      name: 'Premium User', 
      permissions: ['epsx:analytics:view', 'epsx:analytics:export:basic', 'epsx:profile:manage', 'epsx:realtime:access']
    },
    {
      name: 'Moderator',
      permissions: ['admin:users:view', 'admin:users:edit', 'epsx:analytics:view', 'epsx:moderation:access']
    },
    {
      name: 'Analytics User',
      permissions: ['epsx:analytics:view', 'epsx:analytics:export:advanced', 'epsx:analytics:rankings:manage', 'epsx:realtime:access']
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

  const applyTemplate = (template: typeof permissionTemplates[0]) => {
    setPermissions(template.permissions)
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
        console.error('Bulk grant permissions failed:', error)
      }
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      
      <div className="relative bg-white dark:bg-gray-900 w-full max-w-2xl mx-4 shadow-2xl border-2 border-yellow-400 max-h-[90vh] overflow-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700 bg-gradient-to-r from-yellow-400 to-orange-500">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-black/10 rounded-lg flex items-center justify-center">
              <UserPlus size={18} className="text-black" />
            </div>
            <div className="text-black">
              <h2 className="text-xl font-medium">Grant Permissions</h2>
              <p className="text-sm opacity-75">Add permissions to {selectedCount} users</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 bg-black/10 hover:bg-black/20 rounded-lg flex items-center justify-center transition-all"
          >
            <X size={16} className="text-black" />
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
                    <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                    {user.email}
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

          {/* Permission Templates */}
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Quick Templates</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {permissionTemplates.map((template, index) => (
                <button
                  key={index}
                  onClick={() => applyTemplate(template)}
                  className="p-3 text-left bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-800 dark:to-gray-700 hover:from-yellow-50 hover:to-orange-50 dark:hover:from-gray-700 dark:hover:to-gray-600 border border-gray-200 dark:border-gray-600 rounded-lg transition-all"
                >
                  <div className="font-medium text-gray-900 dark:text-white text-sm">{template.name}</div>
                  <div className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                    {template.permissions.length} permissions
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Custom Permissions */}
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Permissions to Grant</h3>
            <div className="space-y-3">
              {permissions.map((permission, index) => (
                <div key={index} className="flex items-center gap-3">
                  <input
                    type="text"
                    value={permission}
                    onChange={(e) => handlePermissionChange(index, e.target.value)}
                    placeholder="e.g., epsx:analytics:view"
                    className="flex-1 px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:border-yellow-400 focus:outline-none"
                  />
                  {permissions.length > 1 && (
                    <button
                      onClick={() => handleRemovePermission(index)}
                      className="w-10 h-10 bg-red-100 dark:bg-red-900/20 hover:bg-red-200 dark:hover:bg-red-900/40 rounded-lg flex items-center justify-center transition-all"
                    >
                      <Trash2 size={14} className="text-red-600 dark:text-red-400" />
                    </button>
                  )}
                </div>
              ))}
              
              <button
                onClick={handleAddPermission}
                className="flex items-center gap-2 px-4 py-2 text-yellow-600 dark:text-yellow-400 hover:text-yellow-700 dark:hover:text-yellow-300 transition-all"
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
                    placeholder="Why are you granting these permissions?"
                    rows={3}
                    className="w-full px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:border-yellow-400 focus:outline-none"
                  />
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            This will grant permissions to {selectedCount} users
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
              className="flex items-center gap-2 px-6 py-2 bg-gradient-to-r from-yellow-400 to-orange-500 text-black font-medium hover:from-yellow-500 hover:to-orange-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isPending ? (
                <>
                  <div className="w-4 h-4 border-2 border-black/20 border-t-black/60 rounded-full animate-spin"></div>
                  Granting...
                </>
              ) : (
                <>
                  <Check size={16} />
                  Grant Permissions
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}