'use client';

import { useState } from 'react'

import { PlanManagement } from '@/components/plans/PlanManagement'
import { PromotionManagement } from '@/components/promotions/PromotionManagement'
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider'

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

// Demo promotion data
const demoPromotions = [
  {
    id: 1,
    name: 'Black Friday 2024',
    code: 'BLACKFRIDAY24',
    discountType: 'percentage' as const,
    discountValue: 25,
    maxDiscountAmount: 50,
    minPurchaseAmount: 99,
    usageLimit: 1000,
    currentUsage: 347,
    isActive: true,
    startDate: '2024-11-25T00:00:00Z',
    endDate: '2024-11-30T23:59:59Z',
    applicablePlans: ['personal', 'api'],
    description: 'Black Friday mega sale - 25% off all plans',
    createdAt: new Date(Date.now() - 86400000 * 10).toISOString(),
    updatedAt: new Date(Date.now() - 86400000 * 2).toISOString(),
    totalRevenue: 8675.50,
    conversionRate: 12.5
  },
  {
    id: 2,
    name: 'New Year Special',
    code: 'NEWYEAR2025',
    discountType: 'fixed' as const,
    discountValue: 20,
    maxDiscountAmount: null,
    minPurchaseAmount: 50,
    usageLimit: 500,
    currentUsage: 89,
    isActive: true,
    startDate: '2025-01-01T00:00:00Z',
    endDate: '2025-01-31T23:59:59Z',
    applicablePlans: ['personal'],
    description: 'New Year kickstart - $20 off personal plans',
    createdAt: new Date(Date.now() - 86400000 * 5).toISOString(),
    updatedAt: new Date().toISOString(),
    totalRevenue: 1780.00,
    conversionRate: 8.9
  },
  {
    id: 3,
    name: 'API Launch Promo',
    code: 'APILAUNCH',
    discountType: 'percentage' as const,
    discountValue: 15,
    maxDiscountAmount: 100,
    minPurchaseAmount: 199,
    usageLimit: 200,
    currentUsage: 156,
    isActive: true,
    startDate: '2024-12-01T00:00:00Z',
    endDate: '2025-03-01T23:59:59Z',
    applicablePlans: ['api'],
    description: 'Special launch promotion for API plans',
    createdAt: new Date(Date.now() - 86400000 * 15).toISOString(),
    updatedAt: new Date(Date.now() - 86400000 * 1).toISOString(),
    totalRevenue: 4680.75,
    conversionRate: 15.2
  },
  {
    id: 4,
    name: 'Summer Sale',
    code: 'SUMMER2024',
    discountType: 'percentage' as const,
    discountValue: 30,
    maxDiscountAmount: 75,
    minPurchaseAmount: 0,
    usageLimit: 750,
    currentUsage: 642,
    isActive: false,
    startDate: '2024-06-01T00:00:00Z',
    endDate: '2024-08-31T23:59:59Z',
    applicablePlans: ['personal', 'api'],
    description: 'Summer vacation special - 30% off everything',
    createdAt: new Date(Date.now() - 86400000 * 90).toISOString(),
    updatedAt: new Date(Date.now() - 86400000 * 30).toISOString(),
    totalRevenue: 12456.25,
    conversionRate: 18.7
  },
  {
    id: 5,
    name: 'Student Discount',
    code: 'STUDENT50',
    discountType: 'percentage' as const,
    discountValue: 50,
    maxDiscountAmount: 25,
    minPurchaseAmount: 0,
    usageLimit: null,
    currentUsage: 1247,
    isActive: true,
    startDate: '2024-01-01T00:00:00Z',
    endDate: '2025-12-31T23:59:59Z',
    applicablePlans: ['personal'],
    description: 'Year-round student discount program',
    createdAt: new Date(Date.now() - 86400000 * 180).toISOString(),
    updatedAt: new Date(Date.now() - 86400000 * 7).toISOString(),
    totalRevenue: 15234.80,
    conversionRate: 22.3
  }
]

/**
 *
 */
export default function AdminPlansPage() {
  const { user, isLoading, isAuthenticated } = useSharedAuth();
  const [activeTab, setActiveTab] = useState<'plans' | 'promotions'>('plans');

  if (isLoading) {
    return <PlansHubSkeleton />;
  }

  if (!isAuthenticated || !user) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Authentication Required</h1>
          <p className="text-gray-600 dark:text-gray-400">Please connect your wallet to access this page.</p>
        </div>
      </div>
    );
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
                className={`px-6 py-3 rounded-xl font-semibold text-base min-h-[44px] ${
                  activeTab === 'plans'
                    ? 'bg-gradient-to-r from-emerald-400 to-green-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                💳 Plans
              </button>
              <button
                onClick={() => setActiveTab('promotions')}
                className={`px-6 py-3 rounded-xl font-semibold text-base min-h-[44px] ${
                  activeTab === 'promotions'
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
        <PlanManagement currentUser={user} />
      ) : (
        <PromotionManagement promotions={demoPromotions} currentUser={user} />
      )}
    </div>
  )
}