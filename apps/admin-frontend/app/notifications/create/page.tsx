import { Suspense } from 'react'
import { NotificationSendForm } from '@/components/notifications/NotificationSendForm'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'

export const dynamic = 'force-dynamic'

function CreateNotificationPageSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 via-blue-50 to-indigo-100 p-6">
      <div className="max-w-4xl mx-auto">
        {/* Header skeleton */}
        <div className="mb-8">
          <div className="h-12 bg-gradient-to-r from-purple-600 to-blue-600 bg-clip-text w-96 mb-2 animate-pulse rounded-lg shadow-xl"></div>
          <div className="h-4 bg-gradient-to-r from-gray-400 to-gray-500 w-80 animate-pulse rounded-lg shadow-lg"></div>
        </div>

        {/* Delivery mode cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          {Array.from({ length: 3 }).map((_, i) => (
            <div 
              key={i} 
              className="h-32 bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-2xl shadow-2xl border-2 border-purple-400/30 animate-pulse"
            >
              <div className="p-4">
                <div className="h-6 bg-purple-300/40 rounded mb-2 w-1/2"></div>
                <div className="h-4 bg-purple-300/30 rounded w-3/4"></div>
              </div>
            </div>
          ))}
        </div>

        {/* Form skeleton */}
        <div className="bg-gradient-to-br from-white via-slate-50 to-purple-50 rounded-3xl shadow-2xl border-2 border-purple-200/30 p-8">
          <div className="space-y-6">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="space-y-2">
                <div className="h-4 bg-gradient-to-r from-purple-300 to-blue-300 w-32 rounded animate-pulse"></div>
                <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-lg animate-pulse"></div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

async function CreateNotificationDataWrapper() {
  // Check authentication and permissions
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:notifications:manage')) {
    notFound()
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 to-pink-600 bg-clip-text text-transparent mb-2">
            Create Notification
          </h1>
          <p className="text-gray-600 dark:text-gray-300">
            Send notifications to users across the platform
          </p>
        </div>
        
        <NotificationSendForm />
      </div>
    </div>
  )
}

export default function CreateNotificationPage() {
  return (
    <Suspense fallback={<CreateNotificationPageSkeleton />}>
      <CreateNotificationDataWrapper />
    </Suspense>
  )
}