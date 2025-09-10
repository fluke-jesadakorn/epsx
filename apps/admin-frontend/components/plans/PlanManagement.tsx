'use client'

import { useState } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'

interface Plan {
  id: number
  name: string
  planType: 'personal' | 'api'
  basePrice: number
  currentPrice: number
  currency: string
  features: string[]
  affiliateCommissionRate: number
  displayOrder: number
  isActive: boolean
  isHighlighted: boolean
  createdAt: string
  updatedAt: string
  activePromotions: string[]
  effectivePrice: number
}

interface PlanManagementProps {
  plans: Plan[]
  currentUser: any
}

export function PlanManagement({ plans, currentUser }: PlanManagementProps) {
  const [selectedPlan, setSelectedPlan] = useState<Plan | null>(null)
  const [isCreating, setIsCreating] = useState(false)
  const [filterType, setFilterType] = useState<'all' | 'personal' | 'api'>('all')

  const filteredPlans = plans.filter(plan => 
    filterType === 'all' || plan.planType === filterType
  )

  const personalPlans = plans.filter(p => p.planType === 'personal')
  const apiPlans = plans.filter(p => p.planType === 'api')
  const activePlans = plans.filter(p => p.isActive)
  const highlightedPlans = plans.filter(p => p.isHighlighted)

  const totalRevenue = plans.reduce((sum, plan) => sum + plan.effectivePrice, 0)

  return (
    <div className="max-w-7xl mx-auto space-y-8">
      {/* Hero Section */}
      <div className="text-center mb-12">
        <div className="inline-flex items-center gap-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white px-8 py-4 rounded-2xl shadow-xl mb-6">
          <span className="text-3xl">💳</span>
          <h1 className="text-3xl font-bold">Plans Management</h1>
        </div>
        <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
          Manage pricing plans, features, and affiliate commission rates for your platform
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
            <h3 className="text-2xl font-bold mb-4">Create New Plan</h3>
            <p className="text-white/80 mb-6">Add a new pricing plan with custom features and pricing</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              New Plan
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 text-white">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">🎯</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Bulk Actions</h3>
            <p className="text-white/80 mb-6">Update multiple plans, pricing, or features at once</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              Bulk Edit
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-gradient-to-br from-orange-400 via-red-500 to-pink-500 text-white">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">📊</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Analytics</h3>
            <p className="text-white/80 mb-6">View plan performance, conversions, and revenue</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              View Analytics
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
            <div className="text-purple-600 dark:text-purple-400 font-semibold mb-2">Highlighted</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{highlightedPlans.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">Featured plans</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-green-600 dark:text-green-400 font-semibold mb-2">Avg Price</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              ${(totalRevenue / plans.length || 0).toFixed(0)}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">USD per plan</div>
          </div>
        </PancakeCard>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-4 mb-6">
        <button
          onClick={() => setFilterType('all')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterType === 'all'
              ? 'bg-gradient-to-r from-emerald-400 to-green-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          All Plans ({plans.length})
        </button>
        <button
          onClick={() => setFilterType('personal')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterType === 'personal'
              ? 'bg-gradient-to-r from-blue-400 to-purple-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Personal ({personalPlans.length})
        </button>
        <button
          onClick={() => setFilterType('api')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterType === 'api'
              ? 'bg-gradient-to-r from-orange-400 to-red-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          API Plans ({apiPlans.length})
        </button>
      </div>

      {/* Plans List */}
      <PancakeCard className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border border-white/30 overflow-hidden">
        <div className="p-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
              {filterType === 'all' ? 'All Plans' : 
               filterType === 'personal' ? 'Personal Plans' : 'API Plans'}
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
                    plan.planType === 'personal' 
                      ? 'bg-gradient-to-br from-blue-400 to-purple-500'
                      : 'bg-gradient-to-br from-orange-400 to-red-500'
                  }`}>
                    {plan.planType === 'personal' ? '👤' : '🔧'}
                  </div>
                  
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                        {plan.name}
                      </h3>
                      {plan.isHighlighted && (
                        <span className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white text-xs px-3 py-1 rounded-full font-semibold">
                          FEATURED
                        </span>
                      )}
                      {!plan.isActive && (
                        <span className="bg-gray-400 text-white text-xs px-3 py-1 rounded-full font-semibold">
                          INACTIVE
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
                      <span className="font-semibold">
                        ${plan.currentPrice} {plan.currency}
                        {plan.currentPrice !== plan.basePrice && (
                          <span className="line-through ml-2 text-gray-400">
                            ${plan.basePrice}
                          </span>
                        )}
                      </span>
                      <span>•</span>
                      <span>{plan.features.length} features</span>
                      <span>•</span>
                      <span>{plan.affiliateCommissionRate}% commission</span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-4">
                  {plan.activePromotions.length > 0 && (
                    <div className="text-xs bg-pink-100 dark:bg-pink-900/20 text-pink-600 dark:text-pink-400 px-3 py-1 rounded-full">
                      {plan.activePromotions.length} promotion{plan.activePromotions.length > 1 ? 's' : ''}
                    </div>
                  )}
                  
                  <button
                    className={`px-4 py-2 rounded-xl font-semibold transition-all ${
                      plan.isActive
                        ? 'bg-gradient-to-r from-green-400 to-green-500 text-white'
                        : 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300'
                    }`}
                  >
                    {plan.isActive ? 'Active' : 'Inactive'}
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
                {filterType === 'all' 
                  ? 'Start by creating your first plan'
                  : `No ${filterType} plans available. Try switching filters or create a new plan.`
                }
              </p>
            </div>
          )}
        </div>
      </PancakeCard>
    </div>
  )
}