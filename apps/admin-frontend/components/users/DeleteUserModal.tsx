'use client'

import React, { useState } from 'react'
import { X, Trash2, AlertTriangle, Check, AlertCircle } from 'lucide-react'
import { ClientUserAPI } from '@/lib/api/client-admin-api'

/**
 * PancakeSwap x Windows Phone Delete User Modal
 * Modal for deleting users with confirmation and backend integration
 * Performs soft delete as per backend implementation
 */

interface DeleteUserModalProps {
  isOpen: boolean
  onClose: () => void
  onUserDeleted: () => void
  user: any // User data to delete
}

export default function DeleteUserModal({ isOpen, onClose, onUserDeleted, user }: DeleteUserModalProps) {
  const [isDeleting, setIsDeleting] = useState(false)
  const [deleteStatus, setDeleteStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [errorMessage, setErrorMessage] = useState('')

  const resetState = () => {
    setDeleteStatus('idle')
    setErrorMessage('')
    setIsDeleting(false)
  }

  const handleClose = () => {
    resetState()
    onClose()
  }

  const handleDelete = async () => {
    if (!user?.id) {
      setErrorMessage('User ID is required for deletion')
      return
    }

    setIsDeleting(true)
    setErrorMessage('')

    try {
      const result = await ClientUserAPI.deleteUser(user.id)

      console.log('✅ User deleted successfully:', result)
      setDeleteStatus('success')
      
      // Close modal after success and refresh the user list
      setTimeout(() => {
        handleClose()
        onUserDeleted() // Refresh the parent component
      }, 1500)

    } catch (error) {
      console.error('❌ Failed to delete user:', error)
      setDeleteStatus('error')
      setErrorMessage(error instanceof Error ? error.message : 'Failed to delete user')
    } finally {
      setIsDeleting(false)
    }
  }

  if (!isOpen || !user) return null

  const hasAdminPerms = user.permissions?.some((p: string) => p.startsWith('admin:')) || false

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50">
      <div className="bg-white dark:bg-gray-900 w-full max-w-lg shadow-xl border-2 border-red-400/20">
        {/* PancakeSwap x Windows Phone header */}
        <div className="bg-gradient-to-r from-red-500 to-red-700 p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 bg-black rounded-lg flex items-center justify-center">
                <Trash2 size={16} className="text-red-400" />
              </div>
              <div>
                <h2 className="text-lg font-extralight tracking-wide text-white">delete user</h2>
                <p className="text-white/70 text-xs font-light">permanent removal</p>
              </div>
            </div>
            <button 
              onClick={handleClose}
              className="w-8 h-8 bg-black/10 hover:bg-black/20 flex items-center justify-center transition-all"
            >
              <X size={16} className="text-white" />
            </button>
          </div>
        </div>

        <div className="p-6 space-y-6">
          {/* Warning Message */}
          <div className="flex items-start gap-3 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-600 dark:text-red-400">
            <AlertTriangle size={20} className="flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-medium text-sm mb-2">Are you sure you want to delete this user?</h3>
              <p className="text-sm font-light opacity-90">
                This action will soft-delete the user and remove their access to the system. 
                This action can be reversed by system administrators.
              </p>
            </div>
          </div>

          {/* User Information */}
          <div className="bg-gray-50 dark:bg-gray-800 p-4 border border-gray-200 dark:border-gray-700">
            <h4 className="text-sm font-light text-gray-700 dark:text-gray-300 mb-3 uppercase tracking-wider">
              User Details
            </h4>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600 dark:text-gray-400">Email:</span>
                <span className="text-sm font-medium text-gray-900 dark:text-white">{user.email}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600 dark:text-gray-400">Role:</span>
                <span className={`text-sm font-medium ${hasAdminPerms ? 'text-red-600' : 'text-blue-600'}`}>
                  {hasAdminPerms ? '🔑 admin' : '👤 user'}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-600 dark:text-gray-400">Permissions:</span>
                <span className="text-sm font-medium text-gray-900 dark:text-white">
                  {user.permissions?.length || 0} granted
                </span>
              </div>
            </div>
          </div>

          {/* Additional Warning for Admin Users */}
          {hasAdminPerms && (
            <div className="flex items-start gap-3 p-4 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 text-amber-700 dark:text-amber-400">
              <AlertTriangle size={16} className="flex-shrink-0 mt-0.5" />
              <div>
                <p className="text-sm font-light">
                  <strong>Warning:</strong> This user has admin privileges. Deleting this user may impact system administration capabilities.
                </p>
              </div>
            </div>
          )}

          {/* Error Message */}
          {errorMessage && (
            <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-600 dark:text-red-400">
              <AlertCircle size={16} />
              <span className="text-sm font-light">{errorMessage}</span>
            </div>
          )}

          {/* Success Message */}
          {deleteStatus === 'success' && (
            <div className="flex items-center gap-2 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-green-600 dark:text-green-400">
              <Check size={16} />
              <span className="text-sm font-light">User deleted successfully!</span>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={handleClose}
              disabled={isDeleting}
              className="flex-1 px-4 py-3 bg-gray-200 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-700 transition-all font-light disabled:opacity-50"
            >
              cancel
            </button>
            <button
              type="button"
              onClick={handleDelete}
              disabled={isDeleting}
              className="flex-1 px-4 py-3 bg-gradient-to-r from-red-500 to-red-700 text-white font-medium hover:from-red-600 hover:to-red-800 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isDeleting ? 'deleting...' : 'delete user'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}