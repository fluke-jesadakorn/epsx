'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { PlanManagement } from '@/components/plans/PlanManagement';
import { PromotionManagement } from '@/components/promotions/PromotionManagement';
import { useSharedAuth } from '@/shared/components/auth/Provider';

function PlansHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-emerald-400 to-green-500 rounded-2xl w-96 mx-auto mb-4 shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto"></div>
        </div>

        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }, (_, i) => `action-card-${i}`).map((cardId) => (
            <div key={cardId} className="bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 rounded-3xl p-8 shadow-2xl">
              <div className="h-12 w-12 bg-white/20 rounded-2xl mb-6"></div>
              <div className="h-8 bg-white/30 rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-white/20 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-white/40 rounded-2xl"></div>
            </div>
          ))}
        </div>

        {/* Stats grid skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }, (_, i) => `stats-card-${i}`).map((cardId) => (
            <div key={cardId} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 shadow-xl border border-white/20">
              <div className="h-6 bg-gradient-to-r from-emerald-300 to-green-400 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-gradient-to-r from-gray-200 to-gray-300 rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-gray-200 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>

        {/* Plans table skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-emerald-400 to-green-500 rounded-xl mb-6 w-1/3"></div>
            <div className="space-y-4">
              {Array.from({ length: 6 }, (_, i) => `plan-row-${i}`).map((rowId) => (
                <div key={rowId} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl">
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

/**
 *
 */
export default function AdminPlansPage() {
  const { user, isAuthenticated, isLoading } = useSharedAuth()
  const router = useRouter()
  const [activeTab, setActiveTab] = useState<'plans' | 'promotions'>('plans');

  // Redirect to auth if not authenticated
  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/auth')
    }
  }, [isAuthenticated, isLoading, router])

  // Show loading state while checking authentication
  if (isLoading) {
    return <PlansHubSkeleton />
  }

  // Show loading state if not authenticated (will redirect)
  if (!isAuthenticated) {
    return <PlansHubSkeleton />
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto mb-6">
        {/* Tab Navigation */}
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-emerald-400/20 via-pink-400/20 to-purple-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-2">
            <div className="grid grid-cols-2 gap-2">
              <button
                onClick={() => setActiveTab('plans')}
                className={`px-6 py-3 rounded-xl font-semibold text-base min-h-[44px] ${activeTab === 'plans'
                  ? 'bg-gradient-to-r from-emerald-400 to-green-500 text-white shadow-lg'
                  : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }`}
              >
                💳 Plans
              </button>
              <button
                onClick={() => setActiveTab('promotions')}
                className={`px-6 py-3 rounded-xl font-semibold text-base min-h-[44px] ${activeTab === 'promotions'
                  ? 'bg-gradient-to-r from-pink-400 to-rose-500 text-white shadow-lg'
                  : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }`}
              >
                🎁 Promotions
              </button>
            </div>
          </div>
        </div>
      </div>

      {activeTab === 'plans' ? (
        <PlanManagement />
      ) : (
        <PromotionManagement />
      )}
    </div>
  )
}