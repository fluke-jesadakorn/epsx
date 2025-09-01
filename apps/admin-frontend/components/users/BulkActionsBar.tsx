'use client'

import React, { useState, useMemo } from 'react'
import { CheckSquare, X, Shield, UserPlus, Trash2, Check, Settings } from 'lucide-react'

interface BulkActionsBarProps {
  selectedUsers: any[]
  totalUsers: number
  onDeselectAll: () => void
  onBulkGrantPermissions: (userIds: string[]) => void
  onBulkRevokePermissions: (userIds: string[]) => void
  onBulkAssignRoles: (userIds: string[]) => void
  onBulkValidatePermissions: (userIds: string[]) => void
  onBulkApplyTemplate: (userIds: string[]) => void
  isLoading?: boolean
}

/**
 * PancakeSwap x Windows Phone Bulk Actions Bar
 * Floating action bar that appears when users are selected
 */
export default function BulkActionsBar({
  selectedUsers,
  totalUsers,
  onDeselectAll,
  onBulkGrantPermissions,
  onBulkRevokePermissions,
  onBulkAssignRoles,
  onBulkValidatePermissions,
  onBulkApplyTemplate,
  isLoading = false
}: BulkActionsBarProps) {
  const selectedCount = selectedUsers.length
  const selectedUserIds = useMemo(() => selectedUsers.map(u => u.id), [selectedUsers])

  if (selectedCount === 0) return null

  return (
    <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-50 animate-in slide-in-from-bottom-2 duration-300">
      <div className="bg-gradient-to-r from-yellow-400 to-orange-500 text-black px-6 py-4 shadow-2xl border-2 border-yellow-400 backdrop-blur-sm">
        <div className="flex items-center gap-6">
          {/* Selection count */}
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-black/10 rounded-lg flex items-center justify-center">
              <CheckSquare size={16} />
            </div>
            <div>
              <div className="text-sm font-medium">
                {selectedCount.toLocaleString()} selected
              </div>
              <div className="text-xs opacity-75">
                of {totalUsers.toLocaleString()} users
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => onBulkGrantPermissions(selectedUserIds)}
              disabled={isLoading}
              className="px-3 py-2 bg-green-500/20 hover:bg-green-500/30 rounded-lg flex items-center gap-2 text-sm font-medium transition-all disabled:opacity-50 group"
              title="Grant permissions"
            >
              <UserPlus size={14} />
              <span className="hidden sm:inline">Grant</span>
            </button>

            <button
              onClick={() => onBulkRevokePermissions(selectedUserIds)}
              disabled={isLoading}
              className="px-3 py-2 bg-red-500/20 hover:bg-red-500/30 rounded-lg flex items-center gap-2 text-sm font-medium transition-all disabled:opacity-50 group"
              title="Revoke permissions"
            >
              <Trash2 size={14} />
              <span className="hidden sm:inline">Revoke</span>
            </button>

            <button
              onClick={() => onBulkAssignRoles(selectedUserIds)}
              disabled={isLoading}
              className="px-3 py-2 bg-blue-500/20 hover:bg-blue-500/30 rounded-lg flex items-center gap-2 text-sm font-medium transition-all disabled:opacity-50 group"
              title="Assign roles"
            >
              <Shield size={14} />
              <span className="hidden sm:inline">Roles</span>
            </button>

            <button
              onClick={() => onBulkApplyTemplate(selectedUserIds)}
              disabled={isLoading}
              className="px-3 py-2 bg-purple-500/20 hover:bg-purple-500/30 rounded-lg flex items-center gap-2 text-sm font-medium transition-all disabled:opacity-50 group"
              title="Apply template"
            >
              <Settings size={14} />
              <span className="hidden sm:inline">Template</span>
            </button>

            <button
              onClick={() => onBulkValidatePermissions(selectedUserIds)}
              disabled={isLoading}
              className="px-3 py-2 bg-yellow-500/20 hover:bg-yellow-500/30 rounded-lg flex items-center gap-2 text-sm font-medium transition-all disabled:opacity-50 group"
              title="Validate permissions"
            >
              <Check size={14} />
              <span className="hidden sm:inline">Validate</span>
            </button>
          </div>

          {/* Close button */}
          <button
            onClick={onDeselectAll}
            className="w-8 h-8 bg-black/10 hover:bg-black/20 rounded-lg flex items-center justify-center transition-all ml-2"
            title="Clear selection"
          >
            <X size={14} />
          </button>
        </div>

        {isLoading && (
          <div className="mt-2 flex items-center gap-2 text-xs opacity-75">
            <div className="w-3 h-3 border-2 border-black/20 border-t-black/60 rounded-full animate-spin"></div>
            Processing bulk operation...
          </div>
        )}
      </div>
    </div>
  )
}