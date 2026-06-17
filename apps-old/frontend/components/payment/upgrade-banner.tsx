'use client';

import { cn } from '@/lib/utils';
import { ArrowRight, Clock, Gift, TrendingUp } from 'lucide-react';
import { useEffect, useState } from 'react';
import { env } from '@/shared/env/schema';

export interface UpgradePreviewData {
    current_plan: {
        id: string | null;
        name: string;
        price: string;
        expires_at: string | null;
        started_at: string | null;
        days_remaining: number;
    } | null;
    new_plan: {
        id: string;
        name: string;
        price: string;
    };
    credit_from_current_plan: string;
    wallet_credit_balance: string;
    total_credits_available: string;
    amount_to_pay: string;
    new_duration_days: number;
    new_expiry_date: string;
    is_upgrade_allowed: boolean;
}

interface UpgradeBannerProps {
    newPlanId: string;
    walletAddress?: string;
    className?: string;
}

export function UpgradeBanner({
    newPlanId,
    walletAddress,
    className,
}: UpgradeBannerProps) {
    const [preview, setPreview] = useState<UpgradePreviewData | null>(null);
    const [loading, setLoading] = useState(true);
    const [_error, setError] = useState<string | null>(null);

    useEffect(() => {
        if ((walletAddress?.length ?? 0) === 0 || !newPlanId) {
            setLoading(false);
            return;
        }

        const fetchPreview = async () => {
            try {
                setLoading(true);
                const baseUrl = env.BACKEND_URL;
                const url = `${baseUrl}/api/payments/plans/upgrade_preview?new_plan_id=${newPlanId}`;

                const response = await fetch(url, {
                    method: 'GET',
                    headers: {
                        Accept: 'application/json',
                        'Content-Type': 'application/json',
                    },
                    credentials: 'include',
                });

                if (!response.ok) {
                    throw new Error(`HTTP error: ${response.status}`);
                }

                const result: { success?: boolean; data?: UpgradePreviewData } = await response.json();
                if ((result.success ?? false) && result.data) {
                    setPreview(result.data);
                }
            } catch (_err) {
      // Error logged silently
                setError('Unable to calculate upgrade credit');
            } finally {
                setLoading(false);
            }
        };

        void fetchPreview();
    }, [newPlanId, walletAddress]);

    // Don't render anything if no preview or loading
    if (loading || !preview) {
        return null;
    }

    // Don't show banner for new subscriptions (no current plan)
    if (!preview.current_plan) {
        return null;
    }

    const amtToPay = parseFloat(preview.amount_to_pay);
    const isFreeUpgrade = amtToPay === 0 && preview.current_plan.days_remaining > 0;

    // Show free upgrade day conversion info
    if (preview.is_upgrade_allowed && isFreeUpgrade) {
        return (
            <div
                className={cn(
                    'rounded-lg border-2 border-green-500/30 bg-gradient-to-r from-green-500/10 to-emerald-500/10 p-4',
                    className
                )}
            >
                <div className="flex items-start gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-green-500/20">
                        <Gift className="h-5 w-5 text-green-500" />
                    </div>
                    <div className="flex-1">
                        <h4 className="font-semibold text-green-600 dark:text-green-400">
                            Free Upgrade
                        </h4>
                        <p className="mt-1 text-sm text-muted-foreground">
                            Your remaining{' '}
                            <span className="font-medium text-foreground">
                                {preview.current_plan.days_remaining} days
                            </span>{' '}
                            on {preview.current_plan.name} converts to{' '}
                            <span className="font-bold text-green-600 dark:text-green-400">
                                {preview.new_duration_days} days
                            </span>{' '}
                            on {preview.new_plan.name}
                        </p>
                        <div className="mt-2 flex items-center gap-2">
                            <div className="flex items-center gap-1 rounded bg-green-500/20 px-2 py-1 text-sm font-medium text-green-600 dark:text-green-400">
                                <TrendingUp className="h-3.5 w-3.5" />
                                <span>{preview.current_plan.days_remaining}d → {preview.new_duration_days}d</span>
                            </div>
                            <ArrowRight className="h-4 w-4 text-muted-foreground" />
                            <div className="flex items-center gap-1 rounded bg-primary/10 px-2 py-1 text-sm font-medium text-primary">
                                <Clock className="h-3.5 w-3.5" />
                                <span>$0 — FREE</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    // Downgrades are disabled — no banner needed
    if (!preview.is_upgrade_allowed) {
        return null;
    }

    // Same plan extension
    if (preview.current_plan.name === preview.new_plan.name) {
        return (
            <div
                className={cn(
                    'rounded-lg border border-border/50 bg-muted/30 p-4',
                    className
                )}
            >
                <div className="flex items-start gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-primary/10">
                        <Clock className="h-5 w-5 text-primary" />
                    </div>
                    <div className="flex-1">
                        <h4 className="font-semibold text-foreground">Extending Your Plan</h4>
                        <p className="mt-1 text-sm text-muted-foreground">
                            Your {preview.current_plan.name} plan will be extended by{' '}
                            <span className="font-medium text-foreground">30 days</span>.
                        </p>
                        <p className="mt-1 text-xs text-muted-foreground">
                            Current: {preview.current_plan.days_remaining} days remaining →
                            New total: {preview.current_plan.days_remaining + preview.new_duration_days} days
                        </p>
                    </div>
                </div>
            </div>
        );
    }

    return null;
}

export default UpgradeBanner;
