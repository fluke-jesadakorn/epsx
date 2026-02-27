'use client'

import { Tag } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { createPromotionsClient, isApiSuccess } from '@/shared/api/promotions'
import { useSharedAuth } from '@/shared/components/auth'
import { createAdminApiClient } from '@/shared/utils/api-client'

import type { DisplayPromotion } from '@/components/promotions/types'

interface PromotionManagementProps {
  currentUser?: Record<string, unknown>
}

// Module-level helpers
function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

function getDiscountDisplay(p: DisplayPromotion) {
  return p.discountType === 'percentage' ? `${p.discountValue}% OFF` : `$${p.discountValue} OFF`
}

function getUsagePercentage(p: DisplayPromotion) {
  if (p.usageLimit === undefined || p.usageLimit === 0) { return 0 }
  return Math.min((p.currentUsage / p.usageLimit) * 100, 100)
}

const isExpired = (endDate: string) => new Date(endDate) < new Date()
const isUpcoming = (startDate: string) => new Date(startDate) > new Date()

// Sub-components
function UsageBar({ promotion }: { promotion: DisplayPromotion }) {
  return (
    <div className="bg-muted/30 rounded-xl p-3 border border-border/20">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-muted-foreground">Usage</span>
        <span className="text-sm text-muted-foreground/60">{promotion.currentUsage}/{promotion.usageLimit ?? '∞'}</span>
      </div>
      <div className="bg-muted rounded-full h-2">
        <div className="bg-gradient-to-r from-[#ffb237] to-[#ed4b9e] rounded-full h-2 transition-all duration-500" style={{ width: `${getUsagePercentage(promotion)}%` }} />
      </div>
    </div>
  )
}

function PromotionMobileCard({ promotion, onSelect }: { promotion: DisplayPromotion; onSelect: (p: DisplayPromotion) => void }) {
  return (
    <div key={promotion.id} className="p-4 bg-muted/30 rounded-xl cursor-pointer border border-border/20 hover:bg-muted/50 transition-colors" onClick={() => onSelect(promotion)}>
      <div className="flex items-center gap-3 mb-3">
        <div className="h-12 w-12 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-xl flex items-center justify-center border border-[#ffb237]/20 text-xl">🎯</div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1 flex-wrap">
            <h3 className="text-lg font-bold text-foreground truncate">{promotion.name}</h3>
            <div className={`text-xs px-2 py-1 rounded-full font-semibold ${promotion.isActive ? 'bg-success/10 text-success' : 'bg-muted text-muted-foreground'}`}>{promotion.isActive ? 'ACTIVE' : 'INACTIVE'}</div>
          </div>
          <code className="bg-muted/30 text-foreground px-2 py-1 rounded text-sm font-mono border border-border/20">{promotion.code}</code>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-3 mb-3">
        <div className="bg-muted/30 rounded-xl p-3 border border-border/20"><div className="text-sm font-medium text-muted-foreground">Discount</div><div className="text-lg font-bold text-[#ffb237]">{getDiscountDisplay(promotion)}</div></div>
        <div className="bg-muted/30 rounded-xl p-3 border border-border/20"><div className="text-sm font-medium text-muted-foreground">Revenue</div><div className="text-lg font-bold text-success">${Math.round(Number(promotion.totalRevenue) / 1000)}K</div></div>
      </div>
      <UsageBar promotion={promotion} />
    </div>
  )
}

function PromotionDesktopCard({ promotion, onSelect }: { promotion: DisplayPromotion; onSelect: (p: DisplayPromotion) => void }) {
  return (
    <div className="p-6 bg-muted/30 rounded-xl border border-border/20 transition-all hover:bg-muted/50 cursor-pointer" onClick={() => onSelect(promotion)}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <div className="h-14 w-14 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-xl flex items-center justify-center border border-[#ffb237]/20 text-2xl">🎯</div>
          <div>
            <div className="flex items-center gap-3 mb-1 flex-wrap">
              <h3 className="text-xl font-bold text-foreground">{promotion.name}</h3>
              <code className="bg-muted/30 text-foreground px-3 py-1 rounded-lg text-sm font-mono border border-border/20">{promotion.code}</code>
              <div className={`text-xs px-3 py-1 rounded-full font-semibold ${promotion.isActive ? 'bg-success/10 text-success' : 'bg-muted text-muted-foreground'}`}>{promotion.isActive ? 'ACTIVE' : 'INACTIVE'}</div>
            </div>
            <p className="text-muted-foreground mb-2">{promotion.description}</p>
            <div className="flex items-center gap-4 text-sm text-muted-foreground flex-wrap">
              <span className="font-semibold text-[#ffb237]">{getDiscountDisplay(promotion)}</span>
              <span>•</span>
              <span>{formatDate(promotion.startDate)} - {promotion.endDate !== undefined ? formatDate(promotion.endDate) : 'N/A'}</span>
              <span>•</span>
              <span>{promotion.applicablePlans.join(', ')} plans</span>
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold text-foreground mb-1">${Math.round(Number(promotion.totalRevenue)).toLocaleString()}</div>
          <div className="text-sm text-muted-foreground mb-2">{promotion.conversionRate.toFixed(1)}% conversion</div>
          {promotion.endDate !== undefined && isExpired(promotion.endDate) && <div className="text-xs bg-destructive/10 text-destructive px-2 py-1 rounded-full border border-destructive/20 inline-block">EXPIRED</div>}
          {isUpcoming(promotion.startDate) && <div className="text-xs bg-info/10 text-info px-2 py-1 rounded-full border border-info/20 inline-block">UPCOMING</div>}
        </div>
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <UsageBar promotion={promotion} />
        <div className="bg-muted/30 rounded-xl p-4 border border-border/20"><span className="text-sm font-medium text-muted-foreground">Revenue</span><div className="text-lg font-bold text-success">${promotion.totalRevenue.toLocaleString()}</div></div>
        <div className="bg-muted/30 rounded-xl p-4 border border-border/20"><span className="text-sm font-medium text-muted-foreground">Performance</span><div className="text-lg font-bold text-secondary">{promotion.conversionRate.toFixed(1)}%</div></div>
      </div>
    </div>
  )
}

interface PromotionListProps {
  filteredPromotions: DisplayPromotion[];
  filterStatus: 'all' | 'active' | 'inactive';
  onSelect: (p: DisplayPromotion) => void;
}

function PromotionList({ filteredPromotions, filterStatus, onSelect }: PromotionListProps) {
  return (
    <div className="p-4 sm:p-6">
      <div className="block sm:hidden space-y-4">
        {filteredPromotions.map(p => <PromotionMobileCard key={p.id} promotion={p} onSelect={onSelect} />)}
      </div>
      <div className="hidden sm:block space-y-4">
        {filteredPromotions.map(p => <PromotionDesktopCard key={p.id} promotion={p} onSelect={onSelect} />)}
      </div>
      {filteredPromotions.length === 0 && (
        <div className="text-center py-12 sm:py-16">
          <div className="h-20 w-20 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-2xl flex items-center justify-center mx-auto mb-4 border border-[#ffb237]/20"><span className="text-4xl">🎯</span></div>
          <h3 className="text-xl font-semibold text-foreground mb-2">No campaigns found</h3>
          <p className="text-muted-foreground">{filterStatus === 'all' ? 'Start by creating your first promotional campaign' : `No ${filterStatus} campaigns available. Try switching filters or create a new campaign.`}</p>
        </div>
      )}
    </div>
  )
}

function PromotionLoadingState() {
  return (
    <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        {(['a', 'b', 'c'] as const).map(k => <div key={k} className="bg-card border border-border/20 rounded-xl h-64" />)}
      </div>
    </div>
  )
}

interface StatsProps { total: number; active: number; usage: number; revenue: number }
function PromotionStats({ total, active, usage, revenue }: StatsProps) {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
      <div className="bg-card rounded-xl p-4 sm:p-6 shadow-xl border border-border/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="p-2 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-xl text-[#ffb237] border border-[#ffb237]/20"><Tag className="w-5 h-5" /></div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span></div>
        <div className="space-y-1"><div className="text-2xl sm:text-3xl font-bold text-foreground">{total}</div><div className="text-xs sm:text-sm text-muted-foreground">Campaigns</div><div className="text-xs text-muted-foreground/60">All time</div></div>
      </div>
      <div className="bg-card rounded-xl p-4 sm:p-6 shadow-xl border border-border/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="p-2 bg-success/10 rounded-xl text-success"><span className="text-sm">✅</span></div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Active</span></div>
        <div className="space-y-1"><div className="text-2xl sm:text-3xl font-bold text-success">{active}</div><div className="text-xs sm:text-sm text-muted-foreground">Live</div><div className="text-xs text-muted-foreground/60">Running now</div></div>
      </div>
      <div className="bg-card rounded-xl p-4 sm:p-6 shadow-xl border border-border/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="p-2 bg-secondary/10 rounded-xl text-secondary"><span className="text-sm">🎟️</span></div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Usage</span></div>
        <div className="space-y-1"><div className="text-xl sm:text-3xl font-bold text-secondary truncate">{usage > 999999 ? `${Math.round(usage / 1000000)}M` : `${Math.round(usage / 1000)}K`}</div><div className="text-xs sm:text-sm text-muted-foreground">Redemptions</div><div className="text-xs text-muted-foreground/60">Codes used</div></div>
      </div>
      <div className="bg-card rounded-xl p-4 sm:p-6 shadow-xl border border-border/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="p-2 bg-[#ffb237]/10 rounded-xl text-[#ffb237]"><span className="text-sm">💰</span></div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Revenue</span></div>
        <div className="space-y-1"><div className="text-xl sm:text-3xl font-bold text-foreground truncate">${Math.round(Number(revenue) / 1000)}K</div><div className="text-xs sm:text-sm text-muted-foreground">Generated</div><div className="text-xs text-muted-foreground/60">From campaigns</div></div>
      </div>
    </div>
  )
}

export function PromotionManagement({ currentUser }: PromotionManagementProps) {
  const { user: authUser } = useSharedAuth()
  const _user = currentUser ?? authUser
  const [promotions, setPromotions] = useState<DisplayPromotion[]>([])
  const [loading, setLoading] = useState(true)
  const [_selectedPromotion, setSelectedPromotion] = useState<DisplayPromotion | null>(null)
  const [_isCreating, setIsCreating] = useState(false)
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'inactive'>('all')

  const loadPromotions = useCallback(async () => {
    try {
      setLoading(true)
      const apiClient = createAdminApiClient()
      const promotionsClient = createPromotionsClient(apiClient)
      const response = await promotionsClient.getPromotions({ limit: 100 })
      if (isApiSuccess(response)) {
        const promos = response.data.promotions
        setPromotions(promos.map(p => ({
          ...p,
          discountType: p.discountType,
          discountValue: parseFloat(p.discountValue),
          maxDiscountAmount: p.maxDiscountAmount !== undefined ? parseFloat(p.maxDiscountAmount) : null,
          minPurchaseAmount: parseFloat(p.minPurchaseAmount ?? '0'),
          totalRevenue: parseFloat(p.totalRevenue),
        })))
      } else {
        toast({ title: "Error", description: response.error?.message ?? "Failed to load promotions", variant: "destructive" })
      }
    } catch (_error) {
      toast({ title: "Error", description: "Failed to load promotions", variant: "destructive" })
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { void loadPromotions() }, [loadPromotions])

  const filteredPromotions = promotions.filter(p => filterStatus === 'all' || (filterStatus === 'active' && p.isActive) || (filterStatus === 'inactive' && !p.isActive))
  const activePromotions = promotions.filter(p => p.isActive)
  const totalUsage = promotions.reduce((sum, p) => sum + p.currentUsage, 0)
  const totalRevenue = promotions.reduce((sum, p) => sum + (typeof p.totalRevenue === 'number' ? p.totalRevenue : (parseFloat(String(p.totalRevenue)) || 0)), 0)

  if (loading) { return <PromotionLoadingState /> }

  return (
    <div className="space-y-6 sm:space-y-8">
      <div className="relative max-w-7xl mx-auto">
        {/* Action Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mb-8 sm:mb-12">
          <div className="relative overflow-hidden rounded-2xl bg-card border border-border/20 shadow-xl cursor-pointer" onClick={() => setIsCreating(true)}>
            <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
            <div className="p-6 sm:p-8">
              <div className="bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 border border-[#ffb237]/20"><span className="text-xl sm:text-2xl">🏷️</span></div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Create Campaign</h3>
              <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Launch new discount codes and promotional offers</p>
              <div className="bg-gradient-to-r from-[#ffb237] to-[#ed4b9e] text-white rounded-xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center">New Campaign</div>
            </div>
          </div>
          <div className="relative overflow-hidden rounded-2xl bg-card border border-border/20 shadow-xl">
            <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
            <div className="p-6 sm:p-8">
              <div className="bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 border border-[#ffb237]/20"><span className="text-xl sm:text-2xl">📊</span></div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Analytics</h3>
              <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Track campaign performance and conversion rates</p>
              <div className="bg-gradient-to-r from-[#ffb237] to-[#ed4b9e] text-white rounded-xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center">View Reports</div>
            </div>
          </div>
          <div className="relative overflow-hidden rounded-2xl bg-card border border-border/20 shadow-xl sm:col-span-2 lg:col-span-1">
            <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
            <div className="p-6 sm:p-8">
              <div className="bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 border border-[#ffb237]/20"><span className="text-xl sm:text-2xl">⚡</span></div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Quick Actions</h3>
              <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Bulk activate, deactivate, or extend campaigns</p>
              <div className="bg-gradient-to-r from-[#ffb237] to-[#ed4b9e] text-white rounded-xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center">Bulk Edit</div>
            </div>
          </div>
        </div>

        <PromotionStats total={promotions.length} active={activePromotions.length} usage={totalUsage} revenue={totalRevenue} />

        {/* Filter Tabs */}
        <div className="rounded-2xl bg-card border border-border/20 shadow-xl mb-6 overflow-hidden">
          <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
          <div className="p-4">
            <div className="flex flex-col sm:flex-row gap-2 sm:gap-4">
              <button onClick={() => setFilterStatus('all')} className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'all' ? 'bg-gradient-to-r from-[#ffb237] to-[#ed4b9e] text-white shadow-sm' : 'bg-muted/30 text-muted-foreground hover:bg-muted/50'}`}>All Campaigns ({promotions.length})</button>
              <button onClick={() => setFilterStatus('active')} className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'active' ? 'bg-success text-success-foreground shadow-sm' : 'bg-muted/30 text-muted-foreground hover:bg-muted/50'}`}>Active ({activePromotions.length})</button>
              <button onClick={() => setFilterStatus('inactive')} className={`px-4 sm:px-6 py-3 rounded-xl font-semibold text-sm sm:text-base min-h-[44px] transition-all ${filterStatus === 'inactive' ? 'bg-muted text-foreground border border-border/40' : 'bg-muted/30 text-muted-foreground hover:bg-muted/50'}`}>Inactive ({promotions.filter(p => !p.isActive).length})</button>
            </div>
          </div>
        </div>

        {/* Promotions List */}
        <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
          <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
          <div className="flex items-center gap-3 p-5 border-b border-border/20">
            <div className="p-2 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[14px] text-[#ffb237] border border-[#ffb237]/20"><Tag className="w-4 h-4" /></div>
            <div className="flex-1 flex items-center justify-between">
              <h2 className="text-sm font-bold text-foreground uppercase tracking-wide">{filterStatus === 'all' ? 'All Campaigns' : filterStatus === 'active' ? 'Active Campaigns' : 'Inactive Campaigns'}</h2>
              <div className="text-sm text-muted-foreground">{filteredPromotions.length} campaigns</div>
            </div>
          </div>
          <PromotionList filteredPromotions={filteredPromotions} filterStatus={filterStatus} onSelect={setSelectedPromotion} />
        </div>
      </div>
    </div>
  )
}

export default PromotionManagement;
