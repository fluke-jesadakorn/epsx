'use client'

import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { createPromotionsClient, isApiSuccess, type Promotion } from '@/shared/api/promotions'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface DisplayPromotion extends Omit<Promotion, 'discountValue' | 'maxDiscountAmount' | 'minPurchaseAmount' | 'totalRevenue'> {
  discountValue: number;
  maxDiscountAmount: number | null;
  minPurchaseAmount: number;
  totalRevenue: number;
}

interface PromotionManagementProps {
  currentUser?: any
}

/**
 *
 * @param root0
 * @param root0.currentUser
 */
export function PromotionManagement({ currentUser }: PromotionManagementProps) {
  const { user: authUser } = useSharedAuth()
  const _user = currentUser || authUser
  const [promotions, setPromotions] = useState<DisplayPromotion[]>([])
  const [loading, setLoading] = useState(true)
  const [_selectedPromotion, setSelectedPromotion] = useState<DisplayPromotion | null>(null)
  const [_isCreating, setIsCreating] = useState(false)
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'inactive'>('all')

  useEffect(() => {
    loadPromotions()
  }, [])

  const loadPromotions = async () => {
    try {
      setLoading(true)
      const apiClient = createAdminApiClient()
      const promotionsClient = createPromotionsClient(apiClient)

      const response = await promotionsClient.getPromotions({
        limit: 100,
      })

      if (isApiSuccess(response)) {
        const promos = response.data?.promotions || []
        setPromotions(promos.map(p => ({
          ...p,
          discountType: p.discountType as 'percentage' | 'fixed',
          discountValue: parseFloat(p.discountValue),
          maxDiscountAmount: p.maxDiscountAmount ? parseFloat(p.maxDiscountAmount) : null,
          minPurchaseAmount: parseFloat(p.minPurchaseAmount || '0'),
          totalRevenue: parseFloat(p.totalRevenue),
        })))
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to load promotions",
          variant: "destructive"
        })
      }
    } catch (_error) {
      toast({
        title: "Error",
        description: "Failed to load promotions",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const filteredPromotions = promotions.filter(promotion =>
    filterStatus === 'all' ||
    (filterStatus === 'active' && promotion.isActive) ||
    (filterStatus === 'inactive' && !promotion.isActive)
  )

  const activePromotions = promotions.filter(p => p.isActive)
  const totalUsage = promotions.reduce((sum, p) => sum + p.currentUsage, 0)
  const totalRevenue = promotions.reduce((sum, p) => sum + (typeof p.totalRevenue === 'number' ? p.totalRevenue : parseFloat(String(p.totalRevenue)) || 0), 0)
  const _avgConversionRate = promotions.length > 0
    ? promotions.reduce((sum, p) => sum + p.conversionRate, 0) / promotions.length
    : 0

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
        <div className="text-center mb-12">
          <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6"></div>
          <div className="h-6 bg-muted rounded-full w-64 mx-auto"></div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-card border border-border rounded-3xl h-64"></div>
          ))}
        </div>
      </div>
    )
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
  }

  const getDiscountDisplay = (promotion: DisplayPromotion) => {
    if (promotion.discountType === 'percentage') {
      return `${promotion.discountValue}% OFF`
    } else {
      return `$${promotion.discountValue} OFF`
    }
  }

  const getUsagePercentage = (promotion: DisplayPromotion) => {
    if (!promotion.usageLimit) { return 0 }
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
        <div className="absolute top-20 left-20 w-32 h-32 bg-primary/10 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-secondary/10 rounded-full blur-2xl"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-primary/5 rounded-full blur-3xl"></div>
      </div>

      <div className="relative max-w-7xl mx-auto">
        {/* Hero Section */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold text-foreground mb-4">
              🎯 <span className="bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">Promotions & Campaigns</span>
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary rounded-full animate-ping"></div>
          </div>
          <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
            Create and manage discount codes, promotional campaigns, and marketing offers
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mb-8 sm:mb-12">
          <div
            className="relative group overflow-hidden rounded-2xl sm:rounded-3xl border border-primary/20 bg-primary/5 p-6 sm:p-8 cursor-pointer hover:bg-primary/10 transition-all duration-300 active:scale-[0.98]"
            onClick={() => setIsCreating(true)}
          >
            <div className="bg-primary/20 text-primary rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 group-hover:scale-110 transition-transform">
              <span className="text-xl sm:text-2xl">🏷️</span>
            </div>
            <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Create Campaign</h3>
            <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Launch new discount codes and promotional offers</p>
            <div className="bg-primary text-primary-foreground rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center shadow-lg shadow-primary/20">
              New Campaign
            </div>
          </div>

          <div className="relative group overflow-hidden rounded-2xl sm:rounded-3xl border border-secondary/20 bg-secondary/5 p-6 sm:p-8 cursor-pointer hover:bg-secondary/10 transition-all duration-300 active:scale-[0.98]">
            <div className="bg-secondary/20 text-secondary rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 group-hover:scale-110 transition-transform">
              <span className="text-xl sm:text-2xl">📊</span>
            </div>
            <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Analytics</h3>
            <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Track campaign performance and conversion rates</p>
            <div className="bg-secondary text-secondary-foreground rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center shadow-lg shadow-secondary/20">
              View Reports
            </div>
          </div>

          <div className="relative group overflow-hidden rounded-2xl sm:rounded-3xl border border-warning/20 bg-warning/5 p-6 sm:p-8 cursor-pointer hover:bg-warning/10 transition-all duration-300 active:scale-[0.98] sm:col-span-2 lg:col-span-1">
            <div className="bg-warning/20 text-warning rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 group-hover:scale-110 transition-transform">
              <span className="text-xl sm:text-2xl">⚡</span>
            </div>
            <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Quick Actions</h3>
            <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Bulk activate, deactivate, or extend campaigns</p>
            <div className="bg-warning text-warning-foreground rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center shadow-lg shadow-warning/20">
              Bulk Edit
            </div>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-primary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">🎯</div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-primary">{promotions.length}</div>
              <div className="text-xs sm:text-sm text-foreground/80">Campaigns</div>
              <div className="text-xs text-muted-foreground">All time</div>
            </div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-success/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">✅</div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Active</span>
            </div>
            <div className="space-y-1">
              <div className="text-2xl sm:text-3xl font-bold text-success">{activePromotions.length}</div>
              <div className="text-xs sm:text-sm text-foreground/80">Live</div>
              <div className="text-xs text-muted-foreground">Running now</div>
            </div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-secondary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">🎟️</div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Usage</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-secondary truncate">
                {totalUsage > 999999 ? `${Math.round(totalUsage / 1000000)}M` : `${Math.round(totalUsage / 1000)}K`}
              </div>
              <div className="text-xs sm:text-sm text-foreground/80">Redemptions</div>
              <div className="text-xs text-muted-foreground">Codes used</div>
            </div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-secondary/20">
            <div className="flex items-center justify-between mb-3 sm:mb-4">
              <div className="text-xl sm:text-2xl">💰</div>
              <span className="text-xs sm:text-sm font-medium text-muted-foreground">Revenue</span>
            </div>
            <div className="space-y-1">
              <div className="text-xl sm:text-3xl font-bold text-secondary truncate">
                ${Math.round(Number(totalRevenue) / 1000)}K
              </div>
              <div className="text-xs sm:text-sm text-foreground/80">Generated</div>
              <div className="text-xs text-muted-foreground">From campaigns</div>
            </div>
          </div>
        </div>

        {/* Filter Tabs */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-border/20 p-0.5 mb-6">
          <div className="relative bg-card rounded-2xl sm:rounded-3xl p-4">
            <div className="flex flex-col sm:flex-row gap-2 sm:gap-4">
              <button
                onClick={() => setFilterStatus('all')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'all'
                  ? 'bg-primary text-primary-foreground shadow-lg'
                  : 'bg-muted text-muted-foreground hover:bg-muted/80'
                  }`}
              >
                All Campaigns ({promotions.length})
              </button>
              <button
                onClick={() => setFilterStatus('active')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'active'
                  ? 'bg-success text-success-foreground shadow-lg'
                  : 'bg-muted text-muted-foreground hover:bg-muted/80'
                  }`}
              >
                Active ({activePromotions.length})
              </button>
              <button
                onClick={() => setFilterStatus('inactive')}
                className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'inactive'
                  ? 'bg-secondary text-secondary-foreground shadow-lg'
                  : 'bg-muted text-muted-foreground hover:bg-muted/80'
                  }`}
              >
                Inactive ({promotions.filter(p => !p.isActive).length})
              </button>
            </div>
          </div>
        </div>

        {/* Promotions List */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-border/20 p-0.5">
          <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
              <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                {filterStatus === 'all' ? 'All Campaigns' :
                  filterStatus === 'active' ? 'Active Campaigns' : 'Inactive Campaigns'}
              </h2>
              <div className="text-sm text-muted-foreground">
                {filteredPromotions.length} campaigns
              </div>
            </div>

            {/* Mobile Card Layout */}
            <div className="block sm:hidden space-y-4">
              {filteredPromotions.map(promotion => (
                <div
                  key={promotion.id}
                  className="p-4 bg-muted/40 rounded-2xl cursor-pointer border border-border/50 hover:bg-muted transition-colors"
                  onClick={() => setSelectedPromotion(promotion)}
                >
                  <div className="flex items-center gap-3 mb-3">
                    <div className="h-12 w-12 bg-primary text-primary-foreground rounded-2xl flex items-center justify-center text-xl">
                      🎯
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <h3 className="text-lg font-bold text-foreground truncate">{promotion.name}</h3>
                        <div className={`text-xs px-2 py-1 rounded-full font-semibold ${promotion.isActive
                          ? 'bg-success/10 text-success'
                          : 'bg-muted text-muted-foreground'
                          }`}>
                          {promotion.isActive ? 'ACTIVE' : 'INACTIVE'}
                        </div>
                      </div>
                      <code className="bg-muted text-foreground px-2 py-1 rounded text-sm font-mono border border-border/50">
                        {promotion.code}
                      </code>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-3 mb-3">
                    <div className="bg-muted/50 rounded-xl p-3 border border-border/50">
                      <div className="text-sm font-medium text-muted-foreground">Discount</div>
                      <div className="text-lg font-bold text-primary">{getDiscountDisplay(promotion)}</div>
                    </div>
                    <div className="bg-muted/50 rounded-xl p-3 border border-border/50">
                      <div className="text-sm font-medium text-muted-foreground">Revenue</div>
                      <div className="text-lg font-bold text-success">${Math.round(Number(promotion.totalRevenue) / 1000)}K</div>
                    </div>
                  </div>

                  {/* Usage Progress for Mobile */}
                  <div className="bg-muted/50 rounded-xl p-3 border border-border/50">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-muted-foreground">Usage</span>
                      <span className="text-sm text-muted-foreground/60">
                        {promotion.currentUsage}/{promotion.usageLimit || '∞'}
                      </span>
                    </div>
                    <div className="bg-muted rounded-full h-2">
                      <div
                        className="bg-primary rounded-full h-2 transition-all duration-500"
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
                  className="p-6 bg-muted/30 rounded-2xl border border-border transition-all hover:bg-muted/50 cursor-pointer"
                  onClick={() => setSelectedPromotion(promotion)}
                >
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-4">
                      <div className="h-14 w-14 bg-primary text-primary-foreground rounded-2xl flex items-center justify-center text-2xl shadow-lg shadow-primary/20">
                        🎯
                      </div>
                      <div>
                        <div className="flex items-center gap-3 mb-1 flex-wrap">
                          <h3 className="text-xl font-bold text-foreground">
                            {promotion.name}
                          </h3>
                          <code className="bg-muted text-foreground px-3 py-1 rounded-lg text-sm font-mono border border-border/50">
                            {promotion.code}
                          </code>
                          <div className={`text-xs px-3 py-1 rounded-full font-semibold ${promotion.isActive
                            ? 'bg-success/10 text-success'
                            : 'bg-muted text-muted-foreground'
                            }`}>
                            {promotion.isActive ? 'ACTIVE' : 'INACTIVE'}
                          </div>
                        </div>
                        <p className="text-muted-foreground mb-2">{promotion.description}</p>
                        <div className="flex items-center gap-4 text-sm text-muted-foreground flex-wrap">
                          <span className="font-semibold text-primary">
                            {getDiscountDisplay(promotion)}
                          </span>
                          <span>•</span>
                          <span>{formatDate(promotion.startDate || '')} - {formatDate(promotion.endDate || '')}</span>
                          <span>•</span>
                          <span>{promotion.applicablePlans.join(', ')} plans</span>
                        </div>
                      </div>
                    </div>

                    <div className="text-right">
                      <div className="text-2xl font-bold text-foreground mb-1">
                        ${Math.round(Number(promotion.totalRevenue)).toLocaleString()}
                      </div>
                      <div className="text-sm text-muted-foreground mb-2">
                        {promotion.conversionRate.toFixed(1)}% conversion
                      </div>
                      {isExpired(promotion.endDate || '') && (
                        <div className="text-xs bg-destructive/10 text-destructive px-2 py-1 rounded-full border border-destructive/20 inline-block">
                          EXPIRED
                        </div>
                      )}
                      {isUpcoming(promotion.startDate || '') && (
                        <div className="text-xs bg-info/10 text-info px-2 py-1 rounded-full border border-info/20 inline-block">
                          UPCOMING
                        </div>
                      )}
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {/* Usage Progress */}
                    <div className="bg-muted/50 rounded-xl p-4 border border-border/50">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium text-muted-foreground">Usage</span>
                        <span className="text-sm text-muted-foreground/60">
                          {promotion.currentUsage}/{promotion.usageLimit || '∞'}
                        </span>
                      </div>
                      <div className="bg-muted rounded-full h-2">
                        <div
                          className="bg-primary rounded-full h-2 transition-all duration-500"
                          style={{ width: `${getUsagePercentage(promotion)}%` }}
                        ></div>
                      </div>
                    </div>

                    {/* Revenue */}
                    <div className="bg-muted/50 rounded-xl p-4 border border-border/50">
                      <span className="text-sm font-medium text-muted-foreground">Revenue</span>
                      <div className="text-lg font-bold text-success">
                        ${promotion.totalRevenue.toLocaleString()}
                      </div>
                    </div>

                    {/* Performance */}
                    <div className="bg-muted/50 rounded-xl p-4 border border-border/50">
                      <span className="text-sm font-medium text-muted-foreground">Performance</span>
                      <div className="text-lg font-bold text-secondary">
                        {promotion.conversionRate.toFixed(1)}%
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {filteredPromotions.length === 0 && (
              <div className="text-center py-12 sm:py-16">
                <div className="h-20 w-20 bg-muted rounded-2xl flex items-center justify-center mx-auto mb-4 border border-border">
                  <span className="text-4xl">🎯</span>
                </div>
                <h3 className="text-xl font-semibold text-muted-foreground mb-2">
                  No campaigns found
                </h3>
                <p className="text-muted-foreground/60">
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