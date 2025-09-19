import { Suspense } from 'react'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'
import { ServerAuth } from '@/lib/server/auth-helpers'
import { notFound } from 'next/navigation'
import UserPermissionInheritance from '@/components/permissions/UserPermissionInheritance'
import { UserIcon, ArrowLeftIcon } from 'lucide-react'
import Link from 'next/link'

export const dynamic = 'force-dynamic'

interface Props {
  params: {
    userId: string
  }
}

function UserPermissionsSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-green-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-blue-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Back Navigation */}
        <div className="flex items-center gap-3 mb-6">
          <div className="h-8 w-8 bg-gray-200 rounded animate-pulse"></div>
          <div className="h-6 bg-gray-200 rounded w-32 animate-pulse"></div>
        </div>
        
        {/* Header skeleton */}
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-3">
            <div className="h-6 w-6 bg-blue-400 rounded animate-pulse"></div>
            <div className="space-y-2">
              <div className="h-6 bg-gradient-to-r from-blue-400 to-green-500 rounded w-64 animate-pulse"></div>
              <div className="h-4 bg-gray-200 rounded w-48 animate-pulse"></div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <div className="h-9 bg-gray-200 rounded w-32 animate-pulse"></div>
            <div className="h-9 bg-blue-500 rounded w-36 animate-pulse"></div>
          </div>
        </div>
        
        {/* Performance Stats */}
        <div className="bg-gray-50 rounded-2xl p-4 mb-6">
          <div className="grid grid-cols-2 lg:grid-cols-6 gap-4">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="flex items-center gap-2">
                <div className="h-4 w-4 bg-blue-400 rounded animate-pulse"></div>
                <div className="space-y-1">
                  <div className="h-3 bg-gray-200 rounded w-16 animate-pulse"></div>
                  <div className="h-5 bg-blue-300 rounded w-8 animate-pulse"></div>
                </div>
              </div>
            ))}
          </div>
        </div>
        
        {/* Permission Lists */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          {Array.from({ length: 2 }).map((_, cardIndex) => (
            <div key={cardIndex} className="bg-white/90 backdrop-blur-sm rounded-3xl shadow-2xl p-6">
              <div className="flex items-center gap-2 mb-4">
                <div className="h-5 w-5 bg-blue-400 rounded animate-pulse"></div>
                <div className="h-5 bg-gray-200 rounded w-32 animate-pulse"></div>
                <div className="h-6 bg-gray-100 rounded w-8 ml-auto animate-pulse"></div>
              </div>
              
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {Array.from({ length: 4 }).map((_, i) => (
                  <div key={i} className="flex items-center gap-3 p-3 bg-blue-50 rounded-lg">
                    <div className="h-4 w-4 bg-blue-400 rounded animate-pulse"></div>
                    <div className="h-4 bg-gray-200 rounded flex-1 animate-pulse"></div>
                    <div className="h-5 bg-gray-100 rounded w-20 animate-pulse"></div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
        
        {/* Impact Analysis */}
        <div className="bg-gray-50 rounded-3xl p-6">
          <div className="h-6 bg-gray-200 rounded w-48 mb-4 animate-pulse"></div>
          <div className="space-y-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="p-4 bg-blue-50 rounded-lg">
                <div className="h-5 bg-blue-300 rounded w-64 mb-2 animate-pulse"></div>
                <div className="space-y-1">
                  <div className="h-3 bg-blue-200 rounded w-full animate-pulse"></div>
                  <div className="h-3 bg-blue-200 rounded w-3/4 animate-pulse"></div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

async function UserPermissionsDataWrapper({ params }: Props) {
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
  let user = null
  
  try {
    const response = await client.getUser(params.userId)
    
    if (response.success && response.data) {
      user = response.data
    } else {
      console.warn('❌ Failed to fetch user data:', response)
    }
  } catch (error) {
    console.error('💥 Failed to fetch user data:', error)
    
    // Fallback demo user
    user = {
      id: params.userId,
      email: 'demo@epsx.io',
      display_name: 'Demo User',
      tier: 'premium',
      is_active: true,
      last_login_at: new Date(Date.now() - 3600000).toISOString(),
    }
  }
  
  if (!user) {
    notFound()
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-green-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-blue-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Back Navigation */}
        <div className="flex items-center gap-3 mb-6">
          <Link 
            href="/users" 
            className="flex items-center gap-2 text-gray-600 hover:text-gray-900 transition-colors"
          >
            <ArrowLeftIcon className="h-4 w-4" />
            <span className="text-sm font-medium">Back to Users</span>
          </Link>
        </div>
        
        <UserPermissionInheritance userId={params.userId} user={user} />
      </div>
    </div>
  )
}

export default function UserPermissionsPage({ params }: Props) {
  return (
    <Suspense fallback={<UserPermissionsSkeleton />}>
      <UserPermissionsDataWrapper params={params} />
    </Suspense>
  )
}