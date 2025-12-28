'use client'

import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'
import * as Promo from '@/shared/utils/promo'
import { useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

interface PlanManagementProps {
  currentUser?: any
}

/**
 *
 * @param root0
 * @param root0.currentUser
 */
export function PlanManagement({ currentUser }: PlanManagementProps) {
  const { user: authUser } = useSharedAuth()
  // Note: currentUser and authUser available for future permission checks
  const _user = currentUser || authUser
  const router = useRouter()
  const [plans, setPlans] = useState<PlanResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [_selectedPlan, setSelectedPlan] = useState<PlanResponse | null>(null)
  const [filterCategory, setFilterCategory] = useState<'all' | 'standard' | 'api' | 'enterprise' | 'custom'>('all')

  // Load plans on component mount
  useEffect(() => {
    loadPlans()
  }, [])

  const loadPlans = async () => {
    try {
      setLoading(true)
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)

      const response = await plansClient.getPlans({
        limit: 100,
        plan_category: filterCategory === 'all' ? undefined : filterCategory
      })


      if (isApiSuccess(response)) {
        // Backend returns: { success, data: { plans: [...], has_more, total_count }, message }
        // API client wraps backend response as-is in its own data field
        // So response.data is the entire backend JSON response
        const backendResponse = response.data as any

        // Extract plans from backend response structure
        const plansData = backendResponse?.data?.plans || backendResponse?.plans || []

        setPlans(plansData)
      } else {
        console.error('[PlanManagement] API error:', response.error)
        toast({
          title: "Error",
          description: response.error || "Failed to load plans",
          variant: "destructive"
        })
      }
    } catch (error) {
      console.error('[PlanManagement] Exception:', error)
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to load plans",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }



  const filteredPlans = plans.filter(plan => {
    return filterCategory === 'all' || plan.plan_category === filterCategory
  })

  const standardPlans = plans.filter(p => p.plan_category === 'standard')
  const apiPlans = plans.filter(p => p.plan_category === 'api')
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise')
  const activePlans = plans.filter(p => p.is_active)

  const totalRevenue = plans.reduce((sum, plan) => {
    const revenue = typeof plan.revenue_last_30_days === 'string'
      ? parseFloat(plan.revenue_last_30_days)
      : plan.revenue_last_30_days
    return sum + (isNaN(revenue) ? 0 : revenue)
  }, 0)
  const avgRevenue = plans.length > 0 ? totalRevenue / plans.length : 0

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-emerald-400 to-green-500 rounded-2xl w-96 mx-auto mb-6"></div>
          <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto"></div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-gradient-to-br from-gray-300 to-gray-400 rounded-3xl h-64"></div>
          ))}
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-gray-300 rounded-3xl h-32"></div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div>
      <div className="space-y-6 sm:space-y-8">
        {/* Background Decorations */}
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
          <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
        </div>

        <div className="relative max-w-7xl mx-auto">
          {/* Hero Section */}
          <div className="text-center mb-8 sm:mb-12">
            <div className="relative inline-block">
              <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
                💳 Dynamic Plans
              </h1>
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
            </div>
            <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
              Create and manage unlimited plans with context-specific features for web app, API, and admin access
            </p>
          </div>

          {/* Action Cards */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-green-500/20 to-teal-500/20 p-0.5 cursor-pointer"
              onClick={() => router.push('/plans/new')}
            >
              <div className="relative bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 text-white rounded-2xl sm:rounded-3xl">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">➕</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Create Dynamic Plan</h3>
                  <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Create unlimited plans with context-specific features</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    New Plan
                  </div>
                </div>
              </div>
            </div>

            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-500/20 to-pink-500/20 p-0.5 cursor-pointer"
              onClick={() => loadPlans()}
            >
              <div className="relative bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 text-white rounded-2xl sm:rounded-3xl">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">🔄</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                  <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload plan data and analytics from server</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    Refresh
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Stats Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-emerald-300/50 dark:border-emerald-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">💳</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-emerald-600 dark:text-emerald-400">{plans.length}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Plans</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">All types</div>
              </div>
            </div>

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">✅</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Active</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-blue-600 dark:text-blue-400">{activePlans.length}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Active</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">Available</div>
              </div>
            </div>

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">🏢</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Enterprise</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-purple-600 dark:text-purple-400">{enterprisePlans.length}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Enterprise</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">Premium</div>
              </div>
            </div>

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">💵</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Price</span>
              </div>
              <div className="space-y-1">
                <div className="text-xl sm:text-3xl font-bold text-green-600 dark:text-green-400 truncate">
                  ${avgRevenue.toFixed(0)}
                </div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Average</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">USD</div>
              </div>
            </div>
          </div>

          {/* Category Filter Tabs */}
          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5 mb-6">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4">
              <div className="grid grid-cols-2 sm:flex gap-2 sm:gap-4">
                <button
                  onClick={() => setFilterCategory('all')}
                  className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${filterCategory === 'all'
                    ? 'bg-gradient-to-r from-emerald-400 to-green-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                    }`}
                >
                  All Plans ({plans.length})
                </button>
                <button
                  onClick={() => setFilterCategory('standard')}
                  className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${filterCategory === 'standard'
                    ? 'bg-gradient-to-r from-blue-400 to-purple-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                    }`}
                >
                  Standard ({standardPlans.length})
                </button>
                <button
                  onClick={() => setFilterCategory('api')}
                  className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${filterCategory === 'api'
                    ? 'bg-gradient-to-r from-orange-400 to-red-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                    }`}
                >
                  API Plans ({apiPlans.length})
                </button>
                <button
                  onClick={() => setFilterCategory('enterprise')}
                  className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${filterCategory === 'enterprise'
                    ? 'bg-gradient-to-r from-purple-400 to-pink-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                    }`}
                >
                  Enterprise ({enterprisePlans.length})
                </button>
              </div>
            </div>
          </div>

          {/* Plans List */}
          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-green-400/20 to-teal-400/20 p-0.5">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
              <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
                <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-emerald-600 via-green-600 to-teal-600 bg-clip-text text-transparent">
                  {filterCategory === 'all' ? 'All Plans' :
                    filterCategory === 'standard' ? 'Standard Plans' :
                      filterCategory === 'api' ? 'API Plans' : 'Enterprise Plans'}
                </h2>
                <div className="text-sm text-gray-500 dark:text-gray-400">
                  {filteredPlans.length} plans
                </div>
              </div>

              {/* Mobile Card Layout */}
              <div className="block sm:hidden space-y-4">
                {filteredPlans.map(plan => (
                  <div
                    key={plan.id}
                    className="p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl cursor-pointer"
                    onClick={() => setSelectedPlan(plan)}
                  >
                    <div className="flex items-center gap-3 mb-3">
                      <div className={`h-12 w-12 rounded-2xl flex items-center justify-center text-xl ${plan.plan_category === 'standard'
                        ? 'bg-gradient-to-br from-blue-400 to-purple-500'
                        : plan.plan_category === 'api'
                          ? 'bg-gradient-to-br from-orange-400 to-red-500'
                          : 'bg-gradient-to-br from-purple-400 to-pink-500'
                        }`}>
                        {plan.plan_category === 'standard' ? '👤' :
                          plan.plan_category === 'api' ? '🔧' : '🏢'}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1 flex-wrap">
                          <h3 className="text-lg font-bold text-gray-900 dark:text-white truncate">{plan.name}</h3>
                          {!plan.is_active && (
                            <span className="bg-gray-400 text-white text-xs px-2 py-1 rounded-full font-semibold">
                              INACTIVE
                            </span>
                          )}
                          {plan.promotion_active && plan.promotion_status === 'active' && (
                            <span className="bg-rose-500 text-white text-xs px-2 py-1 rounded-full font-semibold">
                              {Math.round(plan.promotion_discount || 0)}% OFF
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-2">
                          {Number(plan.current_price) === 0 ? (
                            <span className="text-sm font-bold bg-gradient-to-r from-emerald-500 to-green-500 bg-clip-text text-transparent">Free</span>
                          ) : plan.promotion_active && plan.effective_price !== undefined ? (
                            <>
                              <span className="text-sm line-through text-gray-500">${plan.current_price}</span>
                              <span className="text-sm font-bold text-rose-600 dark:text-rose-400">${plan.effective_price.toFixed(2)}</span>
                              <span className="text-xs text-gray-600 dark:text-gray-400">{plan.currency}</span>
                            </>
                          ) : (
                            <p className="text-sm text-gray-600 dark:text-gray-400">${plan.current_price} {plan.currency}</p>
                          )}
                        </div>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-3 mb-3">
                      <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Subscribers</div>
                        <div className="text-lg font-bold text-green-600 dark:text-green-400">{plan.subscriber_count}</div>
                      </div>
                      <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Revenue</div>
                        <div className="text-lg font-bold text-purple-600 dark:text-purple-400">${plan.revenue_last_30_days}</div>
                      </div>
                    </div>

                    <div className="grid grid-cols-3 gap-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          router.push(`/plans/${plan.id}/subscribers`)
                        }}
                        className="px-3 py-2 rounded-xl font-semibold bg-gradient-to-r from-emerald-400 to-green-500 text-white min-h-[44px] text-sm"
                      >
                        👥
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          router.push(`/plans/${plan.id}/edit`)
                        }}
                        className="px-3 py-2 rounded-xl font-semibold bg-gradient-to-r from-purple-400 to-purple-500 text-white min-h-[44px] text-sm"
                      >
                        ✏️ Edit
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          router.push(`/plans/${plan.id}/analytics`)
                        }}
                        className="px-3 py-2 rounded-xl font-semibold bg-gradient-to-r from-blue-400 to-blue-500 text-white min-h-[44px] text-sm"
                      >
                        📊
                      </button>
                    </div>
                  </div>
                ))}
              </div>

              {/* Desktop Layout */}
              <div className="hidden sm:block space-y-4">
                {filteredPlans.map(plan => (
                  <div
                    key={plan.id}
                    className="group relative overflow-hidden bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 hover:border-emerald-300 dark:hover:border-emerald-600 hover:shadow-lg hover:shadow-emerald-500/10 transition-all duration-300 cursor-pointer"
                    onClick={() => setSelectedPlan(plan)}
                  >
                    {/* Status indicator bar */}
                    <div className={`absolute top-0 left-0 right-0 h-1 ${!plan.is_active
                      ? 'bg-gray-400'
                      : plan.promotion_active
                        ? 'bg-gradient-to-r from-rose-500 to-pink-500'
                        : 'bg-gradient-to-r from-emerald-400 to-green-500'
                      }`} />

                    <div className="p-6">
                      {/* Top Row: Icon, Plan Info, Action Buttons */}
                      <div className="flex items-start gap-5">
                        {/* Plan Icon */}
                        <div className={`h-14 w-14 rounded-2xl flex items-center justify-center text-2xl flex-shrink-0 shadow-lg ${plan.plan_category === 'standard'
                          ? 'bg-gradient-to-br from-blue-400 to-purple-500'
                          : plan.plan_category === 'api'
                            ? 'bg-gradient-to-br from-orange-400 to-red-500'
                            : 'bg-gradient-to-br from-purple-400 to-pink-500'
                          }`}>
                          {plan.plan_category === 'standard' ? '👤' :
                            plan.plan_category === 'api' ? '🔧' : '🏢'}
                        </div>

                        {/* Plan Info - Title & Price */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1.5 flex-wrap">
                            <h3 className="text-xl font-bold text-gray-900 dark:text-white truncate">
                              {plan.name}
                            </h3>
                            {plan.plan_category === 'enterprise' && (
                              <span className="bg-gradient-to-r from-purple-500 to-pink-500 text-white text-xs px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wide">
                                Enterprise
                              </span>
                            )}
                            {!plan.is_active && (
                              <span className="bg-gray-500 text-white text-xs px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wide">
                                Inactive
                              </span>
                            )}
                            {plan.promotion_status && plan.promotion_status !== 'disabled' && (
                              <span className={`text-xs px-2.5 py-0.5 rounded-full font-bold ${Promo.getStatusColor(plan.promotion_status)}`}>
                                {Promo.getStatusIcon(plan.promotion_status)} {Promo.getStatusText(plan.promotion_status)}
                                {plan.promotion_status === 'active' && plan.promotion_ends_at && (
                                  <span className="ml-1 font-normal opacity-80">({Promo.getTimeRemaining(plan.promotion_ends_at)})</span>
                                )}
                              </span>
                            )}
                          </div>

                          {/* Price Display */}
                          <div className="flex items-baseline gap-2">
                            {Number(plan.current_price) === 0 ? (
                              <span className="text-2xl font-bold bg-gradient-to-r from-emerald-500 to-green-500 bg-clip-text text-transparent">
                                Free
                              </span>
                            ) : plan.promotion_active && plan.effective_price !== undefined ? (
                              <>
                                <span className="text-2xl font-bold text-rose-600 dark:text-rose-400">
                                  ${plan.effective_price.toFixed(2)}
                                </span>
                                <span className="text-sm text-gray-400 line-through">${plan.current_price}</span>
                                <span className="text-sm text-gray-500 dark:text-gray-400">{plan.currency}</span>
                                <span className="bg-rose-500 text-white text-xs px-2 py-0.5 rounded-full font-bold">
                                  {Math.round(plan.promotion_discount || 0)}% OFF
                                </span>
                              </>
                            ) : (
                              <>
                                <span className="text-2xl font-bold text-gray-900 dark:text-white">
                                  ${plan.current_price}
                                </span>
                                <span className="text-sm text-gray-500 dark:text-gray-400">{plan.currency}</span>
                              </>
                            )}
                          </div>
                        </div>

                        {/* Action Buttons - Now in a compact group */}
                        <div className="flex items-center gap-2 flex-shrink-0">
                          <button
                            onClick={(e) => {
                              e.stopPropagation()
                              router.push(`/plans/${plan.id}/subscribers`)
                            }}
                            className="group/btn flex items-center gap-2 px-4 py-2.5 rounded-xl font-medium bg-emerald-50 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100 dark:hover:bg-emerald-900/50 border border-emerald-200 dark:border-emerald-700 transition-all duration-200"
                            title="View Subscribers"
                          >
                            <span className="text-lg">👥</span>
                            <span className="hidden lg:inline">Subscribers</span>
                          </button>

                          <button
                            onClick={(e) => {
                              e.stopPropagation()
                              router.push(`/plans/${plan.id}/edit`)
                            }}
                            className="group/btn flex items-center gap-2 px-4 py-2.5 rounded-xl font-medium bg-purple-50 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 hover:bg-purple-100 dark:hover:bg-purple-900/50 border border-purple-200 dark:border-purple-700 transition-all duration-200"
                            title="Edit Plan"
                          >
                            <span className="text-lg">✏️</span>
                            <span className="hidden lg:inline">Edit</span>
                          </button>

                          <button
                            onClick={(e) => {
                              e.stopPropagation()
                              router.push(`/plans/${plan.id}/analytics`)
                            }}
                            className="group/btn flex items-center gap-2 px-4 py-2.5 rounded-xl font-medium bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 hover:bg-blue-100 dark:hover:bg-blue-900/50 border border-blue-200 dark:border-blue-700 transition-all duration-200"
                            title="View Analytics"
                          >
                            <span className="text-lg">📊</span>
                            <span className="hidden lg:inline">Analytics</span>
                          </button>
                        </div>
                      </div>

                      {/* Divider */}
                      <div className="h-px bg-gray-100 dark:bg-gray-700 my-4" />

                      {/* Bottom Row: Stats Grid */}
                      <div className="grid grid-cols-4 gap-4">
                        {/* Subscribers */}
                        <div className="bg-emerald-50 dark:bg-emerald-900/20 rounded-xl p-3">
                          <div className="flex items-center gap-2 text-emerald-600 dark:text-emerald-400 mb-1">
                            <span className="text-sm">👥</span>
                            <span className="text-xs font-medium uppercase tracking-wide">Subscribers</span>
                          </div>
                          <div className="text-xl font-bold text-emerald-700 dark:text-emerald-300">
                            {plan.subscriber_count}
                          </div>
                        </div>

                        {/* Permissions */}
                        <div className="bg-purple-50 dark:bg-purple-900/20 rounded-xl p-3">
                          <div className="flex items-center gap-2 text-purple-600 dark:text-purple-400 mb-1">
                            <span className="text-sm">🔐</span>
                            <span className="text-xs font-medium uppercase tracking-wide">Permissions</span>
                          </div>
                          <div className="text-xl font-bold text-purple-700 dark:text-purple-300">
                            {plan.permissions?.length || 0}
                          </div>
                        </div>

                        {/* Target Audience */}
                        <div className="bg-blue-50 dark:bg-blue-900/20 rounded-xl p-3">
                          <div className="flex items-center gap-2 text-blue-600 dark:text-blue-400 mb-1">
                            <span className="text-sm">🎯</span>
                            <span className="text-xs font-medium uppercase tracking-wide">Target</span>
                          </div>
                          <div className="text-sm font-bold text-blue-700 dark:text-blue-300 capitalize">
                            {plan.target_audience.replace('_', ' ')}
                          </div>
                        </div>

                        {/* Revenue */}
                        <div className="bg-amber-50 dark:bg-amber-900/20 rounded-xl p-3">
                          <div className="flex items-center gap-2 text-amber-600 dark:text-amber-400 mb-1">
                            <span className="text-sm">💰</span>
                            <span className="text-xs font-medium uppercase tracking-wide">Revenue (30d)</span>
                          </div>
                          <div className="text-xl font-bold text-amber-700 dark:text-amber-300">
                            ${plan.revenue_last_30_days}
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              {filteredPlans.length === 0 && (
                <div className="text-center py-12 sm:py-16">
                  <div className="h-20 w-20 bg-gradient-to-br from-emerald-200 to-green-200 dark:from-emerald-800 dark:to-green-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-4xl">📋</span>
                  </div>
                  <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                    No plans found
                  </h3>
                  <p className="text-gray-500 dark:text-gray-500">
                    {filterCategory === 'all'
                      ? 'Start by creating your first plan'
                      : `No ${filterCategory} plans available. Try switching filters or create a new plan.`
                    }
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PlanManagement;