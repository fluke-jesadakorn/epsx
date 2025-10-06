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

/**
 *
 * @param root0
 * @param root0.promotions
 * @param root0.currentUser
 */
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
    if (!promotion.usageLimit) {return 0}
    return Math.min((promotion.currentUsage / promotion.usageLimit) * 100, 100)
  }

  const isExpired = (endDate: string) => {
    return new Date(endDate) < new Date()
  }

  const isUpcoming = (startDate: string) => {
    return new Date(startDate) > new Date()
  }

  return (
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
              🎯 Promotions & Campaigns
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Create and manage discount codes, promotional campaigns, and marketing offers
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mb-8 sm:mb-12">
          <div 
            className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-pink-400/20 via-rose-500/20 to-red-500/20 p-0.5 cursor-pointer"
            onClick={() => setIsCreating(true)}
          >
            <div className="relative bg-gradient-to-br from-pink-400 via-rose-500 to-red-500 text-white rounded-2xl sm:rounded-3xl">
              <div className="p-6 sm:p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                  <span className="text-xl sm:text-2xl">🏷️</span>
                </div>
                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Create Campaign</h3>
                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Launch new discount codes and promotional offers</p>
                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                  New Campaign
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-indigo-500/20 to-blue-500/20 p-0.5">
            <div className="relative bg-gradient-to-br from-purple-400 via-indigo-500 to-blue-500 text-white rounded-2xl sm:rounded-3xl">
              <div className="p-6 sm:p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                  <span className="text-xl sm:text-2xl">📊</span>
                </div>
                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Analytics</h3>
                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Track campaign performance and conversion rates</p>
                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                  View Reports
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-orange-400/20 via-amber-500/20 to-yellow-500/20 p-0.5 sm:col-span-2 lg:col-span-1">
            <div className="relative bg-gradient-to-br from-orange-400 via-amber-500 to-yellow-500 text-white rounded-2xl sm:rounded-3xl">
              <div className="p-6 sm:p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                  <span className="text-xl sm:text-2xl">⚡</span>
                </div>
                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Quick Actions</h3>
                <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Bulk activate, deactivate, or extend campaigns</p>
                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                  Bulk Edit
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-pink-300/50 dark:border-pink-700/50">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">🎯</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-pink-600 dark:text-pink-400">{promotions.length}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Campaigns</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">All time</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">✅</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Active</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-green-600 dark:text-green-400">{activePromotions.length}</div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Live</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Running now</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">🎟️</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Usage</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-blue-600 dark:text-blue-400 truncate">
                {totalUsage > 999999 ? `${Math.round(totalUsage/1000000)}M` : `${Math.round(totalUsage/1000)}K`}
              </div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Redemptions</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">Codes used</div>
            </div>
          </div>

          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">💰</div>
              <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Revenue</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-purple-600 dark:text-purple-400 truncate">
                ${Math.round(totalRevenue / 1000)}K
              </div>
              <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Generated</div>
              <div className="text-xs text-gray-500 dark:text-gray-400">From campaigns</div>
            </div>
          </div>
        </div>

        {/* Filter Tabs */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5 mb-6">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4">
            <div className="flex flex-col sm:flex-row gap-2 sm:gap-4">
              <button
                onClick={() => setFilterStatus('all')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${
                  filterStatus === 'all'
                    ? 'bg-gradient-to-r from-pink-400 to-rose-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                All Campaigns ({promotions.length})
              </button>
              <button
                onClick={() => setFilterStatus('active')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${
                  filterStatus === 'active'
                    ? 'bg-gradient-to-r from-green-400 to-emerald-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                Active ({activePromotions.length})
              </button>
              <button
                onClick={() => setFilterStatus('inactive')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] ${
                  filterStatus === 'inactive'
                    ? 'bg-gradient-to-r from-gray-400 to-gray-500 text-white shadow-lg'
                    : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                Inactive ({promotions.filter(p => !p.isActive).length})
              </button>
            </div>
          </div>
        </div>

        {/* Promotions List */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-pink-400/20 via-rose-400/20 to-red-400/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
              <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-pink-600 via-rose-600 to-red-600 bg-clip-text text-transparent">
                {filterStatus === 'all' ? 'All Campaigns' : 
                 filterStatus === 'active' ? 'Active Campaigns' : 'Inactive Campaigns'}
              </h2>
              <div className="text-sm text-gray-500 dark:text-gray-400">
                {filteredPromotions.length} campaigns
              </div>
            </div>

            {/* Mobile Card Layout */}
            <div className="block sm:hidden space-y-4">
              {filteredPromotions.map(promotion => (
                <div
                  key={promotion.id}
                  className="p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl cursor-pointer"
                  onClick={() => setSelectedPromotion(promotion)}
                >
                  <div className="flex items-center gap-3 mb-3">
                    <div className="h-12 w-12 bg-gradient-to-br from-pink-400 to-rose-500 rounded-2xl flex items-center justify-center text-xl">
                      🎯
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <h3 className="text-lg font-bold text-gray-900 dark:text-white truncate">{promotion.name}</h3>
                        <div className={`text-xs px-2 py-1 rounded-full font-semibold ${
                          promotion.isActive 
                            ? 'bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400'
                            : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
                        }`}>
                          {promotion.isActive ? 'ACTIVE' : 'INACTIVE'}
                        </div>
                      </div>
                      <code className="bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 px-2 py-1 rounded text-sm font-mono">
                        {promotion.code}
                      </code>
                    </div>
                  </div>
                  
                  <div className="grid grid-cols-2 gap-3 mb-3">
                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                      <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Discount</div>
                      <div className="text-lg font-bold text-pink-600 dark:text-pink-400">{getDiscountDisplay(promotion)}</div>
                    </div>
                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                      <div className="text-sm font-medium text-gray-700 dark:text-gray-300">Revenue</div>
                      <div className="text-lg font-bold text-green-600 dark:text-green-400">${Math.round(promotion.totalRevenue / 1000)}K</div>
                    </div>
                  </div>
                  
                  {/* Usage Progress for Mobile */}
                  <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-3">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Usage</span>
                      <span className="text-sm text-gray-500 dark:text-gray-400">
                        {promotion.currentUsage}/{promotion.usageLimit || '∞'}
                      </span>
                    </div>
                    <div className="bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                      <div 
                        className="bg-gradient-to-r from-pink-400 to-rose-500 rounded-full h-2"
                        style={{ width: `${getUsagePercentage(promotion)}%` }}
                      ></div>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Desktop Layout */}
            <div className="hidden sm:block space-y-6">
              {filteredPromotions.map(promotion => (
                <div
                  key={promotion.id}
                  className="p-6 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl hover:from-pink-50 hover:to-rose-50 dark:hover:from-gray-600 dark:hover:to-gray-700 cursor-pointer"
                  onClick={() => setSelectedPromotion(promotion)}
                >
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-4">
                      <div className="h-14 w-14 bg-gradient-to-br from-pink-400 to-rose-500 rounded-2xl flex items-center justify-center text-2xl">
                        🎯
                      </div>
                      <div>
                        <div className="flex items-center gap-3 mb-1 flex-wrap">
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
                        <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400 flex-wrap">
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

                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
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
                          className="bg-gradient-to-r from-pink-400 to-rose-500 rounded-full h-2"
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
              <div className="text-center py-12 sm:py-16">
                <div className="h-20 w-20 bg-gradient-to-br from-pink-200 to-rose-200 dark:from-pink-800 dark:to-rose-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                  <span className="text-4xl">🎯</span>
                </div>
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
        </div>
      </div>
    </div>
  )
}

export default PromotionManagement;