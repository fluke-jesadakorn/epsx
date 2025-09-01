'use client'

import React, { useState, useTransition } from 'react'
import { X, Shield, Check, Users, Crown, Star, Settings } from 'lucide-react'

interface BulkAssignRolesModalProps {
  isOpen: boolean
  onClose: () => void
  selectedUsers: any[]
  onConfirm: (data: { userIds: string[], role: string, mergePermissions: boolean, reason?: string }) => Promise<void>
}

/**
 * PancakeSwap x Windows Phone Bulk Assign Roles Modal
 * Modal for assigning roles to multiple users
 */
export default function BulkAssignRolesModal({
  isOpen,
  onClose,
  selectedUsers,
  onConfirm
}: BulkAssignRolesModalProps) {
  const [isPending, startTransition] = useTransition()
  const [selectedRole, setSelectedRole] = useState('')
  const [mergePermissions, setMergePermissions] = useState(true)
  const [reason, setReason] = useState('')
  const [showAdvanced, setShowAdvanced] = useState(false)

  if (!isOpen) return null

  const selectedCount = selectedUsers.length
  const userIds = selectedUsers.map(u => u.id)

  // Available roles with descriptions
  const availableRoles = [
    {
      id: 'admin',
      name: 'Administrator',
      description: 'Full system access with all permissions',
      permissions: ['admin:*:*'],
      icon: Crown,
      color: 'from-red-500 to-red-700',
      count: selectedUsers.filter(u => u.permissions?.some((p: string) => p.startsWith('admin:'))).length
    },
    {
      id: 'user',
      name: 'Standard User',
      description: 'Basic user with analytics access',
      permissions: ['epsx:analytics:view', 'epsx:profile:manage'],
      icon: Users,
      color: 'from-blue-500 to-blue-700',
      count: selectedUsers.filter(u => !u.permissions?.some((p: string) => p.startsWith('admin:')) && u.subscription_tier !== 'premium').length
    },
    {
      id: 'premium',
      name: 'Premium User',
      description: 'Enhanced features with export capabilities',
      permissions: ['epsx:analytics:view', 'epsx:analytics:export:basic', 'epsx:profile:manage', 'epsx:realtime:access'],
      icon: Star,
      color: 'from-yellow-500 to-orange-500',
      count: selectedUsers.filter(u => u.subscription_tier === 'premium').length
    },
    {
      id: 'guest',
      name: 'Guest',
      description: 'Limited read-only access',
      permissions: ['epsx:analytics:view'],
      icon: Settings,
      color: 'from-gray-500 to-gray-700',
      count: 0
    }
  ]

  const selectedRoleData = availableRoles.find(r => r.id === selectedRole)

  const handleConfirm = async () => {
    if (!selectedRole) return

    startTransition(async () => {
      try {
        await onConfirm({
          userIds,
          role: selectedRole,
          mergePermissions,
          reason: reason.trim() || undefined
        })
        
        // Reset form
        setSelectedRole('')
        setMergePermissions(true)
        setReason('')
        onClose()
      } catch (error) {
        console.error('Bulk assign roles failed:', error)
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
              <Shield size={18} className="text-black" />
            </div>
            <div className="text-black">
              <h2 className="text-xl font-medium">Assign Roles</h2>
              <p className="text-sm opacity-75">Change role for {selectedCount} users</p>
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
                {selectedUsers.slice(0, 10).map(user => {
                  const hasAdmin = user.permissions?.some((p: string) => p.startsWith('admin:'))
                  const isPremium = user.subscription_tier === 'premium'
                  return (
                    <div key={user.id} className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${hasAdmin ? 'bg-red-400' : isPremium ? 'bg-yellow-400' : 'bg-blue-400'}`}></div>
                      {user.email}
                      <span className="text-xs opacity-75">
                        ({hasAdmin ? 'admin' : isPremium ? 'premium' : 'user'})
                      </span>
                    </div>
                  )
                })}
                {selectedUsers.length > 10 && (
                  <div className="text-sm text-gray-500 dark:text-gray-500 col-span-2">
                    + {selectedUsers.length - 10} more users...
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Role Selection */}
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-3">Select Role</h3>
            <div className="space-y-3">
              {availableRoles.map((role) => {
                const Icon = role.icon
                const isSelected = selectedRole === role.id
                return (
                  <button
                    key={role.id}
                    onClick={() => setSelectedRole(role.id)}
                    className={`w-full p-4 text-left border-2 rounded-lg transition-all ${
                      isSelected 
                        ? 'border-yellow-400 bg-yellow-50 dark:bg-yellow-900/20' 
                        : 'border-gray-200 dark:border-gray-700 hover:border-yellow-400/50'
                    }`}
                  >
                    <div className="flex items-start gap-4">
                      <div className={`w-12 h-12 bg-gradient-to-br ${role.color} rounded-lg flex items-center justify-center text-white`}>
                        <Icon size={20} />
                      </div>
                      <div className="flex-1">
                        <div className="flex items-center justify-between mb-1">
                          <h4 className="font-medium text-gray-900 dark:text-white">{role.name}</h4>
                          {role.count > 0 && (
                            <span className="text-xs bg-gray-200 dark:bg-gray-700 px-2 py-1 rounded">
                              {role.count} currently have this role
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">{role.description}</p>
                        <div className="text-xs text-gray-500 dark:text-gray-500">
                          Permissions: {role.permissions.join(', ')}
                        </div>
                      </div>
                    </div>
                  </button>
                )
              })}
            </div>
          </div>

          {/* Permission Merge Option */}
          {selectedRole && (
            <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
              <h4 className="text-sm font-medium text-blue-900 dark:text-blue-200 mb-3">Permission Handling</h4>
              <div className="space-y-2">
                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="radio"
                    name="permissionMode"
                    checked={mergePermissions}
                    onChange={() => setMergePermissions(true)}
                    className="mt-1"
                  />
                  <div>
                    <div className="text-sm font-medium text-blue-900 dark:text-blue-200">
                      Merge with existing permissions (Recommended)
                    </div>
                    <div className="text-xs text-blue-700 dark:text-blue-300">
                      Add new role permissions to users' existing permissions
                    </div>
                  </div>
                </label>
                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="radio"
                    name="permissionMode"
                    checked={!mergePermissions}
                    onChange={() => setMergePermissions(false)}
                    className="mt-1"
                  />
                  <div>
                    <div className="text-sm font-medium text-blue-900 dark:text-blue-200">
                      Replace all permissions
                    </div>
                    <div className="text-xs text-blue-700 dark:text-blue-300">
                      Remove all existing permissions and set only the role permissions
                    </div>
                  </div>
                </label>
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
              <div className="mt-4">
                <div>
                  <label className="block text-sm font-medium text-gray-900 dark:text-white mb-2">
                    Reason (optional)
                  </label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="Why are you changing these user roles?"
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
            {selectedRoleData && (
              <div>
                Assigning <span className="font-medium">{selectedRoleData.name}</span> role to {selectedCount} users
                {mergePermissions ? ' (merging permissions)' : ' (replacing permissions)'}
              </div>
            )}
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
              disabled={isPending || !selectedRole}
              className="flex items-center gap-2 px-6 py-2 bg-gradient-to-r from-yellow-400 to-orange-500 text-black font-medium hover:from-yellow-500 hover:to-orange-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isPending ? (
                <>
                  <div className="w-4 h-4 border-2 border-black/20 border-t-black/60 rounded-full animate-spin"></div>
                  Assigning...
                </>
              ) : (
                <>
                  <Check size={16} />
                  Assign Role
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}