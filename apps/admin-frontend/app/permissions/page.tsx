import { Suspense } from 'react'
import { PermissionManagement } from '@/components/permissions/PermissionManagement'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'
import { ServerAuth } from '@/lib/server/auth-helpers'
import { transformBackendUsersResponse, validateBackendUser } from '@/lib/transformers/user-transformer'
import { notFound } from 'next/navigation'

export const dynamic = 'force-dynamic'

function PermissionsHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl w-96 mx-auto mb-4 animate-pulse shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto animate-pulse"></div>
        </div>
        
        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-gradient-to-br from-yellow-400 via-orange-500 to-pink-500 rounded-3xl p-8 shadow-2xl animate-pulse">
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
            <div key={i} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 shadow-xl border border-white/20 animate-pulse">
              <div className="h-6 bg-gradient-to-r from-yellow-300 to-orange-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>
        
        {/* Permissions table skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-xl mb-6 w-1/3 animate-pulse"></div>
            <div className="space-y-4">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl animate-pulse">
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

async function PermissionsDataWrapper() {
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:permissions:view')) {
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
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <PermissionManagement 
        users={users}
        currentUser={session.user}
      />
    </div>
  )
}

export default function AdminPermissionsPage() {
  return (
    <Suspense fallback={<PermissionsHubSkeleton />}>
      <PermissionsDataWrapper />
    </Suspense>
  )
}