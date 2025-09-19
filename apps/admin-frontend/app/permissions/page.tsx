import { Suspense } from 'react'
import { PermissionManagement } from '@/components/permissions/PermissionManagement'
import { PermissionForms } from '@/components/permissions/PermissionForms'
import { GrantPermissionHub } from '@/components/admin/GrantPermissionHub'
import { GrantPermissionForm } from '@/components/permissions/GrantPermissionForm'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'
import { ServerAuth } from '@/lib/server/auth-helpers'
import { transformBackendUsersResponse, validateBackendUser } from '@/lib/transformers/user-transformer'
import { notFound } from 'next/navigation'

export const dynamic = 'force-dynamic'

type PermissionMode = 'view' | 'bulk' | 'grant'

interface PermissionsPageProps {
  searchParams?: Promise<{
    mode?: PermissionMode
    userId?: string
    email?: string
  }>
}

function PermissionsHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl w-96 mx-auto mb-4 shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto"></div>
        </div>
        
        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-gradient-to-br from-yellow-400 via-orange-500 to-pink-500 rounded-3xl p-8 shadow-2xl">
              <div className="h-12 w-12 bg-white/20 rounded-2xl mb-6"></div>
              <div className="h-8 bg-white/30 rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-white/20 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-white/40 rounded-2xl"></div>
            </div>
          ))}
        </div>
        
        {/* Stats grid skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 shadow-xl border border-white/20">
              <div className="h-6 bg-gradient-to-r from-yellow-300 to-orange-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>
        
        {/* Permissions table skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-xl mb-6 w-1/3"></div>
            <div className="space-y-4">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-2xl"></div>
                    <div className="space-y-2 flex-1">
                      <div className="h-5 bg-gray-300 rounded-lg w-1/3"></div>
                      <div className="h-4 bg-gray-200 rounded-lg w-1/2"></div>
                    </div>
                  </div>
                  <div className="h-8 w-24 bg-gradient-to-r from-green-400 to-green-500 rounded-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

async function PermissionsDataWrapper({ searchParams }: { searchParams?: PermissionsPageProps['searchParams'] }) {
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  // Await searchParams and extract values
  const params = searchParams ? await searchParams : {}
  const mode = params.mode || 'view'
  const userId = params.userId
  const email = params.email
  
  // Check permissions based on mode
  const requiredPermissions = {
    view: 'admin:permissions:view',
    bulk: 'admin:permissions:manage', 
    grant: 'admin:permissions:grant'
  }
  
  if (!UnifiedAuth.hasPermission(session.user, requiredPermissions[mode])) {
    notFound()
  }
  
  // Get server-side authentication token and create properly configured client
  const { accessToken } = await ServerAuth.getTokens()
  const client = new UnifiedAdminClient(undefined, accessToken, true)
  let users = []
  
  try {
    const response = await client.getUsers({ 
      limit: 100,
      offset: 0
    })
    
    console.log('🔍 Permissions page - Users API response:', {
      success: response.success,
      hasData: !!response.data,
      usersCount: response.data?.users?.length || 0
    })
    
    // Transform backend response to frontend format
    if (response.success && response.data) {
      // Validate and transform backend data
      const validUsers = response.data.users?.filter(validateBackendUser) || []
      const transformedData = transformBackendUsersResponse({
        users: validUsers,
        total_count: response.data.total || response.data.total_count || 0
      })
      users = transformedData.users
      
      console.log('✅ Permissions page - Transformed users:', users.length)
    } else {
      console.warn('❌ Permissions page - API response was not successful:', response)
      users = []
    }
  } catch (error) {
    console.error('💥 Permissions page - Failed to fetch users data:', error)
    
    // Fallback demo data for when backend is unavailable
    console.log('🎯 Using demo data for permissions page')
    users = [
      {
        id: 'demo-user-1',
        email: 'admin@epsx.io',
        name: 'EPSX Admin',
        displayName: 'EPSX Admin',
        role: 'admin' as const,
        status: 'active' as const,
        isActive: true,
        createdAt: new Date(Date.now() - 86400000 * 30).toISOString(),
        updatedAt: new Date().toISOString(),
        lastLoginAt: new Date(Date.now() - 3600000).toISOString(),
        firebaseUid: 'demo-firebase-uid-1',
        permissions: [
          'admin:*:*',
          'epsx:analytics:view',
          'epsx:users:manage',
          'epsx:rankings:view:10'
        ]
      },
      {
        id: 'demo-user-2', 
        email: 'user@epsx.io',
        name: 'Premium User',
        displayName: 'Premium User',
        role: 'premium_user' as const,
        status: 'active' as const,
        isActive: true,
        createdAt: new Date(Date.now() - 86400000 * 15).toISOString(),
        updatedAt: new Date(Date.now() - 86400000).toISOString(),
        lastLoginAt: new Date(Date.now() - 7200000).toISOString(),
        firebaseUid: 'demo-firebase-uid-2',
        permissions: [
          'epsx:rankings:view:5',
          'epsx:analytics:basic',
          'epsx:export:csv'
        ]
      },
      {
        id: 'demo-user-3',
        email: 'testuser@example.com',
        name: 'Test User',
        displayName: 'Test User',
        role: 'user' as const,
        status: 'active' as const,
        isActive: true,
        createdAt: new Date(Date.now() - 86400000 * 7).toISOString(),
        updatedAt: new Date(Date.now() - 86400000 * 2).toISOString(),
        lastLoginAt: new Date(Date.now() - 14400000).toISOString(),
        firebaseUid: 'demo-firebase-uid-3',
        permissions: [
          'epsx:rankings:view:1',
          'epsx:basic:access'
        ]
      },
      {
        id: 'demo-user-4',
        email: 'trader@epsx.io',
        name: 'Professional Trader',
        displayName: 'Professional Trader',
        role: 'premium_user' as const,
        status: 'active' as const,
        isActive: true,
        createdAt: new Date(Date.now() - 86400000 * 45).toISOString(),
        updatedAt: new Date(Date.now() - 86400000 * 3).toISOString(),
        lastLoginAt: new Date(Date.now() - 1800000).toISOString(),
        firebaseUid: 'demo-firebase-uid-4',
        permissions: [
          'epsx:rankings:view:10',
          'epsx:analytics:premium',
          'epsx:export:json',
          'epsx:alerts:create'
        ]
      },
      {
        id: 'demo-user-5',
        email: 'newbie@example.com', 
        name: 'New User',
        displayName: 'New User',
        role: 'user' as const,
        status: 'active' as const,
        isActive: true,
        createdAt: new Date(Date.now() - 86400000 * 2).toISOString(),
        updatedAt: new Date(Date.now() - 86400000).toISOString(),
        lastLoginAt: new Date(Date.now() - 28800000).toISOString(),
        firebaseUid: 'demo-firebase-uid-5',
        permissions: [
          'epsx:rankings:view:1'
        ]
      }
    ]
  }
  
  // Validate user ID or email if provided for grant mode
  let selectedUser: (typeof users)[0] | null = null
  if (mode === 'grant' && (userId || email)) {
    if (userId) {
      selectedUser = users.find(u => u.id === userId) || null
      if (!selectedUser) {
        console.error(`❌ User not found with ID: ${userId}`)
        // Return error page or redirect with error
        return (
          <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
            <div className="max-w-2xl mx-auto text-center py-20">
              <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl p-8 shadow-xl">
                <div className="mb-6">
                  <div className="w-16 h-16 bg-gradient-to-r from-red-400 to-red-600 rounded-full mx-auto flex items-center justify-center">
                    <span className="text-white text-2xl">⚠️</span>
                  </div>
                </div>
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">User Not Found</h1>
                <p className="text-gray-600 dark:text-gray-300 mb-6">
                  The user with ID "{userId}" could not be found in the system.
                </p>
                <div className="space-y-2 text-sm text-gray-500 dark:text-gray-400">
                  <p>Please check:</p>
                  <ul className="list-disc list-inside space-y-1">
                    <li>The user ID is correct and exists in the database</li>
                    <li>The user hasn't been deleted or deactivated</li>
                    <li>You have permission to access this user's information</li>
                  </ul>
                </div>
                <button 
                  onClick={() => window.history.back()}
                  className="mt-6 px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-2xl hover:from-purple-700 hover:to-pink-700 transition-colors"
                >
                  Go Back
                </button>
              </div>
            </div>
          </div>
        )
      }
    } else if (email) {
      selectedUser = users.find(u => u.email === email) || null
      if (!selectedUser) {
        console.error(`❌ User not found with email: ${email}`)
        return (
          <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
            <div className="max-w-2xl mx-auto text-center py-20">
              <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl p-8 shadow-xl">
                <div className="mb-6">
                  <div className="w-16 h-16 bg-gradient-to-r from-red-400 to-red-600 rounded-full mx-auto flex items-center justify-center">
                    <span className="text-white text-2xl">⚠️</span>
                  </div>
                </div>
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">User Not Found</h1>
                <p className="text-gray-600 dark:text-gray-300 mb-6">
                  The user with email "{email}" could not be found in the system.
                </p>
                <div className="space-y-2 text-sm text-gray-500 dark:text-gray-400">
                  <p>Please check:</p>
                  <ul className="list-disc list-inside space-y-1">
                    <li>The email address is correct and exists in the database</li>
                    <li>The user hasn't been deleted or deactivated</li>
                    <li>You have permission to access this user's information</li>
                  </ul>
                </div>
                <button 
                  onClick={() => window.history.back()}
                  className="mt-6 px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-2xl hover:from-purple-700 hover:to-pink-700 transition-colors"
                >
                  Go Back
                </button>
              </div>
            </div>
          </div>
        )
      }
    }
  }
  
  // Render appropriate component based on mode
  const renderComponent = () => {
    switch (mode) {
      case 'bulk':
        return (
          <PermissionForms 
            mode="bulkGrant"
            users={users}
            currentUser={session.user}
          />
        )
      case 'grant':
        if (selectedUser) {
          return (
            <GrantPermissionForm 
              user={selectedUser}
              currentUser={session.user}
              onClose={() => window.history.back()}
            />
          )
        } else {
          return (
            <GrantPermissionHub 
              users={users}
              currentUser={session.user}
            />
          )
        }
      default:
        return (
          <PermissionManagement 
            users={users}
            currentUser={session.user}
          />
        )
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {renderComponent()}
    </div>
  )
}

export default function AdminPermissionsPage(props: PermissionsPageProps) {
  return (
    <Suspense fallback={<PermissionsHubSkeleton />}>
      <PermissionsDataWrapper searchParams={props.searchParams} />
    </Suspense>
  )
}