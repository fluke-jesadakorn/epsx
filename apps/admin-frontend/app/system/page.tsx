import { Suspense } from 'react'
import { SettingsDashboard } from '@/components/admin/SettingsDashboard'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

function SystemHubSkeleton() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="mb-8">
        <div className="h-10 bg-gray-200 dark:bg-gray-700 rounded w-72 mb-2 "></div>
        <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-64 "></div>
      </div>
      
      {/* Status cards skeleton */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="h-24 bg-gray-200 dark:bg-gray-700 rounded-lg "></div>
        ))}
      </div>
      
      {/* Pivot navigation skeleton */}
      <div className="mb-6">
        <div className="flex gap-4 border-b border-gray-200 dark:border-gray-700 pb-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="h-6 bg-gray-200 dark:bg-gray-700 rounded w-20 "></div>
          ))}
        </div>
      </div>
      
      {/* Metrics widgets skeleton */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-64 bg-gray-200 dark:bg-gray-700 rounded-lg "></div>
        ))}
      </div>
      
      {/* Configuration sections skeleton */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6 mb-8">
        <div className="h-48 bg-gray-200 dark:bg-gray-700 rounded-lg "></div>
        <div className="h-48 bg-gray-200 dark:bg-gray-700 rounded-lg "></div>
      </div>
      
      {/* Feature flags skeleton */}
      <div className="mb-8">
        <div className="h-6 bg-gray-200 dark:bg-gray-700 rounded w-32 mb-4 "></div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="h-16 bg-gray-200 dark:bg-gray-700 rounded-lg "></div>
          ))}
        </div>
      </div>
      
      {/* Action buttons skeleton */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="h-20 bg-gray-200 dark:bg-gray-700 rounded-lg "></div>
        ))}
      </div>
    </div>
  )
}

async function SystemDataWrapper() {
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="mb-8">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 to-pink-600 bg-clip-text text-transparent mb-2">
            System Management
          </h1>
          <p className="text-gray-600 dark:text-gray-300">
            Configure system settings and monitor platform health
          </p>
        </div>
        
        <SettingsDashboard 
          initialSystemConfig={{}}
          initialGeneralSettings={{}}
          initialNotificationSettings={{}}
          initialSecuritySettings={{}}
          initialFeatureFlags={{}}
          initialEnvironmentConfig={{}}
        />
      </div>
    </div>
  )
}

export default function SystemPage() {
  return (
    <Suspense fallback={<SystemHubSkeleton />}>
      <SystemDataWrapper />
    </Suspense>
  )
}