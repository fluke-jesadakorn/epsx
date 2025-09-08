/**
 * User Edit Form Component
 * Server Action integrated form for user profile editing
 */

'use client'

import { useState, useTransition } from 'react'
import { useRouter } from 'next/navigation'
import { User, Mail, Shield, Phone, Globe, Clock, Loader2, ArrowLeft, Save } from 'lucide-react'
import { updateUser } from '@/app/actions/admin-server'
import { updateUserProfile } from '@/app/actions/user-actions'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

interface UserEditFormProps {
  user: UnifiedUserData
}

interface EditFormData {
  displayName: string
  phoneNumber: string
  timezone: string
  language: string
  status: 'active' | 'disabled' | 'pending' | 'suspended'
}

export function UserEditForm({ user }: UserEditFormProps) {
  const router = useRouter()
  const [isPending, startTransition] = useTransition()
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  
  const [formData, setFormData] = useState<EditFormData>({
    displayName: user.displayName || '',
    phoneNumber: user.phoneNumber || '',
    timezone: user.timezone || 'UTC',
    language: user.language || 'en',
    status: user.status || 'active'
  })

  // Server Action wrapper
  async function handleFormAction(formData: FormData) {
    const userId = user.id || user.user_id || user.firebase_uid
    
    if (!userId) {
      setError('User ID not found')
      return
    }

    const updateData = {
      name: formData.get('displayName') as string,
      is_active: formData.get('status') !== 'disabled'
    }

    startTransition(async () => {
      setError(null)
      setSuccess(null)
      
      try {
        // Use server action for main user update
        const result = await updateUser(userId, updateData)
        
        if (!result.success) {
          setError(result.error || 'Failed to update user')
          return
        }

        // Update profile details
        const profileResult = await updateUserProfile(userId, {
          name: formData.get('displayName') as string,
          phone: formData.get('phoneNumber') as string,
          timezone: formData.get('timezone') as string,
          language: formData.get('language') as string,
        })

        if (!profileResult.success) {
          setError(profileResult.error || 'Failed to update profile')
          return
        }

        setSuccess('User updated successfully')
        
        // Navigate after successful update
        setTimeout(() => {
          router.push(`/users/${userId}`)
        }, 1000)
        
      } catch (err) {
        console.error('Update error:', err)
        setError('An unexpected error occurred')
      }
    })
  }

  const handleCancel = () => {
    router.back()
  }

  return (
    <div className="p-6">
      <form action={handleFormAction} className="space-y-6">
        {error && (
          <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-red-800 dark:text-red-200 text-sm">
            {error}
          </div>
        )}
        
        {success && (
          <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md text-green-800 dark:text-green-200 text-sm">
            {success}
          </div>
        )}

        {/* Basic Information */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 border-b pb-2">
            Basic Information
          </h3>
          
          <div>
            <label htmlFor="displayName" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <User className="h-4 w-4" />
              Display Name
            </label>
            <Input
              id="displayName"
              name="displayName"
              type="text"
              defaultValue={formData.displayName}
              variant="wp"
              size="default"
              placeholder="Full name"
              required
            />
          </div>

          <div>
            <label htmlFor="email" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Mail className="h-4 w-4" />
              Email Address
            </label>
            <Input
              id="email"
              type="email"
              value={user.email}
              disabled
              variant="ghost"
              size="default"
            />
            <p className="text-xs text-gray-500 mt-1">Email cannot be changed</p>
          </div>

          <div>
            <label htmlFor="phoneNumber" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Phone className="h-4 w-4" />
              Phone Number
            </label>
            <Input
              id="phoneNumber"
              name="phoneNumber"
              type="text"
              defaultValue={formData.phoneNumber}
              variant="wp"
              size="default"
              placeholder="Phone number (optional)"
            />
          </div>

          <div>
            <label htmlFor="status" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Shield className="h-4 w-4" />
              Status
            </label>
            <select
              id="status"
              name="status"
              defaultValue={formData.status}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
            >
              <option value="active">Active</option>
              <option value="disabled">Disabled</option>
              <option value="pending">Pending</option>
              <option value="suspended">Suspended</option>
            </select>
          </div>
        </div>

        {/* Preferences */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 border-b pb-2">
            Preferences
          </h3>
          
          <div>
            <label htmlFor="timezone" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Clock className="h-4 w-4" />
              Timezone
            </label>
            <select
              id="timezone"
              name="timezone"
              defaultValue={formData.timezone}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
            >
              <option value="UTC">UTC</option>
              <option value="America/New_York">Eastern Time</option>
              <option value="America/Chicago">Central Time</option>
              <option value="America/Denver">Mountain Time</option>
              <option value="America/Los_Angeles">Pacific Time</option>
              <option value="Europe/London">London</option>
              <option value="Europe/Paris">Paris</option>
              <option value="Asia/Tokyo">Tokyo</option>
            </select>
          </div>

          <div>
            <label htmlFor="language" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <Globe className="h-4 w-4" />
              Language
            </label>
            <select
              id="language"
              name="language"
              defaultValue={formData.language}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:border-transparent transition-colors"
            >
              <option value="en">English</option>
              <option value="es">Spanish</option>
              <option value="fr">French</option>
              <option value="de">German</option>
              <option value="ja">Japanese</option>
              <option value="zh">Chinese</option>
            </select>
          </div>
        </div>

        {/* Security Settings - Read Only */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 border-b pb-2">
            Security Information
          </h3>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-3 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium">Email Verified</p>
                <div className={`px-2 py-1 rounded text-xs font-medium ${
                  user.emailVerified 
                    ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-400'
                    : 'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-400'
                }`}>
                  {user.emailVerified ? 'Verified' : 'Not Verified'}
                </div>
              </div>
            </div>

            <div className="p-3 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium">Two-Factor Auth</p>
                <div className={`px-2 py-1 rounded text-xs font-medium ${
                  user.twoFactorEnabled 
                    ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-400'
                    : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-400'
                }`}>
                  {user.twoFactorEnabled ? 'Enabled' : 'Disabled'}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Form Actions */}
        <div className="flex gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
          <Button
            type="button"
            onClick={handleCancel}
            disabled={isPending}
            variant="outline"
            size="default"
          >
            <ArrowLeft className="h-4 w-4" />
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={isPending}
            variant="wp"
            size="default"
          >
            {isPending && <Loader2 className="h-4 w-4 animate-spin" />}
            <Save className="h-4 w-4" />
            Save Changes
          </Button>
        </div>
      </form>
    </div>
  )
}