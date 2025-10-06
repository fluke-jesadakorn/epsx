import { notFound } from 'next/navigation'
import { Suspense } from 'react'

import { PermissionRequestForm } from '@/components/permissions/PermissionForms'
import { UnifiedAuth } from '@/lib/auth/unified-auth'

export const dynamic = 'force-dynamic'

export interface PermissionRequestPageProps {
  searchParams?: Promise<{
    feature?: string
    permission?: string
    tier?: string
  }>
}

function PermissionRequestPageSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-4xl mx-auto">
        {/* Header skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-blue-400 to-purple-500 rounded-2xl w-96 mx-auto mb-4 animate-pulse shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-80 mx-auto animate-pulse"></div>
        </div>

        {/* Feature info card skeleton */}
        <div className="bg-gradient-to-r from-blue-500 via-purple-600 to-pink-500 rounded-3xl shadow-2xl p-8 mb-8">
          <div className="h-8 bg-white/20 rounded-xl mb-4 w-1/2 animate-pulse"></div>
          <div className="h-4 bg-white/15 rounded-lg mb-2 w-3/4 animate-pulse"></div>
          <div className="h-4 bg-white/15 rounded-lg w-1/2 animate-pulse"></div>
        </div>

        {/* Form skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 p-8">
          <div className="space-y-8">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="space-y-3">
                <div className="h-4 bg-gradient-to-r from-blue-300 to-purple-400 w-32 rounded animate-pulse"></div>
                <div className="h-14 bg-gradient-to-r from-gray-200 to-gray-300 rounded-2xl animate-pulse"></div>
              </div>
            ))}

            {/* Action buttons skeleton */}
            <div className="flex justify-end gap-4 pt-6">
              <div className="h-12 w-24 bg-gray-200 rounded-2xl animate-pulse"></div>
              <div className="h-12 w-36 bg-gradient-to-r from-blue-400 to-purple-500 rounded-2xl animate-pulse"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

async function PermissionRequestDataWrapper({ searchParams }: { searchParams?: PermissionRequestPageProps['searchParams'] }) {
  // Check authentication 
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  // Parse search parameters - await in Next.js 15
  const resolvedSearchParams = await searchParams
  const featureName = resolvedSearchParams?.feature || 'Advanced Feature'
  const requiredPermission = resolvedSearchParams?.permission
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 to-pink-600 bg-clip-text text-transparent mb-2">
            Request Permission
          </h1>
          <p className="text-gray-600 dark:text-gray-300">
            Request access to {featureName} feature
          </p>
        </div>
        
        <PermissionRequestForm 
          onSubmit={async (request) => {
            // TODO: Implement actual permission request submission
          }}
        />
      </div>
    </div>
  )
}

/**
 *
 * @param props
 */
export default function PermissionRequestPage(props: PermissionRequestPageProps) {
  return (
    <Suspense fallback={<PermissionRequestPageSkeleton />}>
      <PermissionRequestDataWrapper searchParams={props.searchParams} />
    </Suspense>
  )
}