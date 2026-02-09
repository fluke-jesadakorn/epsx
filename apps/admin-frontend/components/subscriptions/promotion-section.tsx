/**
 * PromotionSection Component
 *
 * Compact horizontal card layout for promotions:
 * - Shows active promotions as small cards in a horizontal scroll
 * - Inline create button
 * - Quick stats and usage progress
 * - Supports compact mode for sidebar display
 */
/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call */
'use client';

import { ChevronRight, Gift, Plus, RefreshCw } from 'lucide-react';
import { useCallback, useState } from 'react';
import { toast } from 'sonner';

import type { DisplayPromotion } from '@/components/promotions/types';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { createPromotionsClient, isApiSuccess, type Promotion } from '@/shared/api/promotions';
import { createAdminApiClient } from '@/shared/utils/api-client';

import {
  CompactPromotionList,
  CreatePromotionCard,
  isExpired,
  PromotionCard,
  PromotionStats
} from './promotion/promotion-components';

interface PromotionSectionProps {
  initialPromotions: DisplayPromotion[];
  className?: string;
  compactMode?: boolean;
}

export function PromotionSection({ initialPromotions, className, compactMode = false }: PromotionSectionProps) {
  const [promotions, setPromotions] = useState<DisplayPromotion[]>(initialPromotions);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [showAll, setShowAll] = useState(false);

  // Stats
  const activePromotions = promotions.filter(p => (p.isActive === true) && !isExpired(p.endDate));
  const totalUsage = promotions.reduce((sum, p) => sum + (p.currentUsage ?? 0), 0);
  const totalRevenue = promotions.reduce((sum, p) => sum + (p.totalRevenue ?? 0), 0);

  // Displayed promotions (show first 6 unless expanded, or first 3 in compact mode)
  const limit = compactMode ? 3 : 6;
  const displayedPromotions = showAll ? promotions : promotions.slice(0, limit);

  // Refresh handler
  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      const apiClient = createAdminApiClient();
      const promotionsClient = createPromotionsClient(apiClient);
      const response = await promotionsClient.getPromotions({ limit: 100 });

      if (isApiSuccess(response)) {
        const promos = response.data?.promotions ?? [];
        setPromotions(promos.map((p: Promotion) => ({
          id: p.id,
          name: p.name,
          code: p.code,
          description: p.description,
          discountType: p.discountType,
          discountValue: parseFloat(p.discountValue),
          maxDiscountAmount: p.maxDiscountAmount ? parseFloat(p.maxDiscountAmount) : null,
          minPurchaseAmount: parseFloat(p.minPurchaseAmount ?? '0'),
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
    } catch (_error) {
      toast.error('Failed to refresh promotions');
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  if (compactMode) {
    return (
      <CompactPromotionList
        promotions={displayedPromotions}
        className={className}
      />
    );
  }

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
            onClick={() => { void handleRefresh(); }}
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
      <PromotionStats
        promotions={promotions}
        activePromotionsCount={activePromotions.length}
        totalRevenue={totalRevenue}
      />

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
            {displayedPromotions.map((promo) => (
              <PromotionCard key={promo.id} promo={promo} />
            ))}

            <CreatePromotionCard />
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
