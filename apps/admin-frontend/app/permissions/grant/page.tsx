import { Suspense } from 'react'
import { GrantPermissionHub } from '@/components/admin/GrantPermissionHub'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'
import { ServerAuth } from '@/lib/server/auth-helpers'
import { transformBackendUsersResponse, validateBackendUser } from '@/lib/transformers/user-transformer'
import { notFound } from 'next/navigation'

export const dynamic = 'force-dynamic'

export interface GrantPermissionPageProps {
  searchParams?: Promise<{
    userId?: string
    email?: string
  }>
}

function GrantPermissionPageSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-6xl mx-auto">
        {/* Header Skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-yellow-400/30 to-pink-500/30 w-96 mx-auto mb-4 animate-pulse rounded-3xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300/50 to-gray-400/50 dark:from-gray-600/50 dark:to-gray-700/50 w-80 mx-auto animate-pulse rounded-2xl"></div>
        </div>
        
        {/* Stats Cards Skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 via-orange-400/20 to-pink-400/20 p-0.5">
              <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6 h-32">
                <div className="animate-pulse">
                  <div className="flex justify-between items-start mb-4">
                    <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded-xl w-16"></div>
                    <div className="w-10 h-10 bg-gradient-to-r from-yellow-400/50 to-orange-500/50 rounded-2xl"></div>
                  </div>
                  <div className="h-8 bg-gray-300 dark:bg-gray-600 rounded-xl w-12 mb-2"></div>
                  <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-xl w-24"></div>
                </div>
              </div>
            </div>
          ))}
        </div>
        
        {/* Search Bar Skeleton */}
        <div className="mb-8">
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/10 to-pink-400/10 p-0.5">
            <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-4">
              <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded-2xl animate-pulse"></div>
            </div>
          </div>
        </div>
        
        {/* User Cards Grid Skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {Array.from({ length: 8 }).map((_, i) => (
            <div key={i} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-red-400/20 p-0.5 group">
              <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6 h-48">
                <div className="animate-pulse">
                  <div className="flex justify-between items-start mb-4">
                    <div className="w-3 h-3 bg-green-400/50 rounded-full"></div>
                    <div className="flex gap-1">
                      <div className="w-4 h-4 bg-purple-400/50 rounded"></div>
                    </div>
                  </div>
                  <div className="space-y-2 mb-4">
                    <div className="h-5 bg-gray-300 dark:bg-gray-600 rounded-xl w-32"></div>
                    <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded-xl w-24"></div>
                    <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-xl w-20"></div>
                  </div>
                  <div className="flex justify-between items-center">
                    <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-xl w-16"></div>
                    <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-xl w-12"></div>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

async function GrantPermissionDataWrapper({ searchParams }: { searchParams?: GrantPermissionPageProps['searchParams'] }) {
  // Check authentication and permissions
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:permissions:grant')) {
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
    
    // Transform backend response to frontend format
    if (response.success && response.data) {
      const validUsers = response.data.users?.filter(validateBackendUser) || []
      const transformedData = transformBackendUsersResponse({
        users: validUsers,
        total_count: response.data.total || response.data.total_count || 0
      })
      users = transformedData.users
    }
  } catch (error) {
    console.error('💥 Grant permissions - Failed to fetch users:', error)
    
    // Fallback demo data
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
          'epsx:users:manage'
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
          'epsx:analytics:basic'
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
          'epsx:rankings:view:1'
        ]
      }
    ]
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <GrantPermissionHub 
        users={users}
        currentUser={session.user}
      />
    </div>
  )
}

export default function GrantPermissionPage(props: GrantPermissionPageProps) {
  return (
    <Suspense fallback={<GrantPermissionPageSkeleton />}>
      <GrantPermissionDataWrapper searchParams={props.searchParams} />
    </Suspense>
  )
}