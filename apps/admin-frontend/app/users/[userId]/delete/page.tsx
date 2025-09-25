/**
 * User Delete Page - Server Component
 * Full-screen delete confirmation page for better mobile experience
 */

import { notFound, redirect } from 'next/navigation'
import { Trash2, AlertTriangle, ArrowLeft, User } from 'lucide-react'
import { AdminServerAPI } from '@/lib/server/admin-api'
import { revalidatePath } from 'next/cache'
import { Breadcrumb } from '@/components/ui/Breadcrumb'
import { Button } from '@/components/ui/button'

interface Props {
  params: Promise<{ userId: string }>
}

// Server action for deleting user
async function deleteUserAction(userId: string) {
  'use server'
  
  try {
    await AdminServerAPI.deleteUser(userId)
    revalidatePath('/users')
    redirect('/users?success=user-deleted')
  } catch (error) {
    console.error('Failed to delete user:', error)
    redirect(`/users/${userId}/delete?error=delete-failed`)
  }
}

export default async function DeleteUserPage({ params }: Props) {
  const { userId } = await params
  
  if (!userId) {
    redirect('/users')
  }

  let user
  try {
    user = await AdminServerAPI.getUserData(userId)
  } catch (error) {
    console.error('Failed to fetch user data:', error)
    notFound()
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-2xl mx-auto px-4 py-8">
        {/* Enhanced Header with Windows Phone + PancakeSwap Breadcrumb */}
        <div className="mb-8">
          <div className="flex items-center gap-4 mb-6">
            <div className="w-14 h-14 bg-gradient-to-br from-red-600 to-red-800 rounded-xl flex items-center justify-center shadow-lg relative overflow-hidden">
              {/* PancakeSwap corner accent */}
              <div className="absolute top-0 right-0 w-4 h-4 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
              <Trash2 className="h-7 w-7 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-extralight tracking-wide text-gray-900 dark:text-gray-100">
                delete user
              </h1>
              <p className="text-gray-600 dark:text-gray-400 font-light">
                permanently remove from analytics platform
              </p>
            </div>
          </div>
          
          <Breadcrumb
            items={[
              { label: 'Users', href: '/users' },
              { label: user.displayName || user.email, href: `/users/${userId}` },
              { label: 'Delete', isActive: true }
            ]}
            variant="pivot"
          />
        </div>

        {/* Warning Card */}
        <div className="bg-red-50 dark:bg-red-900/20 border-2 border-red-200 dark:border-red-800 rounded-lg p-6 mb-6">
          <div className="flex items-start gap-4">
            <AlertTriangle className="h-6 w-6 text-red-600 dark:text-red-400 flex-shrink-0 mt-1" />
            <div>
              <h3 className="text-lg font-semibold text-red-900 dark:text-red-100 mb-2">
                Warning: This action cannot be undone
              </h3>
              <p className="text-red-700 dark:text-red-300 mb-4">
                Deleting this user will permanently remove their account and all associated data. 
                This includes their profile, permissions, activity logs, and any other system data.
              </p>
              <ul className="text-sm text-red-600 dark:text-red-400 space-y-1">
                <li>• User profile and authentication data will be removed</li>
                <li>• All permissions and role assignments will be revoked</li>
                <li>• Activity logs will be preserved for audit purposes</li>
                <li>• Any active sessions will be terminated</li>
              </ul>
            </div>
          </div>
        </div>

        {/* User Details Card */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-8">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
            User to be deleted:
          </h3>
          
          <div className="flex items-start gap-4">
            <div className="w-12 h-12 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full flex items-center justify-center text-white font-medium">
              {user.email.charAt(0).toUpperCase()}
            </div>
            
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-2">
                <h4 className="font-medium text-gray-900 dark:text-white">
                  {user.displayName || 'No display name'}
                </h4>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                  user.status === 'active' 
                    ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                    : 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
                }`}>
                  {user.status || 'inactive'}
                </span>
              </div>
              
              <p className="text-gray-600 dark:text-gray-400 mb-2">{user.email}</p>
              
              <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                <span>ID: {(user.id || userId)?.substring(0, 8)}...</span>
                <span>Tier: {user.subscription_tier || 'basic'}</span>
                <span>Created: {new Date(user.created_at).toLocaleDateString()}</span>
              </div>

              {user.permissions && user.permissions.length > 0 && (
                <div className="mt-3">
                  <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Active Permissions:
                  </p>
                  <div className="flex flex-wrap gap-1">
                    {user.permissions.slice(0, 3).map((permission) => (
                      <span 
                        key={permission}
                        className="px-2 py-1 bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-400 text-xs rounded"
                      >
                        {permission}
                      </span>
                    ))}
                    {user.permissions.length > 3 && (
                      <span className="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 text-xs rounded">
                        +{user.permissions.length - 3} more
                      </span>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex gap-4 justify-end">
          <a
            href={`/users/${userId}`}
            className="px-6 py-3 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors font-medium"
          >
            Cancel
          </a>
          
          <form action={deleteUserAction.bind(null, userId)}>
            <Button
              type="submit"
              variant="destructive"
              size="lg"
            >
              <Trash2 className="h-4 w-4" />
              Delete User Permanently
            </Button>
          </form>
        </div>

        {/* Additional Info */}
        <div className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
          <p>
            Need help? Contact your system administrator or 
            <a href="/support" className="text-blue-600 dark:text-blue-400 hover:underline ml-1">
              support team
            </a>
          </p>
        </div>
      </div>
    </div>
  )
}