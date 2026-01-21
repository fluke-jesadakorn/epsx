/**
 * PromotionSection Component
 * 
 * Compact horizontal card layout for promotions:
 * - Shows active promotions as small cards in a horizontal scroll
 * - Inline create button
 * - Quick stats and usage progress
 */
'use client';

import { useState, useCallback } from 'react';
import { Gift, Plus, RefreshCw, TrendingUp, Ticket, ChevronRight } from 'lucide-react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { createPromotionsClient, isApiSuccess, type Promotion } from '@/shared/api/promotions';
import { createAdminApiClient } from '@/shared/utils/api-client';
import type { DisplayPromotion } from '@/lib/data/access-management';

interface PromotionSectionProps {
  initialPromotions: DisplayPromotion[];
  className?: string;
}

function getUsagePercentage(promo: DisplayPromotion): number {
  if (!promo.usageLimit) return 0;
  return Math.min((promo.currentUsage / promo.usageLimit) * 100, 100);
}

function getDiscountDisplay(promo: DisplayPromotion): string {
  if (promo.discountType === 'percentage') {
    return `${promo.discountValue}%`;
  }
  return `$${promo.discountValue}`;
}

function isExpired(endDate?: string): boolean {
  if (!endDate) return false;
  return new Date(endDate) < new Date();
}

function isUpcoming(startDate: string): boolean {
  return new Date(startDate) > new Date();
}

export function PromotionSection({ initialPromotions, className }: PromotionSectionProps) {
  const [promotions, setPromotions] = useState<DisplayPromotion[]>(initialPromotions);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [showAll, setShowAll] = useState(false);

  // Stats
  const activePromotions = promotions.filter(p => p.isActive && !isExpired(p.endDate));
  const totalUsage = promotions.reduce((sum, p) => sum + p.currentUsage, 0);
  const totalRevenue = promotions.reduce((sum, p) => sum + p.totalRevenue, 0);

  // Displayed promotions (show first 6 unless expanded)
  const displayedPromotions = showAll ? promotions : promotions.slice(0, 6);

  // Refresh handler
  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      const apiClient = createAdminApiClient();
      const promotionsClient = createPromotionsClient(apiClient);
      const response = await promotionsClient.getPromotions({ limit: 100 });

      if (isApiSuccess(response)) {
        const promos = response.data?.promotions || [];
        setPromotions(promos.map((p: Promotion) => ({
          id: p.id,
          name: p.name,
          code: p.code,
          description: p.description,
          discountType: p.discountType,
          discountValue: parseFloat(p.discountValue),
          maxDiscountAmount: p.maxDiscountAmount ? parseFloat(p.maxDiscountAmount) : null,
          minPurchaseAmount: parseFloat(p.minPurchaseAmount || '0'),
          usageLimit: p.usageLimit,
          currentUsage: p.currentUsage,
          isActive: p.isActive,
          startDate: p.startDate,
          endDate: p.endDate,
          applicablePlans: p.applicablePlans,
          totalRevenue: parseFloat(p.totalRevenue),
          conversionRate: p.conversionRate,
        })));
        toast.success('Promotions refreshed');
      }
    } catch (error) {
      console.error('Failed to refresh promotions:', error);
      toast.error('Failed to refresh promotions');
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  return (
    <div className={cn('space-y-4', className)}>
      {/* Section Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-xl bg-success/10">
            <Gift className="h-5 w-5 text-success" />
          </div>
          <div>
            <h2 className="text-lg font-bold text-foreground">Promotions</h2>
            <p className="text-xs text-muted-foreground">
              {activePromotions.length} active • {totalUsage.toLocaleString()} redemptions
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRefresh}
            disabled={isRefreshing}
            className="gap-1 text-xs"
          >
            <RefreshCw className={cn('h-3 w-3', isRefreshing && 'animate-spin')} />
          </Button>
          <Button
            size="sm"
            className="gap-1 text-xs bg-success hover:bg-success/90"
            onClick={() => toast.info('Create promotion modal coming soon')}
          >
            <Plus className="h-3 w-3" />
            New
          </Button>
        </div>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-3 gap-3">
        <div className="bg-card rounded-xl border border-border p-3">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
            <Ticket className="h-3 w-3" />
            Campaigns
          </div>
          <div className="text-lg font-bold text-foreground">{promotions.length}</div>
        </div>
        <div className="bg-card rounded-xl border border-border p-3">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
            <Gift className="h-3 w-3 text-success" />
            Active
          </div>
          <div className="text-lg font-bold text-success">{activePromotions.length}</div>
        </div>
        <div className="bg-card rounded-xl border border-border p-3">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
            <TrendingUp className="h-3 w-3 text-primary" />
            Revenue
          </div>
          <div className="text-lg font-bold text-primary">
            ${totalRevenue >= 1000 ? `${(totalRevenue / 1000).toFixed(1)}K` : totalRevenue.toFixed(0)}
          </div>
        </div>
      </div>

      {/* Promotion Cards - Horizontal Scroll */}
      {promotions.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground bg-card rounded-xl border border-border">
          <Gift className="h-8 w-8 mx-auto mb-2 opacity-30" />
          <p className="text-sm">No promotions yet</p>
          <p className="text-xs mt-1">Create your first promotional campaign</p>
        </div>
      ) : (
        <div className="relative">
          <div className="flex gap-3 overflow-x-auto pb-2 scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent">
            {displayedPromotions.map((promo) => {
              const expired = isExpired(promo.endDate);
              const upcoming = isUpcoming(promo.startDate);
              const usagePercent = getUsagePercentage(promo);

              return (
                <div
                  key={promo.id}
                  className={cn(
                    'flex-shrink-0 w-48 bg-card rounded-xl border p-4 transition-all duration-200',
                    'hover:shadow-lg hover:border-success/30 hover:scale-[1.02] cursor-pointer',
                    expired && 'opacity-60',
                    promo.isActive && !expired ? 'border-success/20' : 'border-border'
                  )}
                  onClick={() => toast.info(`View promotion: ${promo.name}`)}
                >
                  {/* Header */}
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1 min-w-0">
                      <code className="text-xs font-mono font-bold text-foreground bg-muted px-1.5 py-0.5 rounded">
                        {promo.code}
                      </code>
                    </div>
                    {promo.isActive && !expired && (
                      <div className="w-2 h-2 rounded-full bg-success animate-pulse" />
                    )}
                  </div>

                  {/* Name */}
                  <h3 className="text-sm font-semibold text-foreground truncate mb-1">
                    {promo.name}
                  </h3>

                  {/* Discount */}
                  <div className="text-xl font-bold text-success mb-2">
                    {getDiscountDisplay(promo)} <span className="text-xs font-normal text-muted-foreground">OFF</span>
                  </div>

                  {/* Usage Progress */}
                  <div className="space-y-1">
                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>Usage</span>
                      <span>{promo.currentUsage}/{promo.usageLimit || '∞'}</span>
                    </div>
                    <div className="h-1.5 bg-muted rounded-full overflow-hidden">
                      <div
                        className={cn(
                          'h-full rounded-full transition-all duration-500',
                          usagePercent >= 90 ? 'bg-destructive' : usagePercent >= 70 ? 'bg-warning' : 'bg-success'
                        )}
                        style={{ width: `${usagePercent}%` }}
                      />
                    </div>
                  </div>

                  {/* Status Badge */}
                  <div className="mt-2 flex items-center gap-1">
                    {expired && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-destructive/10 text-destructive">
                        Expired
                      </span>
                    )}
                    {upcoming && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-info/10 text-info">
                        Upcoming
                      </span>
                    )}
                    {!promo.isActive && !expired && !upcoming && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                        Inactive
                      </span>
                    )}
                  </div>
                </div>
              );
            })}

            {/* Create New Card */}
            <div
              className={cn(
                'flex-shrink-0 w-48 bg-card rounded-xl border border-dashed border-success/30 p-4',
                'flex flex-col items-center justify-center cursor-pointer',
                'hover:bg-success/5 hover:border-success/50 transition-all duration-200'
              )}
              onClick={() => toast.info('Create promotion modal coming soon')}
            >
              <div className="w-10 h-10 rounded-full bg-success/10 flex items-center justify-center mb-2">
                <Plus className="h-5 w-5 text-success" />
              </div>
              <span className="text-sm font-medium text-success">New Promotion</span>
            </div>
          </div>

          {/* Show More */}
          {promotions.length > 6 && (
            <button
              onClick={() => setShowAll(!showAll)}
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground mt-2 mx-auto"
            >
              {showAll ? 'Show less' : `Show ${promotions.length - 6} more`}
              <ChevronRight className={cn('h-3 w-3 transition-transform', showAll && 'rotate-90')} />
            </button>
          )}
        </div>
      )}
    </div>
  );
}

export default PromotionSection;
