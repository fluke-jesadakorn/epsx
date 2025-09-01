/**
 * Bulk Operations Layout - Server Component
 * Shared layout for all bulk operations with user data pre-loading
 */

import { notFound, redirect } from 'next/navigation'
import { AdminServerAPI } from '@/lib/server/admin-api'
import { ArrowLeft, Users } from 'lucide-react'

interface Props {
  children: React.ReactNode
  params?: any
  searchParams?: Promise<{
    users?: string
  }>
}

export default async function BulkOperationsLayout({ 
  children,
  searchParams 
}: Props) {
  const resolvedSearchParams = await searchParams
  const selectedUserIds = resolvedSearchParams?.users?.split(',').filter(Boolean) || []
  
  if (selectedUserIds.length === 0) {
    redirect('/users?error=no-users-selected')
  }

  // Pre-load user data for all selected users
  let selectedUsers
  try {
    selectedUsers = await AdminServerAPI.getUsersByIds(selectedUserIds)
  } catch (error) {
    console.error('Failed to fetch selected users:', error)
    redirect('/users?error=fetch-failed')
  }

  // Filter out any null results
  const validUsers = selectedUsers.filter(user => user !== null)
  
  if (validUsers.length === 0) {
    redirect('/users?error=users-not-found')
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-6xl mx-auto px-4 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-[#FFC107] rounded-full flex items-center justify-center">
              <Users className="h-6 w-6 text-black" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                Bulk Operations
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                Manage multiple users simultaneously
              </p>
            </div>
          </div>
          
          <nav className="flex items-center text-sm text-gray-500 dark:text-gray-400">
            <a href="/users" className="hover:text-blue-600 dark:hover:text-blue-400 flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all duration-300 min-h-[44px] hover:scale-105 active:scale-95">
              <ArrowLeft className="h-5 w-5" />
              Users
            </a>
            <span className="mx-2">/</span>
            <span className="text-gray-900 dark:text-gray-100">Bulk Operations</span>
          </nav>
        </div>

        {/* Selected Users Summary */}
        <div className="bg-[#FFC107]/10 border border-[#FFC107]/30 rounded-lg p-6 mb-8">
          <div className="flex items-center gap-3 mb-4">
            <Users className="h-5 w-5 text-[#FFC107]" />
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Selected Users ({validUsers.length})
            </h3>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {validUsers.slice(0, 6).map((user) => (
              <div 
                key={user.id}
                className="flex items-center gap-3 p-3 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700"
              >
                <div className="w-8 h-8 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full flex items-center justify-center text-white text-sm font-medium">
                  {user.email.charAt(0).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-medium text-gray-900 dark:text-white text-sm truncate">
                    {user.displayName || 'No name'}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                    {user.email}
                  </p>
                </div>
                <span className={`px-2 py-1 rounded text-xs font-medium ${
                  user.status === 'active'
                    ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                    : 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
                }`}>
                  {user.status || 'inactive'}
                </span>
              </div>
            ))}
            
            {validUsers.length > 6 && (
              <div className="flex items-center justify-center p-3 bg-gray-100 dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600">
                <span className="text-sm text-gray-600 dark:text-gray-400 font-medium">
                  +{validUsers.length - 6} more users
                </span>
              </div>
            )}
          </div>
          
          {/* Quick Stats */}
          <div className="flex items-center gap-6 mt-4 pt-4 border-t border-[#FFC107]/20">
            <div className="text-sm">
              <span className="text-gray-600 dark:text-gray-400">Active: </span>
              <span className="font-medium text-green-600 dark:text-green-400">
                {validUsers.filter(u => u.status === 'active').length}
              </span>
            </div>
            <div className="text-sm">
              <span className="text-gray-600 dark:text-gray-400">Admins: </span>
              <span className="font-medium text-purple-600 dark:text-purple-400">
                {validUsers.filter(u => u.permissions?.some(p => p.startsWith('admin:'))).length}
              </span>
            </div>
            <div className="text-sm">
              <span className="text-gray-600 dark:text-gray-400">Premium: </span>
              <span className="font-medium text-[#FFC107]">
                {validUsers.filter(u => u.subscription_tier === 'premium').length}
              </span>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          {children}
        </div>

        {/* Footer Help */}
        <div className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
          <p>
            Bulk operations affect multiple users simultaneously. 
            <a href="/docs/bulk-operations" className="text-blue-600 dark:text-blue-400 hover:underline ml-1 px-2 py-1 rounded-lg hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all duration-300 min-h-[44px] inline-flex items-center">
              Learn more about bulk operations
            </a>
          </p>
        </div>
      </div>
    </div>
  )
}