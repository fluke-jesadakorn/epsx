'use client'

import { useState } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'

interface Promotion {
  id: number
  name: string
  code: string
  discountType: 'percentage' | 'fixed'
  discountValue: number
  maxDiscountAmount: number | null
  minPurchaseAmount: number
  usageLimit: number | null
  currentUsage: number
  isActive: boolean
  startDate: string
  endDate: string
  applicablePlans: string[]
  description: string
  createdAt: string
  updatedAt: string
  totalRevenue: number
  conversionRate: number
}

interface PromotionManagementProps {
  promotions: Promotion[]
  currentUser: any
}

export function PromotionManagement({ promotions, currentUser }: PromotionManagementProps) {
  const [selectedPromotion, setSelectedPromotion] = useState<Promotion | null>(null)
  const [isCreating, setIsCreating] = useState(false)
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'inactive'>('all')

  const filteredPromotions = promotions.filter(promotion => 
    filterStatus === 'all' || 
    (filterStatus === 'active' && promotion.isActive) ||
    (filterStatus === 'inactive' && !promotion.isActive)
  )

  const activePromotions = promotions.filter(p => p.isActive)
  const totalUsage = promotions.reduce((sum, p) => sum + p.currentUsage, 0)
  const totalRevenue = promotions.reduce((sum, p) => sum + p.totalRevenue, 0)
  const avgConversionRate = promotions.reduce((sum, p) => sum + p.conversionRate, 0) / promotions.length

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
  }

  const getDiscountDisplay = (promotion: Promotion) => {
    if (promotion.discountType === 'percentage') {
      return `${promotion.discountValue}% OFF`
    } else {
      return `$${promotion.discountValue} OFF`
    }
  }

  const getUsagePercentage = (promotion: Promotion) => {
    if (!promotion.usageLimit) return 0
    return Math.min((promotion.currentUsage / promotion.usageLimit) * 100, 100)
  }

  const isExpired = (endDate: string) => {
    return new Date(endDate) < new Date()
  }

  const isUpcoming = (startDate: string) => {
    return new Date(startDate) > new Date()
  }

  return (
    <div className="max-w-7xl mx-auto space-y-8">
      {/* Hero Section */}
      <div className="text-center mb-12">
        <div className="inline-flex items-center gap-3 bg-gradient-to-r from-pink-400 to-rose-500 text-white px-8 py-4 rounded-2xl shadow-xl mb-6">
          <span className="text-3xl">🎯</span>
          <h1 className="text-3xl font-bold">Promotions & Campaigns</h1>
        </div>
        <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
          Create and manage discount codes, promotional campaigns, and marketing offers
        </p>
      </div>

      {/* Action Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        <PancakeCard 
          className="bg-gradient-to-br from-pink-400 via-rose-500 to-red-500 text-white cursor-pointer hover:scale-105 transition-transform"
          onClick={() => setIsCreating(true)}
        >
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">🏷️</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Create Campaign</h3>
            <p className="text-white/80 mb-6">Launch new discount codes and promotional offers</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              New Campaign
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-gradient-to-br from-purple-400 via-indigo-500 to-blue-500 text-white">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">📊</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Analytics</h3>
            <p className="text-white/80 mb-6">Track campaign performance and conversion rates</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              View Reports
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-gradient-to-br from-orange-400 via-amber-500 to-yellow-500 text-white">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">⚡</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Quick Actions</h3>
            <p className="text-white/80 mb-6">Bulk activate, deactivate, or extend campaigns</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              Bulk Edit
            </div>
          </div>
        </PancakeCard>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-pink-600 dark:text-pink-400 font-semibold mb-2">Total Campaigns</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{promotions.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">All time</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-green-600 dark:text-green-400 font-semibold mb-2">Active Now</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{activePromotions.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">Currently running</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-blue-600 dark:text-blue-400 font-semibold mb-2">Total Usage</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{totalUsage.toLocaleString()}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">Code redemptions</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-purple-600 dark:text-purple-400 font-semibold mb-2">Revenue</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              ${Math.round(totalRevenue / 1000)}K
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">From campaigns</div>
          </div>
        </PancakeCard>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-4 mb-6">
        <button
          onClick={() => setFilterStatus('all')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'all'
              ? 'bg-gradient-to-r from-pink-400 to-rose-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          All Campaigns ({promotions.length})
        </button>
        <button
          onClick={() => setFilterStatus('active')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'active'
              ? 'bg-gradient-to-r from-green-400 to-emerald-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Active ({activePromotions.length})
        </button>
        <button
          onClick={() => setFilterStatus('inactive')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'inactive'
              ? 'bg-gradient-to-r from-gray-400 to-gray-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Inactive ({promotions.filter(p => !p.isActive).length})
        </button>
      </div>

      {/* Promotions List */}
      <PancakeCard className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border border-white/30 overflow-hidden">
        <div className="p-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-pink-600 to-rose-600 bg-clip-text text-transparent">
              {filterStatus === 'all' ? 'All Campaigns' : 
               filterStatus === 'active' ? 'Active Campaigns' : 'Inactive Campaigns'}
            </h2>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {filteredPromotions.length} campaigns
            </div>
          </div>

          <div className="space-y-6">
            {filteredPromotions.map(promotion => (
              <div
                key={promotion.id}
                className="p-6 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl hover:from-pink-50 hover:to-rose-50 dark:hover:from-gray-600 dark:hover:to-gray-700 transition-all cursor-pointer"
                onClick={() => setSelectedPromotion(promotion)}
              >
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-4">
                    <div className="h-14 w-14 bg-gradient-to-br from-pink-400 to-rose-500 rounded-2xl flex items-center justify-center text-2xl">
                      🎯
                    </div>
                    <div>
                      <div className="flex items-center gap-3 mb-1">
                        <h3 className="text-xl font-bold text-gray-900 dark:text-white">
                          {promotion.name}
                        </h3>
                        <code className="bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-lg text-sm font-mono">
                          {promotion.code}
                        </code>
                        <div className={`text-xs px-3 py-1 rounded-full font-semibold ${
                          promotion.isActive 
                            ? 'bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400'
                            : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
                        }`}>
                          {promotion.isActive ? 'ACTIVE' : 'INACTIVE'}
                        </div>
                      </div>
                      <p className="text-gray-600 dark:text-gray-400 mb-2">{promotion.description}</p>
                      <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                        <span className="font-semibold text-pink-600 dark:text-pink-400">
                          {getDiscountDisplay(promotion)}
                        </span>
                        <span>•</span>
                        <span>{formatDate(promotion.startDate)} - {formatDate(promotion.endDate)}</span>
                        <span>•</span>
                        <span>{promotion.applicablePlans.join(', ')} plans</span>
                      </div>
                    </div>
                  </div>

                  <div className="text-right">
                    <div className="text-2xl font-bold text-gray-900 dark:text-white mb-1">
                      ${Math.round(promotion.totalRevenue).toLocaleString()}
                    </div>
                    <div className="text-sm text-gray-500 dark:text-gray-400 mb-2">
                      {promotion.conversionRate.toFixed(1)}% conversion
                    </div>
                    {isExpired(promotion.endDate) && (
                      <div className="text-xs bg-red-100 dark:bg-red-900/20 text-red-600 dark:text-red-400 px-2 py-1 rounded-full">
                        EXPIRED
                      </div>
                    )}
                    {isUpcoming(promotion.startDate) && (
                      <div className="text-xs bg-blue-100 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 px-2 py-1 rounded-full">
                        UPCOMING
                      </div>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  {/* Usage Progress */}
                  <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Usage</span>
                      <span className="text-sm text-gray-500 dark:text-gray-400">
                        {promotion.currentUsage}/{promotion.usageLimit || '∞'}
                      </span>
                    </div>
                    <div className="bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                      <div 
                        className="bg-gradient-to-r from-pink-400 to-rose-500 rounded-full h-2 transition-all"
                        style={{ width: `${getUsagePercentage(promotion)}%` }}
                      ></div>
                    </div>
                  </div>

                  {/* Revenue */}
                  <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Revenue</span>
                    <div className="text-lg font-bold text-green-600 dark:text-green-400">
                      ${promotion.totalRevenue.toLocaleString()}
                    </div>
                  </div>

                  {/* Performance */}
                  <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Performance</span>
                    <div className="text-lg font-bold text-purple-600 dark:text-purple-400">
                      {promotion.conversionRate.toFixed(1)}%
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {filteredPromotions.length === 0 && (
            <div className="text-center py-12">
              <div className="text-6xl mb-4">🎯</div>
              <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                No campaigns found
              </h3>
              <p className="text-gray-500 dark:text-gray-500">
                {filterStatus === 'all' 
                  ? 'Start by creating your first promotional campaign'
                  : `No ${filterStatus} campaigns available. Try switching filters or create a new campaign.`
                }
              </p>
            </div>
          )}
        </div>
      </PancakeCard>
    </div>
  )
}