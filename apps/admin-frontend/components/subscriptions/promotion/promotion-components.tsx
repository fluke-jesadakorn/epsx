'use client';

import { Gift, Plus, Ticket, TrendingUp } from 'lucide-react';
import { toast } from 'sonner';

import type { DisplayPromotion } from '@/components/promotions/types';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

// Helper functions
export function getUsagePercentage(promo: DisplayPromotion): number {
    if ((promo.usageLimit ?? 0) === 0) { return 0; }
    return Math.min(((promo.currentUsage ?? 0) / (promo.usageLimit ?? 1)) * 100, 100);
}

export function getDiscountDisplay(promo: DisplayPromotion): string {
    if (promo.discountType === 'percentage') {
        return `${promo.discountValue}%`;
    }
    return `$${promo.discountValue}`;
}

export function isExpired(endDate?: string): boolean {
    if (!endDate || endDate === '') { return false; }
    return new Date(endDate) < new Date();
}

export function isUpcoming(startDate: string): boolean {
    return new Date(startDate) > new Date();
}

export function isActive(p: DisplayPromotion) {
    return (p.isActive === true) && !isExpired(p.endDate);
}

interface CompactPromotionListProps {
    promotions: DisplayPromotion[];
    className?: string;
}

export function CompactPromotionList({ promotions, className }: CompactPromotionListProps) {
    return (
        <div className={cn('space-y-3', className)}>
            {promotions.map((promo) => {
                const expired = isExpired(promo.endDate);
                const usagePercent = getUsagePercentage(promo);

                return (
                    <div
                        key={promo.id}
                        className={cn(
                            "flex flex-col gap-2 p-3 bg-muted/30 rounded-lg border border-border transition-colors hover:bg-muted/50 cursor-pointer",
                            isActive(promo) ? "border-l-2 border-l-success" : ""
                        )}
                        onClick={() => toast.info(`View promotion: ${promo.name}`)}
                    >
                        <div className="flex justify-between items-start">
                            <div>
                                <div className="flex items-center gap-2 mb-1">
                                    <code className="text-[10px] font-mono font-bold bg-muted px-1 py-0.5 rounded">{promo.code}</code>
                                    {(promo.isActive === true) && !expired && <div className="w-1.5 h-1.5 rounded-full bg-success animate-pulse" />}
                                </div>
                                <div className="font-semibold text-xs">{promo.name}</div>
                            </div>
                            <div className="text-right">
                                <div className="text-sm font-bold text-success">{getDiscountDisplay(promo)}</div>
                            </div>
                        </div>

                        <div className="w-full h-1 bg-muted rounded-full overflow-hidden">
                            <div className="h-full bg-success transition-all" style={{ width: `${usagePercent}%` }} />
                        </div>
                        <div className="flex justify-between text-[10px] text-muted-foreground">
                            <span>{promo.currentUsage} used</span>
                            <span>{usagePercent.toFixed(0)}%</span>
                        </div>
                    </div>
                );
            })}

            <Button variant="outline" size="sm" className="w-full text-xs h-8" onClick={() => toast.info('View all promotions')}>
                View All Promotions
            </Button>
        </div>
    );
}

interface PromotionStatsProps {
    promotions: DisplayPromotion[];
    activePromotionsCount: number;
    totalRevenue: number;
}

export function PromotionStats({ promotions, activePromotionsCount, totalRevenue }: PromotionStatsProps) {
    return (
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
                <div className="text-lg font-bold text-success">{activePromotionsCount}</div>
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
    );
}

interface PromotionCardProps {
    promo: DisplayPromotion;
}

export function PromotionCard({ promo }: PromotionCardProps) {
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
                (promo.isActive === true) && !expired ? 'border-success/20' : 'border-border'
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
                {(promo.isActive === true) && !expired && (
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
                    <span>{promo.currentUsage}/{promo.usageLimit ?? '∞'}</span>
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
                {(promo.isActive !== true) && !expired && !upcoming && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                        Inactive
                    </span>
                )}
            </div>
        </div>
    );
}

export function CreatePromotionCard() {
    return (
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
    );
}
