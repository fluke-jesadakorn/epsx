'use client'

import React, { useState } from 'react'
import { X, User, Shield, Mail, Check, AlertCircle, Plus } from 'lucide-react'
import { ClientUserAPI } from '@/lib/api/client-admin-api'

/**
 * PancakeSwap x Windows Phone Create User Modal
 * Modern modal with DeFi aesthetics for creating new users with backend integration
 */

interface CreateUserModalProps {
  isOpen: boolean
  onClose: () => void
  onUserCreated: () => void
}

interface FormData {
  email: string
  displayName: string
  permissions: string[]
}

const PERMISSION_OPTIONS = [
  // EPSX Platform Permissions
  { id: 'epsx:analytics:view', label: 'View Analytics', platform: 'EPSX', color: 'bg-blue-500' },
  { id: 'epsx:analytics:export', label: 'Export Data', platform: 'EPSX', color: 'bg-blue-600' },
  { id: 'epsx:realtime:access', label: 'Real-time Data', platform: 'EPSX', color: 'bg-blue-700' },
  
  // Admin Permissions  
  { id: 'admin:users:manage', label: 'Manage Users', platform: 'Admin', color: 'bg-red-500' },
  { id: 'admin:analytics:view', label: 'Admin Analytics', platform: 'Admin', color: 'bg-red-600' },
  { id: 'admin:system:manage', label: 'System Config', platform: 'Admin', color: 'bg-red-700' },
  
  // EPSX Pay Permissions
  { id: 'epsx-pay:transactions:read', label: 'View Payments', platform: 'Pay', color: 'bg-green-500' },
  { id: 'epsx-pay:transactions:manage', label: 'Manage Payments', platform: 'Pay', color: 'bg-green-600' },
  
  // EPSX Token Permissions
  { id: 'epsx-token:governance:vote', label: 'Governance Vote', platform: 'Token', color: 'bg-purple-500' },
  { id: 'epsx-token:treasury:view', label: 'View Treasury', platform: 'Token', color: 'bg-purple-600' },
]

export default function CreateUserModal({ isOpen, onClose, onUserCreated }: CreateUserModalProps) {
  const [formData, setFormData] = useState<FormData>({
    email: '',
    displayName: '',
    permissions: []
  })
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitStatus, setSubmitStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [errorMessage, setErrorMessage] = useState('')

  const resetForm = () => {
    setFormData({ email: '', displayName: '', permissions: [] })
    setSubmitStatus('idle')
    setErrorMessage('')
    setIsSubmitting(false)
  }

  const handleClose = () => {
    resetForm()
    onClose()
  }

  const togglePermission = (permissionId: string) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.includes(permissionId)
        ? prev.permissions.filter(p => p !== permissionId)
        : [...prev.permissions, permissionId]
    }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!formData.email || formData.permissions.length === 0) {
      setErrorMessage('Email and at least one permission are required')
      return
    }

    setIsSubmitting(true)
    setErrorMessage('')

    try {
      const result = await ClientUserAPI.createUser({
        email: formData.email,
        display_name: formData.displayName || undefined,
        permissions: formData.permissions
      })

      console.log('✅ User created successfully:', result)
      setSubmitStatus('success')
      
      // Close modal after success and refresh the user list
      setTimeout(() => {
        handleClose()
        onUserCreated() // Refresh the parent component
      }, 1500)

    } catch (error) {
      console.error('❌ Failed to create user:', error)
      setSubmitStatus('error')
      setErrorMessage(error instanceof Error ? error.message : 'Failed to create user')
    } finally {
      setIsSubmitting(false)
    }
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50">
      <div className="bg-white dark:bg-gray-900 w-full max-w-lg shadow-xl border-2 border-yellow-400/20">
        {/* PancakeSwap x Windows Phone header */}
        <div className="bg-gradient-to-r from-yellow-400 to-orange-500 p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 bg-black rounded-lg flex items-center justify-center">
                <Plus size={16} className="text-yellow-400" />
              </div>
              <div>
                <h2 className="text-lg font-extralight tracking-wide text-black">create user</h2>
                <p className="text-black/70 text-xs font-light">add new team member</p>
              </div>
            </div>
            <button 
              onClick={handleClose}
              className="w-8 h-8 bg-black/10 hover:bg-black/20 flex items-center justify-center transition-all"
            >
              <X size={16} className="text-black" />
            </button>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-6">
          {/* Email Field */}
          <div>
            <label className="block text-sm font-light text-gray-700 dark:text-gray-300 mb-2 uppercase tracking-wider">
              <Mail size={14} className="inline mr-2" />
              email address
            </label>
            <input
              type="email"
              required
              value={formData.email}
              onChange={(e) => setFormData(prev => ({ ...prev, email: e.target.value }))}
              className="w-full px-4 py-3 bg-gray-50 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 text-gray-900 dark:text-white placeholder-gray-500 font-light focus:border-yellow-400 focus:outline-none transition-all"
              placeholder="user@company.com"
              disabled={isSubmitting}
            />
          </div>

          {/* Display Name Field */}
          <div>
            <label className="block text-sm font-light text-gray-700 dark:text-gray-300 mb-2 uppercase tracking-wider">
              <User size={14} className="inline mr-2" />
              display name (optional)
            </label>
            <input
              type="text"
              value={formData.displayName}
              onChange={(e) => setFormData(prev => ({ ...prev, displayName: e.target.value }))}
              className="w-full px-4 py-3 bg-gray-50 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 text-gray-900 dark:text-white placeholder-gray-500 font-light focus:border-yellow-400 focus:outline-none transition-all"
              placeholder="John Doe"
              disabled={isSubmitting}
            />
          </div>

          {/* Permissions Grid */}
          <div>
            <label className="block text-sm font-light text-gray-700 dark:text-gray-300 mb-3 uppercase tracking-wider">
              <Shield size={14} className="inline mr-2" />
              permissions ({formData.permissions.length} selected)
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-64 overflow-y-auto">
              {PERMISSION_OPTIONS.map(permission => (
                <button
                  key={permission.id}
                  type="button"
                  onClick={() => togglePermission(permission.id)}
                  disabled={isSubmitting}
                  className={`p-3 text-left transition-all border border-transparent hover:border-yellow-400 ${
                    formData.permissions.includes(permission.id)
                      ? `${permission.color} text-white`
                      : 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white hover:bg-gray-200 dark:hover:bg-gray-700'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-light text-sm">{permission.label}</div>
                      <div className="text-xs opacity-75">{permission.platform}</div>
                    </div>
                    {formData.permissions.includes(permission.id) && (
                      <Check size={14} />
                    )}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Error Message */}
          {errorMessage && (
            <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-600 dark:text-red-400">
              <AlertCircle size={16} />
              <span className="text-sm font-light">{errorMessage}</span>
            </div>
          )}

          {/* Success Message */}
          {submitStatus === 'success' && (
            <div className="flex items-center gap-2 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-green-600 dark:text-green-400">
              <Check size={16} />
              <span className="text-sm font-light">User created successfully!</span>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={handleClose}
              disabled={isSubmitting}
              className="flex-1 px-4 py-3 bg-gray-200 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-700 transition-all font-light disabled:opacity-50"
            >
              cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting || !formData.email || formData.permissions.length === 0}
              className="flex-1 px-4 py-3 bg-gradient-to-r from-yellow-400 to-orange-500 text-black font-medium hover:from-yellow-500 hover:to-orange-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? 'creating...' : 'create user'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}