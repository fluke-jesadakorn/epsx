'use client'

import { useState, useEffect } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { adminClient, PlanResponse, PlanListResponse, isApiSuccess } from '@/lib/api/unified-admin-client'
import { CreatePlanForm } from './CreatePlanForm'
import { PlanAnalyticsModal } from './PlanAnalyticsModal'
import { toast } from '@/hooks/use-toast'

interface PlanManagementProps {
  currentUser: any
}

export function PlanManagement({ currentUser }: PlanManagementProps) {
  const [plans, setPlans] = useState<PlanResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedPlan, setSelectedPlan] = useState<PlanResponse | null>(null)
  const [isCreating, setIsCreating] = useState(false)
  const [showAnalytics, setShowAnalytics] = useState<number | null>(null)
  const [filterCategory, setFilterCategory] = useState<'all' | 'standard' | 'api' | 'enterprise' | 'custom'>('all')
  const [filterActive, setFilterActive] = useState<boolean | null>(null)

  // Load plans on component mount
  useEffect(() => {
    loadPlans()
  }, [])

  const loadPlans = async () => {
    try {
      setLoading(true)
      const response = await adminClient.getPlans({
        limit: 100,
        plan_category: filterCategory === 'all' ? undefined : filterCategory,
        is_active: filterActive
      })

      if (isApiSuccess(response)) {
        const data = response.data as PlanListResponse
        setPlans(data.plans || [])
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to load plans",
          variant: "destructive"
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to load plans",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handlePlanCreated = () => {
    setIsCreating(false)
    loadPlans()
    toast({
      title: "Success",
      description: "Plan created successfully",
    })
  }

  const handleTogglePlanStatus = async (planId: number, currentStatus: boolean) => {
    try {
      const response = await adminClient.updatePlan(planId, {
        is_active: !currentStatus
      })

      if (isApiSuccess(response)) {
        loadPlans()
        toast({
          title: "Success",
          description: `Plan ${!currentStatus ? 'activated' : 'deactivated'} successfully`,
        })
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to update plan status",
          variant: "destructive"
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to update plan status",
        variant: "destructive"
      })
    }
  }

  const filteredPlans = plans.filter(plan => {
    const categoryMatch = filterCategory === 'all' || plan.plan_category === filterCategory
    const activeMatch = filterActive === null || plan.is_active === filterActive
    return categoryMatch && activeMatch
  })

  const standardPlans = plans.filter(p => p.plan_category === 'standard')
  const apiPlans = plans.filter(p => p.plan_category === 'api')
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise')
  const activePlans = plans.filter(p => p.is_active)

  const totalRevenue = plans.reduce((sum, plan) => sum + plan.revenue_last_30_days, 0)
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
    <>
      {/* Plan Creation Form Modal */}
      {isCreating && (
        <CreatePlanForm 
          onClose={() => setIsCreating(false)}
          onSuccess={handlePlanCreated}
        />
      )}

      {/* Plan Analytics Modal */}
      {showAnalytics && (
        <PlanAnalyticsModal 
          planId={showAnalytics}
          onClose={() => setShowAnalytics(null)}
        />
      )}

      <div className="max-w-7xl mx-auto space-y-8">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <div className="inline-flex items-center gap-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white px-8 py-4 rounded-2xl shadow-xl mb-6">
            <span className="text-3xl">💳</span>
            <h1 className="text-3xl font-bold">Dynamic Plans Management</h1>
          </div>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
            Create and manage unlimited plans with context-specific features for web app, API, and admin access
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <PancakeCard 
            className="bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 text-white cursor-pointer hover:scale-105 transition-transform"
            onClick={() => setIsCreating(true)}
          >
            <div className="p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                <span className="text-2xl">➕</span>
              </div>
              <h3 className="text-2xl font-bold mb-4">Create Dynamic Plan</h3>
              <p className="text-white/80 mb-6">Create unlimited plans with context-specific features</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                New Plan
              </div>
            </div>
          </PancakeCard>

          <PancakeCard 
            className="bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 text-white cursor-pointer hover:scale-105 transition-transform"
            onClick={() => loadPlans()}
          >
            <div className="p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                <span className="text-2xl">🔄</span>
              </div>
              <h3 className="text-2xl font-bold mb-4">Refresh Data</h3>
              <p className="text-white/80 mb-6">Reload plan data and analytics from server</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                Refresh
              </div>
            </div>
          </PancakeCard>

          <PancakeCard className="bg-gradient-to-br from-orange-400 via-red-500 to-pink-500 text-white">
            <div className="p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                <span className="text-2xl">📊</span>
              </div>
              <h3 className="text-2xl font-bold mb-4">Global Analytics</h3>
              <p className="text-white/80 mb-6">Overall plan performance and revenue metrics</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                Coming Soon
              </div>
            </div>
          </PancakeCard>
        </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-emerald-600 dark:text-emerald-400 font-semibold mb-2">Total Plans</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{plans.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">All plan types</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-blue-600 dark:text-blue-400 font-semibold mb-2">Active Plans</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{activePlans.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">Currently available</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-purple-600 dark:text-purple-400 font-semibold mb-2">Enterprise</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{enterprisePlans.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">Enterprise plans</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-green-600 dark:text-green-400 font-semibold mb-2">Avg Price</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              ${avgRevenue.toFixed(0)}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">USD per plan</div>
          </div>
        </PancakeCard>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-4 mb-6">
        <button
          onClick={() => setFilterCategory('all')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterCategory === 'all'
              ? 'bg-gradient-to-r from-emerald-400 to-green-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          All Plans ({plans.length})
        </button>
        <button
          onClick={() => setFilterCategory('standard')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterCategory === 'standard'
              ? 'bg-gradient-to-r from-blue-400 to-purple-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Standard ({standardPlans.length})
        </button>
        <button
          onClick={() => setFilterCategory('api')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterCategory === 'api'
              ? 'bg-gradient-to-r from-orange-400 to-red-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          API Plans ({apiPlans.length})
        </button>
        <button
          onClick={() => setFilterCategory('enterprise')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterCategory === 'enterprise'
              ? 'bg-gradient-to-r from-purple-400 to-pink-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Enterprise ({enterprisePlans.length})
        </button>
      </div>

      {/* Plans List */}
      <PancakeCard className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border border-white/30 overflow-hidden">
        <div className="p-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
              {filterCategory === 'all' ? 'All Plans' : 
               filterCategory === 'standard' ? 'Standard Plans' : 
               filterCategory === 'api' ? 'API Plans' : 'Enterprise Plans'}
            </h2>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {filteredPlans.length} plans
            </div>
          </div>

          <div className="space-y-4">
            {filteredPlans.map(plan => (
              <div
                key={plan.id}
                className="flex items-center justify-between p-6 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl hover:from-emerald-50 hover:to-green-50 dark:hover:from-gray-600 dark:hover:to-gray-700 transition-all cursor-pointer"
                onClick={() => setSelectedPlan(plan)}
              >
                <div className="flex items-center gap-6 flex-1">
                  <div className={`h-12 w-12 rounded-2xl flex items-center justify-center text-2xl ${
                    plan.plan_category === 'standard' 
                      ? 'bg-gradient-to-br from-blue-400 to-purple-500'
                      : plan.plan_category === 'api'
                      ? 'bg-gradient-to-br from-orange-400 to-red-500'
                      : 'bg-gradient-to-br from-purple-400 to-pink-500'
                  }`}>
                    {plan.plan_category === 'standard' ? '👤' : 
                     plan.plan_category === 'api' ? '🔧' : '🏢'}
                  </div>
                  
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                        {plan.name}
                      </h3>
                      {plan.plan_category === 'enterprise' && (
                        <span className="bg-gradient-to-r from-purple-400 to-pink-500 text-white text-xs px-3 py-1 rounded-full font-semibold">
                          ENTERPRISE
                        </span>
                      )}
                      {!plan.is_active && (
                        <span className="bg-gray-400 text-white text-xs px-3 py-1 rounded-full font-semibold">
                          INACTIVE
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
                      <span className="font-semibold">
                        ${plan.current_price} {plan.currency}
                      </span>
                      <span>•</span>
                      <span>{plan.features.length} features</span>
                      <span>•</span>
                      <span>{plan.target_audience.replace('_', ' ')}</span>
                      <span>•</span>
                      <span>${plan.revenue_last_30_days} revenue</span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-4">
                  <div className="text-xs bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400 px-3 py-1 rounded-full">
                    {plan.subscriber_count} subscribers
                  </div>
                  
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      handleTogglePlanStatus(plan.id, plan.is_active)
                    }}
                    className={`px-4 py-2 rounded-xl font-semibold transition-all ${
                      plan.is_active
                        ? 'bg-gradient-to-r from-green-400 to-green-500 text-white'
                        : 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300'
                    }`}
                  >
                    {plan.is_active ? 'Active' : 'Inactive'}
                  </button>
                  
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      setShowAnalytics(plan.id)
                    }}
                    className="px-4 py-2 rounded-xl font-semibold bg-gradient-to-r from-blue-400 to-blue-500 text-white transition-all hover:from-blue-500 hover:to-blue-600"
                  >
                    Analytics
                  </button>
                </div>
              </div>
            ))}
          </div>

          {filteredPlans.length === 0 && (
            <div className="text-center py-12">
              <div className="text-6xl mb-4">📋</div>
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
      </PancakeCard>
    </div>
    </>
  )
}