'use client'

import { useState } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'

interface Affiliate {
  id: number
  name: string
  email: string
  affiliateCode: string
  status: 'active' | 'pending' | 'inactive' | 'suspended'
  commissionRate: number
  tier: 'Standard' | 'Premium' | 'Elite'
  totalReferrals: number
  totalSales: number
  totalCommissions: number
  pendingCommissions: number
  paidCommissions: number
  conversionRate: number
  avgOrderValue: number
  paymentMethod: string
  paymentEmail: string | null
  joinedAt: string
  lastActive: string
  approvedAt: string | null
  notes: string
}

interface AffiliateManagementProps {
  affiliates: Affiliate[]
  currentUser: any
}

export function AffiliateManagement({ affiliates, currentUser }: AffiliateManagementProps) {
  const [selectedAffiliate, setSelectedAffiliate] = useState<Affiliate | null>(null)
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'pending' | 'inactive'>('all')

  const filteredAffiliates = affiliates.filter(affiliate => 
    filterStatus === 'all' || affiliate.status === filterStatus
  )

  const activeAffiliates = affiliates.filter(a => a.status === 'active')
  const pendingAffiliates = affiliates.filter(a => a.status === 'pending')
  const totalCommissions = affiliates.reduce((sum, a) => sum + a.totalCommissions, 0)
  const totalSales = affiliates.reduce((sum, a) => sum + a.totalSales, 0)
  const totalPendingCommissions = affiliates.reduce((sum, a) => sum + a.pendingCommissions, 0)

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
  }

  const formatCurrency = (amount: number) => {
    return `$${amount.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-100 dark:bg-green-900/20 text-green-600 dark:text-green-400'
      case 'pending': return 'bg-yellow-100 dark:bg-yellow-900/20 text-yellow-600 dark:text-yellow-400'
      case 'inactive': return 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
      case 'suspended': return 'bg-red-100 dark:bg-red-900/20 text-red-600 dark:text-red-400'
      default: return 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
    }
  }

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'Elite': return 'bg-gradient-to-r from-purple-400 to-pink-500 text-white'
      case 'Premium': return 'bg-gradient-to-r from-blue-400 to-indigo-500 text-white'
      case 'Standard': return 'bg-gradient-to-r from-gray-400 to-gray-500 text-white'
      default: return 'bg-gray-400 text-white'
    }
  }

  const getPerformanceRating = (conversionRate: number) => {
    if (conversionRate >= 20) return { rating: 'Excellent', color: 'text-green-600 dark:text-green-400' }
    if (conversionRate >= 15) return { rating: 'Great', color: 'text-blue-600 dark:text-blue-400' }
    if (conversionRate >= 10) return { rating: 'Good', color: 'text-yellow-600 dark:text-yellow-400' }
    if (conversionRate > 0) return { rating: 'Average', color: 'text-orange-600 dark:text-orange-400' }
    return { rating: 'New', color: 'text-gray-600 dark:text-gray-400' }
  }

  return (
    <div className="max-w-7xl mx-auto space-y-8">
      {/* Hero Section */}
      <div className="text-center mb-12">
        <div className="inline-flex items-center gap-3 bg-gradient-to-r from-indigo-400 to-violet-500 text-white px-8 py-4 rounded-2xl shadow-xl mb-6">
          <span className="text-3xl">🤝</span>
          <h1 className="text-3xl font-bold">Affiliate Partners</h1>
        </div>
        <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
          Manage affiliate partners, track commissions, and optimize your referral program
        </p>
      </div>

      {/* Action Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        <PancakeCard className="bg-gradient-to-br from-indigo-400 via-violet-500 to-purple-500 text-white cursor-pointer hover:scale-105 transition-transform">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">👥</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Recruit Partners</h3>
            <p className="text-white/80 mb-6">Invite new affiliates and grow your partner network</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              Invite Affiliates
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-gradient-to-br from-green-400 via-emerald-500 to-teal-500 text-white cursor-pointer hover:scale-105 transition-transform">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">💰</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Process Payments</h3>
            <p className="text-white/80 mb-6">Pay pending commissions to affiliate partners</p>
            <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
              Pay Commissions
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-gradient-to-br from-orange-400 via-red-500 to-pink-500 text-white cursor-pointer hover:scale-105 transition-transform">
          <div className="p-8">
            <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
              <span className="text-2xl">📈</span>
            </div>
            <h3 className="text-2xl font-bold mb-4">Performance</h3>
            <p className="text-white/80 mb-6">View detailed affiliate performance analytics</p>
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
            <div className="text-indigo-600 dark:text-indigo-400 font-semibold mb-2">Total Partners</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{affiliates.length}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {activeAffiliates.length} active
            </div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-green-600 dark:text-green-400 font-semibold mb-2">Total Sales</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              {formatCurrency(totalSales)}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">From referrals</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-purple-600 dark:text-purple-400 font-semibold mb-2">Commissions</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              {formatCurrency(totalCommissions)}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">All time</div>
          </div>
        </PancakeCard>

        <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
          <div className="p-6">
            <div className="text-orange-600 dark:text-orange-400 font-semibold mb-2">Pending</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">
              {formatCurrency(totalPendingCommissions)}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">To pay out</div>
          </div>
        </PancakeCard>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-4 mb-6">
        <button
          onClick={() => setFilterStatus('all')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'all'
              ? 'bg-gradient-to-r from-indigo-400 to-violet-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          All Partners ({affiliates.length})
        </button>
        <button
          onClick={() => setFilterStatus('active')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'active'
              ? 'bg-gradient-to-r from-green-400 to-emerald-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Active ({activeAffiliates.length})
        </button>
        <button
          onClick={() => setFilterStatus('pending')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'pending'
              ? 'bg-gradient-to-r from-yellow-400 to-orange-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Pending ({pendingAffiliates.length})
        </button>
        <button
          onClick={() => setFilterStatus('inactive')}
          className={`px-6 py-3 rounded-xl font-semibold transition-all ${
            filterStatus === 'inactive'
              ? 'bg-gradient-to-r from-gray-400 to-gray-500 text-white shadow-lg'
              : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
          }`}
        >
          Inactive ({affiliates.filter(a => a.status === 'inactive').length})
        </button>
      </div>

      {/* Affiliates List */}
      <PancakeCard className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border border-white/30 overflow-hidden">
        <div className="p-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-indigo-600 to-violet-600 bg-clip-text text-transparent">
              Affiliate Partners
            </h2>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {filteredAffiliates.length} partners
            </div>
          </div>

          <div className="space-y-6">
            {filteredAffiliates.map(affiliate => {
              const performance = getPerformanceRating(affiliate.conversionRate)
              return (
                <div
                  key={affiliate.id}
                  className="p-6 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl hover:from-indigo-50 hover:to-violet-50 dark:hover:from-gray-600 dark:hover:to-gray-700 transition-all cursor-pointer"
                  onClick={() => setSelectedAffiliate(affiliate)}
                >
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-4">
                      <div className="h-14 w-14 bg-gradient-to-br from-indigo-400 to-violet-500 rounded-2xl flex items-center justify-center text-2xl">
                        🤝
                      </div>
                      <div>
                        <div className="flex items-center gap-3 mb-1">
                          <h3 className="text-xl font-bold text-gray-900 dark:text-white">
                            {affiliate.name}
                          </h3>
                          <code className="bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-lg text-sm font-mono">
                            {affiliate.affiliateCode}
                          </code>
                          <div className={`text-xs px-3 py-1 rounded-full font-semibold ${getStatusColor(affiliate.status)}`}>
                            {affiliate.status.toUpperCase()}
                          </div>
                          <div className={`text-xs px-3 py-1 rounded-full font-semibold ${getTierColor(affiliate.tier)}`}>
                            {affiliate.tier}
                          </div>
                        </div>
                        <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400 mb-2">
                          <span>{affiliate.email}</span>
                          <span>•</span>
                          <span>{affiliate.commissionRate}% commission</span>
                          <span>•</span>
                          <span>Joined {formatDate(affiliate.joinedAt)}</span>
                        </div>
                        <p className="text-sm text-gray-600 dark:text-gray-400">{affiliate.notes}</p>
                      </div>
                    </div>

                    <div className="text-right">
                      <div className="text-2xl font-bold text-gray-900 dark:text-white mb-1">
                        {formatCurrency(affiliate.totalSales)}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400 mb-2">
                        {affiliate.totalReferrals} referrals
                      </div>
                      <div className={`text-sm font-semibold ${performance.color}`}>
                        {performance.rating} ({affiliate.conversionRate.toFixed(1)}%)
                      </div>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                    {/* Commissions */}
                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Commissions</span>
                      <div className="text-lg font-bold text-green-600 dark:text-green-400">
                        {formatCurrency(affiliate.totalCommissions)}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        {formatCurrency(affiliate.pendingCommissions)} pending
                      </div>
                    </div>

                    {/* Performance */}
                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Conversion</span>
                      <div className="text-lg font-bold text-purple-600 dark:text-purple-400">
                        {affiliate.conversionRate.toFixed(1)}%
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        Avg: {formatCurrency(affiliate.avgOrderValue)}
                      </div>
                    </div>

                    {/* Payment Method */}
                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Payment</span>
                      <div className="text-lg font-bold text-blue-600 dark:text-blue-400">
                        {affiliate.paymentMethod}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                        {affiliate.paymentEmail || 'Bank details'}
                      </div>
                    </div>

                    {/* Last Active */}
                    <div className="bg-white/50 dark:bg-gray-600/30 rounded-xl p-4">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Last Active</span>
                      <div className="text-lg font-bold text-orange-600 dark:text-orange-400">
                        {formatDate(affiliate.lastActive)}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        Activity status
                      </div>
                    </div>
                  </div>

                  {affiliate.status === 'pending' && (
                    <div className="mt-4 flex gap-3">
                      <button className="bg-gradient-to-r from-green-400 to-emerald-500 text-white px-6 py-2 rounded-xl font-semibold hover:scale-105 transition-transform">
                        Approve
                      </button>
                      <button className="bg-gradient-to-r from-red-400 to-red-500 text-white px-6 py-2 rounded-xl font-semibold hover:scale-105 transition-transform">
                        Reject
                      </button>
                    </div>
                  )}

                  {affiliate.status === 'active' && affiliate.pendingCommissions > 0 && (
                    <div className="mt-4">
                      <button className="bg-gradient-to-r from-blue-400 to-indigo-500 text-white px-6 py-2 rounded-xl font-semibold hover:scale-105 transition-transform">
                        Pay {formatCurrency(affiliate.pendingCommissions)}
                      </button>
                    </div>
                  )}
                </div>
              )
            })}
          </div>

          {filteredAffiliates.length === 0 && (
            <div className="text-center py-12">
              <div className="text-6xl mb-4">🤝</div>
              <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                No affiliates found
              </h3>
              <p className="text-gray-500 dark:text-gray-500">
                {filterStatus === 'all' 
                  ? 'Start by recruiting your first affiliate partners'
                  : `No ${filterStatus} affiliates available. Try switching filters or recruit new partners.`
                }
              </p>
            </div>
          )}
        </div>
      </PancakeCard>
    </div>
  )
}