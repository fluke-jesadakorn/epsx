/**
 * User Creation Form Component
 * Client-side form for creating new users
 */

'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { User, Mail, Shield, Phone, ArrowLeft, Save, Loader2 } from 'lucide-react'
import { createUser } from '@/lib/actions/unified-user-actions'

interface CreateFormData {
  email: string
  displayName: string
  role: string
  password: string
}

export function UserCreateForm() {
  const router = useRouter()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const [formData, setFormData] = useState<CreateFormData>({
    email: '',
    displayName: '',
    role: 'user-basic-001',
    password: ''
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)
    setError(null)

    try {
      const form = new FormData()
      form.append('email', formData.email.trim())
      form.append('displayName', formData.displayName.trim())
      form.append('role', formData.role)
      form.append('password', formData.password)

      const result = await createUser(form)
      
      if (!result.success) {
        setError(result.error?.message || 'Failed to create user')
        return
      }

      // Redirect to users list on success
      router.push('/users')
      router.refresh()
    } catch (_err) {
      setError('An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  const handleCancel = () => {
    router.push('/users')
  }

  return (
    <div className="p-6">
      <form onSubmit={handleSubmit} className="space-y-6">
        {error && (
          <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-red-800 dark:text-red-200 text-sm">
            {error}
          </div>
        )}

        {/* Basic Information */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 border-b pb-2">
            User Information
          </h3>
          
          <div>
            <label htmlFor="email" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Mail className="h-4 w-4" />
              Email Address *
            </label>
            <input
              id="email"
              type="email"
              required
              value={formData.email}
              onChange={(e) => setFormData(prev => ({ ...prev, email: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
              placeholder="user@example.com"
            />
          </div>

          <div>
            <label htmlFor="displayName" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <User className="h-4 w-4" />
              Display Name
            </label>
            <input
              id="displayName"
              type="text"
              value={formData.displayName}
              onChange={(e) => setFormData(prev => ({ ...prev, displayName: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
              placeholder="Full name (optional)"
            />
          </div>

          <div>
            <label htmlFor="role" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Shield className="h-4 w-4" />
              Role
            </label>
            <select
              id="role"
              value={formData.role}
              onChange={(e) => setFormData(prev => ({ ...prev, role: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
            >
              <option value="user-basic-001">Basic User</option>
              <option value="user-premium-002">Premium User</option>
              <option value="moderator-standard-003">Moderator</option>
              <option value="admin-full-004">Admin</option>
            </select>
          </div>

          <div>
            <label htmlFor="password" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Shield className="h-4 w-4" />
              Temporary Password *
            </label>
            <input
              id="password"
              type="password"
              required
              value={formData.password}
              onChange={(e) => setFormData(prev => ({ ...prev, password: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
              placeholder="Enter temporary password"
              minLength={8}
            />
            <p className="text-xs text-gray-500 mt-1">User will be required to change password on first login</p>
          </div>
        </div>

        {/* Form Actions */}
        <div className="flex gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
          <button
            type="button"
            onClick={handleCancel}
            disabled={isLoading}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors disabled:opacity-50"
          >
            <ArrowLeft className="h-4 w-4" />
            Cancel
          </button>
          <button
            type="submit"
            disabled={isLoading || !formData.email || !formData.password}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 dark:bg-green-600 dark:hover:bg-green-700 rounded-md transition-colors disabled:opacity-50"
          >
            {isLoading && <Loader2 className="h-4 w-4 animate-spin" />}
            <Save className="h-4 w-4" />
            Create User
          </button>
        </div>
      </form>
    </div>
  )
}