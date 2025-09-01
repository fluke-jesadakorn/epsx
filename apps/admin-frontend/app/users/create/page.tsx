/**
 * User Creation Page - Server Component
 * Server-side rendered user creation form with server actions
 */

import { User, Mail, Shield, ArrowLeft, Save, UserPlus } from 'lucide-react'
import { AdminServerAPI } from '@/lib/server/admin-api'
import { revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { Breadcrumb } from '@/components/ui/Breadcrumb'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

// Server action for creating user
async function createUserAction(formData: FormData) {
  'use server'
  
  const email = formData.get('email')?.toString() || ''
  const displayName = formData.get('displayName')?.toString() || ''
  const password = formData.get('password')?.toString() || ''
  const selectedPermissions = formData.getAll('permissions') as string[]
  
  if (!email || !password) {
    redirect('/users/create?error=missing-fields')
  }
  
  if (selectedPermissions.length === 0) {
    redirect('/users/create?error=no-permissions-selected')
  }
  
  try {
    await AdminServerAPI.createUser({
      email: email.trim(),
      displayName: displayName.trim(),
      permissions: selectedPermissions,
      password
    })
    
    revalidatePath('/users')
    redirect('/users?success=user-created')
  } catch (error) {
    console.error('Failed to create user:', error)
    redirect('/users/create?error=creation-failed')
  }
}

interface Props {
  searchParams?: Promise<{
    error?: string
  }>
}

export default async function CreateUserPage({ searchParams }: Props) {
  const resolvedSearchParams = await searchParams
  const errorMessage = resolvedSearchParams?.error
  
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-2xl mx-auto px-4 py-8">
        {/* Enhanced Header with Windows Phone + PancakeSwap Breadcrumb */}
        <div className="mb-8">
          <div className="flex items-center gap-4 mb-6">
            <div className="w-14 h-14 bg-gradient-to-br from-green-500 to-emerald-600 rounded-xl flex items-center justify-center shadow-lg relative overflow-hidden">
              {/* PancakeSwap corner accent */}
              <div className="absolute top-0 right-0 w-4 h-4 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
              <UserPlus className="h-7 w-7 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-extralight tracking-wide text-gray-900 dark:text-gray-100">
                create new user
              </h1>
              <p className="text-gray-600 dark:text-gray-400 font-light">
                add a new user to the analytics platform
              </p>
            </div>
          </div>
          
          <Breadcrumb
            items={[
              { label: 'Users', href: '/users' },
              { label: 'Create', isActive: true }
            ]}
            variant="pivot"
          />
        </div>

        {/* Error Display */}
        {errorMessage && (
          <div className="mb-6 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-red-800 dark:text-red-200 text-sm">
            {errorMessage === 'missing-fields' && 'Email and password are required.'}
            {errorMessage === 'no-permissions-selected' && 'Please select at least one permission for the user.'}
            {errorMessage === 'creation-failed' && 'Failed to create user. Please try again.'}
          </div>
        )}

        {/* Server Form */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <form action={createUserAction} className="p-6 space-y-6">
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
                <Input
                  id="email"
                  name="email"
                  type="email"
                  required
                  variant="wp"
                  size="default"
                  placeholder="user@example.com"
                />
              </div>

              <div>
                <label htmlFor="displayName" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  <User className="h-4 w-4" />
                  Display Name
                </label>
                <Input
                  id="displayName"
                  name="displayName"
                  type="text"
                  variant="wp"
                  size="default"
                  placeholder="Full name (optional)"
                />
              </div>

              {/* Permission Selection */}
              <div>
                <label className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
                  <Shield className="h-4 w-4" />
                  User Permissions *
                </label>
                
                {/* Basic Access Permissions */}
                <div className="mb-6">
                  <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-3 border-b pb-1">
                    Analytics & Data Access
                  </h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <label className="flex items-start gap-3 p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:border-blue-300 dark:hover:border-blue-600 cursor-pointer">
                      <input type="checkbox" name="permissions" value="epsx:analytics:view" className="mt-1 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">Analytics Access</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">View analytics dashboard and data</div>
                        <code className="text-xs text-gray-400">epsx:analytics:view</code>
                      </div>
                    </label>
                    <label className="flex items-start gap-3 p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:border-blue-300 dark:hover:border-blue-600 cursor-pointer">
                      <input type="checkbox" name="permissions" value="epsx:analytics:export" className="mt-1 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">Export Data</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Download analytics reports</div>
                        <code className="text-xs text-gray-400">epsx:analytics:export</code>
                      </div>
                    </label>
                    <label className="flex items-start gap-3 p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:border-blue-300 dark:hover:border-blue-600 cursor-pointer">
                      <input type="checkbox" name="permissions" value="epsx:realtime:access" className="mt-1 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">Real-time Data</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Access live market data feeds</div>
                        <code className="text-xs text-gray-400">epsx:realtime:access</code>
                      </div>
                    </label>
                    <label className="flex items-start gap-3 p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:border-blue-300 dark:hover:border-blue-600 cursor-pointer">
                      <input type="checkbox" name="permissions" value="epsx:premium:access" className="mt-1 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">Premium Features</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Access premium tools and features</div>
                        <code className="text-xs text-gray-400">epsx:premium:access</code>
                      </div>
                    </label>
                  </div>
                </div>

                {/* Administrative Permissions */}
                <div className="mb-6">
                  <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-3 border-b pb-1 text-red-600 dark:text-red-400">
                    Administrative Permissions (Sensitive)
                  </h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <label className="flex items-start gap-3 p-3 border border-red-200 dark:border-red-800 rounded-lg hover:border-red-300 dark:hover:border-red-600 cursor-pointer bg-red-50 dark:bg-red-900/10">
                      <input type="checkbox" name="permissions" value="admin:users:view" className="mt-1 h-4 w-4 text-red-600 focus:ring-red-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">View Users</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Access user management interface</div>
                        <code className="text-xs text-gray-400">admin:users:view</code>
                      </div>
                    </label>
                    <label className="flex items-start gap-3 p-3 border border-red-200 dark:border-red-800 rounded-lg hover:border-red-300 dark:hover:border-red-600 cursor-pointer bg-red-50 dark:bg-red-900/10">
                      <input type="checkbox" name="permissions" value="admin:users:manage" className="mt-1 h-4 w-4 text-red-600 focus:ring-red-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">Manage Users</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Create, edit, and delete users</div>
                        <code className="text-xs text-gray-400">admin:users:manage</code>
                      </div>
                    </label>
                    <label className="flex items-start gap-3 p-3 border border-red-200 dark:border-red-800 rounded-lg hover:border-red-300 dark:hover:border-red-600 cursor-pointer bg-red-50 dark:bg-red-900/10">
                      <input type="checkbox" name="permissions" value="admin:permissions:manage" className="mt-1 h-4 w-4 text-red-600 focus:ring-red-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">Manage Permissions</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Grant and revoke user permissions</div>
                        <code className="text-xs text-gray-400">admin:permissions:manage</code>
                      </div>
                    </label>
                    <label className="flex items-start gap-3 p-3 border border-red-200 dark:border-red-800 rounded-lg hover:border-red-300 dark:hover:border-red-600 cursor-pointer bg-red-50 dark:bg-red-900/10">
                      <input type="checkbox" name="permissions" value="admin:system:configure" className="mt-1 h-4 w-4 text-red-600 focus:ring-red-500 border-gray-300 rounded" />
                      <div>
                        <div className="font-medium text-gray-900 dark:text-gray-100">System Configuration</div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">Access system settings</div>
                        <code className="text-xs text-gray-400">admin:system:configure</code>
                      </div>
                    </label>
                  </div>
                </div>

                <div className="text-sm text-gray-500 dark:text-gray-400 bg-blue-50 dark:bg-blue-900/20 p-3 rounded-lg">
                  <strong>Note:</strong> Permissions can be granted with optional expiry times after user creation. 
                  Select the appropriate permissions based on the user's role and responsibilities.
                </div>
              </div>

              <div>
                <label htmlFor="password" className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  <Shield className="h-4 w-4" />
                  Temporary Password *
                </label>
                <Input
                  id="password"
                  name="password"
                  type="password"
                  required
                  minLength={8}
                  variant="wp"
                  size="default"
                  placeholder="Enter temporary password"
                />
                <p className="text-xs text-gray-500 mt-1">User will be required to change password on first login</p>
              </div>
            </div>

            {/* Form Actions */}
            <div className="flex gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
              <a
                href="/users"
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
              >
                <ArrowLeft className="h-4 w-4" />
                Cancel
              </a>
              <Button
                type="submit"
                variant="pancake"
                size="default"
              >
                <Save className="h-4 w-4" />
                Create User
              </Button>
            </div>
          </form>
        </div>
      </div>
    </div>
  )
}