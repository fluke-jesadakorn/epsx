import { Suspense } from 'react'
import { RemoteConfigDashboard } from '@/components/remote-config/RemoteConfigDashboard'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { ServerAuth } from '@/lib/server/auth-helpers'
import { notFound } from 'next/navigation'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

export interface RemoteConfigPageProps {
  searchParams?: Promise<{
    tab?: string
    filter?: string
  }>
}

function RemoteConfigSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>
      
      <div className="relative z-10">
        {/* Page Header Skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-yellow-400/30 to-pink-500/30 w-96 mx-auto mb-4 rounded-3xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300/50 to-gray-400/50 dark:from-gray-600/50 dark:to-gray-700/50 w-80 mx-auto rounded-2xl"></div>
        </div>
        
        {/* Stats Cards Skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 via-orange-400/20 to-pink-400/20 p-0.5 group">
              <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6 h-32">
                <div className="absolute top-2 right-2 w-8 h-8 bg-gradient-to-br from-yellow-300/30 to-orange-400/30 rounded-full blur-sm"></div>
                <div>
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
        
        {/* Navigation Tabs Skeleton */}
        <div className="mb-8">
          <div className="flex gap-8 border-b border-gray-200/50 dark:border-gray-700/50 pb-3">
            {Array.from({ length: 7 }).map((_, i) => (
              <div key={i} className="h-6 bg-gradient-to-r from-gray-300/50 to-gray-400/50 dark:from-gray-600/50 dark:to-gray-700/50 w-20 rounded-xl"></div>
            ))}
          </div>
        </div>
        
        {/* Content Area Skeleton */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-red-400/20 p-0.5 group">
              <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6 h-48">
                <div className="absolute top-2 right-2 w-6 h-6 bg-gradient-to-br from-purple-300/30 to-pink-400/30 rounded-full blur-sm"></div>
                <div>
                  <div className="flex justify-between items-start mb-4">
                    <div className="w-3 h-3 bg-green-400/50 rounded-full"></div>
                    <div className="flex gap-1">
                      <div className="w-4 h-4 bg-yellow-400/50 rounded"></div>
                      <div className="w-4 h-4 bg-orange-400/50 rounded"></div>
                    </div>
                  </div>
                  <div className="space-y-2 mb-4">
                    <div className="h-5 bg-gray-300 dark:bg-gray-600 rounded-xl w-32"></div>
                    <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded-xl w-48"></div>
                    <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-xl w-36"></div>
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

// Server component that fetches data and passes to client component
async function RemoteConfigDataWrapper({ searchParams }: { searchParams?: RemoteConfigPageProps['searchParams'] }) {
  // Check authentication first
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:config:manage')) {
    notFound()
  }
  
  // Parse search parameters - await in Next.js 15
  const resolvedSearchParams = await searchParams
  const activeTab = resolvedSearchParams?.tab || 'overview'
  const filter = resolvedSearchParams?.filter || ''
  
  // Get server-side authentication token
  const { accessToken } = await ServerAuth.getTokens()
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <RemoteConfigDashboard 
        initialTab={activeTab}
        initialFilter={filter}
      />
    </div>
  )
}

export default function RemoteConfigPage(props: RemoteConfigPageProps) {
  return (
    <Suspense fallback={<RemoteConfigSkeleton />}>
      <RemoteConfigDataWrapper searchParams={props.searchParams} />
    </Suspense>
  )
}

export const metadata = {
  title: 'Remote Config Management - EPSX Admin',
  description: 'Manage Firebase Remote Config parameters, user targeting, and A/B testing experiments'
}