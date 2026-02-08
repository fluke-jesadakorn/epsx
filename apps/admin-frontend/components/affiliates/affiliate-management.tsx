'use client';

import {
  Clock,
  DollarSign,
  Handshake,
  Target,
  Users
} from 'lucide-react';
import { useState } from 'react';

import { PageHeader } from '@/components/shared';

export interface Affiliate {
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

/**
 *
 * @param root0
 * @param root0.affiliates
 * @param root0.currentUser
 */
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
      case 'active': return 'bg-success/10 text-success'
      case 'pending': return 'bg-warning/10 text-warning'
      case 'inactive': return 'bg-muted text-muted-foreground border border-border/50'
      case 'suspended': return 'bg-destructive/10 text-destructive'
      default: return 'bg-muted text-muted-foreground'
    }
  }

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'Elite': return 'bg-primary text-primary-foreground shadow-sm'
      case 'Premium': return 'bg-secondary text-secondary-foreground shadow-sm'
      case 'Standard': return 'bg-muted text-muted-foreground border border-border/50'
      default: return 'bg-muted text-muted-foreground'
    }
  }

  const getPerformanceRating = (conversionRate: number) => {
    if (conversionRate >= 20) { return { rating: 'Excellent', color: 'text-success' } }
    if (conversionRate >= 15) { return { rating: 'Great', color: 'text-primary' } }
    if (conversionRate >= 10) { return { rating: 'Good', color: 'text-warning' } }
    if (conversionRate > 0) { return { rating: 'Average', color: 'text-secondary' } }
    return { rating: 'New', color: 'text-muted-foreground' }
  }

  return (
    <div className="space-y-6 sm:space-y-8">
      <PageHeader
        title="Affiliate Program"
        subtitle="Manage affiliate partners, track referrals, and process commissions"
        icon="Users"
        gradient="purple"
      />
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-primary/10 rounded-full blur-xl animate-pulse" />
        <div className="absolute top-40 right-32 w-24 h-24 bg-secondary/10 rounded-full blur-lg animate-pulse delay-700" />
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-primary/5 rounded-full blur-xl animate-pulse delay-1000" />
      </div>

      <div className="relative max-w-7xl mx-auto">
        {/* Hero Section */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="flex items-center justify-center gap-3 text-3xl sm:text-4xl lg:text-5xl font-bold mb-4">
              <span className="text-primary">
                <Handshake className="w-10 h-10 sm:w-12 sm:h-12" />
              </span>
              <span className="bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                Affiliate Partners
              </span>
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary rounded-full" />
          </div>
          <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
            Manage affiliate partners, track commissions, and optimize your referral program
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mb-8 sm:mb-12">
          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5">
            <div className="relative bg-secondary text-secondary-foreground cursor-pointer rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
              <div className="p-6 sm:p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                  <span className="text-2xl">👥</span>
                </div>
                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Recruit Partners</h3>
                <p className="text-secondary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Invite new affiliates and grow your partner network</p>
                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                  Invite Affiliates
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-success/10 p-0.5">
            <div className="relative bg-success text-success-foreground cursor-pointer rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
              <div className="p-6 sm:p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                  <span className="text-2xl">💰</span>
                </div>
                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Process Payments</h3>
                <p className="text-success-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Pay pending commissions to affiliate partners</p>
                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                  Pay Commissions
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5 sm:col-span-2 lg:col-span-1">
            <div className="relative bg-primary text-primary-foreground cursor-pointer rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
              <div className="p-6 sm:p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                  <span className="text-2xl">📈</span>
                </div>
                <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Performance</h3>
                <p className="text-primary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">View detailed affiliate performance analytics</p>
                <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                  View Analytics
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border border-primary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-primary/10 rounded-xl text-primary">
                <Users className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-primary">{affiliates.length}</div>
              <div className="text-xs sm:text-sm text-muted-foreground">Partners</div>
              <div className="text-xs text-muted-foreground/60">{activeAffiliates.length} active</div>
            </div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border border-success/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-success/10 rounded-xl text-success">
                <DollarSign className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Sales</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-success truncate">
                {totalSales > 999999 ? `$${Math.round(totalSales / 1000000)}M` : `$${Math.round(totalSales / 1000)}K`}
              </div>
              <div className="text-xs sm:text-sm text-muted-foreground">Revenue</div>
              <div className="text-xs text-muted-foreground/60">From referrals</div>
            </div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border border-secondary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-secondary/10 rounded-xl text-secondary">
                <Target className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Earned</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-secondary truncate">
                {totalCommissions > 999999 ? `$${Math.round(totalCommissions / 1000000)}M` : `$${Math.round(totalCommissions / 1000)}K`}
              </div>
              <div className="text-xs sm:text-sm text-muted-foreground">Commissions</div>
              <div className="text-xs text-muted-foreground/60">All time</div>
            </div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-sm border border-warning/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="p-2 bg-warning/10 rounded-xl text-warning">
                <Clock className="w-6 h-6" />
              </div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Pending</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-warning truncate">
                {totalPendingCommissions > 999999 ? `$${Math.round(totalPendingCommissions / 1000000)}M` : `$${Math.round(totalPendingCommissions / 1000)}K`}
              </div>
              <div className="text-xs sm:text-sm text-muted-foreground">To pay</div>
              <div className="text-xs text-muted-foreground/60">Outstanding</div>
            </div>
          </div>
        </div>

        {/* Filter Tabs */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5 mb-6">
          <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4">
            <div className="flex flex-col sm:flex-row gap-2 sm:gap-4">
              <button
                onClick={() => setFilterStatus('all')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'all'
                  ? 'bg-primary text-primary-foreground shadow-sm'
                  : 'bg-muted/50 text-muted-foreground hover:bg-muted'
                  }`}
              >
                All Partners ({affiliates.length})
              </button>
              <button
                onClick={() => setFilterStatus('active')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'active'
                  ? 'bg-success text-success-foreground shadow-sm'
                  : 'bg-muted/50 text-muted-foreground hover:bg-muted'
                  }`}
              >
                Active ({activeAffiliates.length})
              </button>
              <button
                onClick={() => setFilterStatus('pending')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'pending'
                  ? 'bg-warning text-warning-foreground shadow-sm'
                  : 'bg-muted/50 text-muted-foreground hover:bg-muted'
                  }`}
              >
                Pending ({pendingAffiliates.length})
              </button>
              <button
                onClick={() => setFilterStatus('inactive')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'inactive'
                  ? 'bg-muted text-foreground border border-border/50'
                  : 'bg-muted/50 text-muted-foreground hover:bg-muted'
                  }`}
              >
                Inactive ({affiliates.filter(a => a.status === 'inactive').length})
              </button>
            </div>
          </div>
        </div>

        {/* Affiliates List */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-border/50 p-0.5">
          <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
              <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                Affiliate Partners
              </h2>
              <div className="text-sm text-muted-foreground">
                {filteredAffiliates.length} partners
              </div>
            </div>

            {/* Mobile Card Layout */}
            <div className="block sm:hidden space-y-4">
              {filteredAffiliates.map(affiliate => {
                const performance = getPerformanceRating(affiliate.conversionRate)
                return (
                  <div
                    key={affiliate.id}
                    className="p-4 bg-muted/30 rounded-2xl cursor-pointer border border-border/50"
                    onClick={() => setSelectedAffiliate(affiliate)}
                  >
                    <div className="flex items-center gap-3 mb-3">
                      <div className="h-12 w-12 bg-primary/10 rounded-2xl flex items-center justify-center text-primary">
                        <Handshake className="w-6 h-6" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1 flex-wrap">
                          <h3 className="text-lg font-bold text-foreground truncate">{affiliate.name}</h3>
                          <div className={`text-xs px-2 py-1 rounded-full font-semibold ${getStatusColor(affiliate.status)}`}>
                            {affiliate.status.toUpperCase()}
                          </div>
                        </div>
                        <p className="text-sm text-muted-foreground truncate">{affiliate.email}</p>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-3 mb-3">
                      <div className="bg-card/50 rounded-xl p-3 border border-border/50">
                        <div className="text-sm font-medium text-muted-foreground">Sales</div>
                        <div className="text-lg font-bold text-success">{formatCurrency(affiliate.totalSales)}</div>
                      </div>
                      <div className="bg-card/50 rounded-xl p-3 border border-border/50">
                        <div className="text-sm font-medium text-muted-foreground">Commission</div>
                        <div className="text-lg font-bold text-secondary">{affiliate.commissionRate}%</div>
                      </div>
                    </div>

                    {affiliate.status === 'pending' && (
                      <div className="flex gap-2">
                        <button className="bg-success text-success-foreground px-4 py-2 rounded-xl font-semibold text-sm flex-1 min-h-[44px]">
                          Approve
                        </button>
                        <button className="bg-destructive text-destructive-foreground px-4 py-2 rounded-xl font-semibold text-sm flex-1 min-h-[44px]">
                          Reject
                        </button>
                      </div>
                    )}
                  </div>
                )
              })}
            </div>

            {/* Desktop Layout */}
            <div className="hidden sm:block space-y-6">
              {filteredAffiliates.map(affiliate => {
                const performance = getPerformanceRating(affiliate.conversionRate)
                return (
                  <div
                    key={affiliate.id}
                    className="p-6 bg-muted/20 rounded-2xl hover:bg-muted/40 transition-all border border-border/50 cursor-pointer"
                    onClick={() => setSelectedAffiliate(affiliate)}
                  >
                    <div className="flex items-center justify-between mb-4">
                      <div className="flex items-center gap-4">
                        <div className="h-14 w-14 bg-primary/10 rounded-2xl flex items-center justify-center text-primary">
                          <Handshake className="w-8 h-8" />
                        </div>
                        <div>
                          <div className="flex items-center gap-3 mb-1 flex-wrap">
                            <h3 className="text-xl font-bold text-foreground">
                              {affiliate.name}
                            </h3>
                            <code className="bg-muted text-muted-foreground px-3 py-1 rounded-lg text-sm font-mono border border-border/50">
                              {affiliate.affiliateCode}
                            </code>
                            <div className={`text-xs px-3 py-1 rounded-full font-semibold ${getStatusColor(affiliate.status)}`}>
                              {affiliate.status.toUpperCase()}
                            </div>
                            <div className={`text-xs px-3 py-1 rounded-full font-semibold ${getTierColor(affiliate.tier)}`}>
                              {affiliate.tier}
                            </div>
                          </div>
                          <div className="flex items-center gap-3 text-sm text-muted-foreground mb-2 flex-wrap">
                            <span>{affiliate.email}</span>
                            <span className="text-border">•</span>
                            <span>{affiliate.commissionRate}% commission</span>
                            <span className="text-border">•</span>
                            <span>Joined {formatDate(affiliate.joinedAt)}</span>
                          </div>
                          <p className="text-sm text-muted-foreground/80">{affiliate.notes}</p>
                        </div>
                      </div>

                      <div className="text-right">
                        <div className="text-2xl font-bold text-foreground mb-1">
                          {formatCurrency(affiliate.totalSales)}
                        </div>
                        <div className="text-sm text-muted-foreground mb-2">
                          {affiliate.totalReferrals} referrals
                        </div>
                        <div className={`text-sm font-semibold ${performance.color}`}>
                          {performance.rating} ({affiliate.conversionRate.toFixed(1)}%)
                        </div>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                      {/* Commissions */}
                      <div className="bg-card/50 rounded-xl p-4 border border-border/50">
                        <span className="text-sm font-medium text-muted-foreground">Commissions</span>
                        <div className="text-lg font-bold text-success">
                          {formatCurrency(affiliate.totalCommissions)}
                        </div>
                        <div className="text-xs text-muted-foreground/60">
                          {formatCurrency(affiliate.pendingCommissions)} pending
                        </div>
                      </div>

                      {/* Performance */}
                      <div className="bg-card/50 rounded-xl p-4 border border-border/50">
                        <span className="text-sm font-medium text-muted-foreground">Conversion</span>
                        <div className="text-lg font-bold text-secondary">
                          {affiliate.conversionRate.toFixed(1)}%
                        </div>
                        <div className="text-xs text-muted-foreground/60">
                          Avg: {formatCurrency(affiliate.avgOrderValue)}
                        </div>
                      </div>

                      {/* Payment Method */}
                      <div className="bg-card/50 rounded-xl p-4 border border-border/50">
                        <span className="text-sm font-medium text-muted-foreground">Payment</span>
                        <div className="text-lg font-bold text-primary">
                          {affiliate.paymentMethod}
                        </div>
                        <div className="text-xs text-muted-foreground/60 truncate">
                          {affiliate.paymentEmail || 'Bank details'}
                        </div>
                      </div>

                      {/* Last Active */}
                      <div className="bg-card/50 rounded-xl p-4 border border-border/50">
                        <span className="text-sm font-medium text-muted-foreground">Last Active</span>
                        <div className="text-lg font-bold text-warning">
                          {formatDate(affiliate.lastActive)}
                        </div>
                        <div className="text-xs text-muted-foreground/60">
                          Activity status
                        </div>
                      </div>
                    </div>

                    {affiliate.status === 'pending' && (
                      <div className="mt-4 flex flex-col sm:flex-row gap-3">
                        <button className="bg-success text-success-foreground px-6 py-3 rounded-xl font-semibold min-h-[44px]">
                          Approve
                        </button>
                        <button className="bg-destructive text-destructive-foreground px-6 py-3 rounded-xl font-semibold min-h-[44px]">
                          Reject
                        </button>
                      </div>
                    )}

                    {affiliate.status === 'active' && affiliate.pendingCommissions > 0 && (
                      <div className="mt-4">
                        <button className="bg-primary text-primary-foreground px-6 py-3 rounded-xl font-semibold min-h-[44px]">
                          Pay {formatCurrency(affiliate.pendingCommissions)}
                        </button>
                      </div>
                    )}
                  </div>
                )
              })}
            </div>

            {filteredAffiliates.length === 0 && (
              <div className="text-center py-12 sm:py-16">
                <div className="h-20 w-20 bg-gradient-to-br from-indigo-200 to-violet-200 dark:from-indigo-800 dark:to-violet-800 rounded-2xl flex items-center justify-center mx-auto mb-4 text-gray-500 dark:text-gray-300">
                  <Handshake className="w-10 h-10" />
                </div>
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
        </div>
      </div>
    </div>
  )
}