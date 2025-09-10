import { Suspense } from 'react'
import { PlanManagement } from '@/components/plans/PlanManagement'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { ServerAuth } from '@/lib/server/auth-helpers'
import { notFound } from 'next/navigation'

export const dynamic = 'force-dynamic'

function PlansHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-emerald-400 to-green-500 rounded-2xl w-96 mx-auto mb-4 animate-pulse shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto animate-pulse"></div>
        </div>
        
        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 rounded-3xl p-8 shadow-2xl animate-pulse">
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
              <div className="h-6 bg-gradient-to-r from-emerald-300 to-green-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>
        
        {/* Plans table skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-emerald-400 to-green-500 rounded-xl mb-6 w-1/3 animate-pulse"></div>
            <div className="space-y-4">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl animate-pulse">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-gradient-to-br from-emerald-400 to-green-500 rounded-2xl"></div>
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

async function PlansDataWrapper() {
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:plans:view')) {
    notFound()
  }
  
  // For now, we'll use demo data since our backend API is ready but frontend integration needs work
  const demoPlans = [
    {
      id: 1,
      name: 'Basic Plan',
      planType: 'personal',
      basePrice: 19.99,
      currentPrice: 19.99,
      currency: 'USD',
      features: ['Feature 1', 'Feature 2', 'Basic Analytics', 'Email Support'],
      affiliateCommissionRate: 10.00,
      displayOrder: 1,
      isActive: true,
      isHighlighted: false,
      createdAt: new Date(Date.now() - 86400000 * 30).toISOString(),
      updatedAt: new Date().toISOString(),
      activePromotions: [],
      effectivePrice: 19.99
    },
    {
      id: 2,
      name: 'Professional Plan',
      planType: 'personal',
      basePrice: 49.99,
      currentPrice: 39.99,
      currency: 'USD',
      features: ['All Basic Features', 'Advanced Analytics', 'Priority Support', 'Custom Reports'],
      affiliateCommissionRate: 15.00,
      displayOrder: 2,
      isActive: true,
      isHighlighted: true,
      createdAt: new Date(Date.now() - 86400000 * 20).toISOString(),
      updatedAt: new Date(Date.now() - 86400000 * 2).toISOString(),
      activePromotions: ['20% Early Bird'],
      effectivePrice: 39.99
    },
    {
      id: 3,
      name: 'API Starter',
      planType: 'api',
      basePrice: 99.99,
      currentPrice: 99.99,
      currency: 'USD',
      features: ['1000 API calls/month', 'Basic endpoints', 'Rate limiting', 'Documentation access'],
      affiliateCommissionRate: 12.00,
      displayOrder: 1,
      isActive: true,
      isHighlighted: false,
      createdAt: new Date(Date.now() - 86400000 * 15).toISOString(),
      updatedAt: new Date(Date.now() - 86400000 * 5).toISOString(),
      activePromotions: [],
      effectivePrice: 99.99
    },
    {
      id: 4,
      name: 'API Enterprise',
      planType: 'api',
      basePrice: 299.99,
      currentPrice: 299.99,
      currency: 'USD',
      features: ['10000 API calls/month', 'All endpoints', 'Priority rate limiting', 'Dedicated support'],
      affiliateCommissionRate: 20.00,
      displayOrder: 2,
      isActive: true,
      isHighlighted: true,
      createdAt: new Date(Date.now() - 86400000 * 10).toISOString(),
      updatedAt: new Date().toISOString(),
      activePromotions: [],
      effectivePrice: 299.99
    }
  ]
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <PlanManagement 
        plans={demoPlans}
        currentUser={session.user}
      />
    </div>
  )
}

export default function AdminPlansPage() {
  return (
    <Suspense fallback={<PlansHubSkeleton />}>
      <PlansDataWrapper />
    </Suspense>
  )
}