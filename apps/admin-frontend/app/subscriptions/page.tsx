import { notFound } from 'next/navigation'
import { Suspense } from 'react'

import { SubscriptionManagement } from '@/components/subscriptions/SubscriptionManagement'
import { UnifiedAuth } from '@/lib/auth/auth'

export const dynamic = 'force-dynamic'

function SubscriptionsHubSkeleton() {
  return (
    <div className="min-h-screen p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-4 scale-105"></div>
          <div className="h-6 bg-muted rounded-full w-64 mx-auto "></div>
        </div>

        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-card rounded-3xl p-8 border border-border/50">
              <div className="h-12 w-12 bg-muted rounded-2xl mb-6"></div>
              <div className="h-8 bg-muted rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-muted/60 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-primary/20 rounded-2xl"></div>
            </div>
          ))}
        </div>

        {/* Stats grid skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-card/80 backdrop-blur-sm rounded-3xl p-6 border border-border/50 ">
              <div className="h-6 bg-primary/10 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-muted rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-muted/60 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>

        {/* Subscriptions table skeleton */}
        <div className="bg-card/90 backdrop-blur-sm rounded-3xl border border-border/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-primary/10 rounded-xl mb-6 w-1/3 "></div>
            <div className="space-y-4">
              {Array.from({ length: 8 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-muted/20 rounded-2xl ">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-primary/10 rounded-2xl"></div>
                    <div className="space-y-2 flex-1">
                      <div className="h-5 bg-muted rounded-lg w-1/3"></div>
                      <div className="h-4 bg-muted/60 rounded-lg w-1/2"></div>
                    </div>
                  </div>
                  <div className="h-8 w-24 bg-primary/10 rounded-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

async function SubscriptionsDataWrapper() {
  const session = await UnifiedAuth.getSession()

  if (!session?.user) {
    notFound()
  }

  // NOTE: Permission enforcement moved to backend
  // If user lacks permission, API calls will return 403 and show Access Denied UI

  return (
    <div className="min-h-screen p-6">
      <SubscriptionManagement currentUser={session.user} />
    </div>
  )
}

/**
 *
 */
export default function AdminSubscriptionsPage() {
  return (
    <Suspense fallback={<SubscriptionsHubSkeleton />}>
      <SubscriptionsDataWrapper />
    </Suspense>
  )
}