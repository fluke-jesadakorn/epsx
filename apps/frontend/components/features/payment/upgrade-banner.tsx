'use client';

import { cn } from '@/lib/utils';
import { ArrowRight, Clock, Gift, TrendingUp } from 'lucide-react';
import { useEffect, useState } from 'react';
import { env } from '@/shared/env/schema';

export interface UpgradePreviewData {
    current_plan: {
        name: string;
        price: string;
        expires_at: string | null;
        days_remaining: number;
    } | null;
    new_plan: {
        id: number;
        name: string;
        price: string;
        standard_duration_days: number;
    };
    upgrade_details: {
        remaining_credit: string;
        bonus_days: number;
        total_duration_days: number;
        new_expiry_date: string;
    };
    is_upgrade: boolean;
    is_downgrade: boolean;
    payment_required: string;
}

interface UpgradeBannerProps {
    newPlanId: number;
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
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!walletAddress || !newPlanId) {
            setLoading(false);
            return;
        }

        const fetchPreview = async () => {
            try {
                setLoading(true);
                const baseUrl = env.BACKEND_URL;
                const url = `${baseUrl}/api/payments/subscriptions/upgrade-preview?new_plan_id=${newPlanId}`;

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

                const result = await response.json();
                if (result.success && result.data) {
                    setPreview(result.data);
                }
            } catch (err) {
                console.error('Failed to fetch upgrade preview:', err);
                setError('Unable to calculate upgrade credit');
            } finally {
                setLoading(false);
            }
        };

        fetchPreview();
    }, [newPlanId, walletAddress]);

    // Don't render anything if no preview or loading
    if (loading || !preview) {
        return null;
    }

    // Don't show banner for new subscriptions (no current plan)
    if (!preview.current_plan) {
        return null;
    }

    // Show upgrade credit info if valid upgrade
    if (preview.is_upgrade && preview.upgrade_details.bonus_days > 0) {
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
                            🎉 Upgrade Bonus!
                        </h4>
                        <p className="mt-1 text-sm text-muted-foreground">
                            Your remaining{' '}
                            <span className="font-medium text-foreground">
                                {preview.current_plan.days_remaining} days
                            </span>{' '}
                            on {preview.current_plan.name} gives you
                            <span className="font-bold text-green-600 dark:text-green-400">
                                {' '}
                                ${parseFloat(preview.upgrade_details.remaining_credit).toFixed(2)} credit
                            </span>
                        </p>
                        <div className="mt-2 flex items-center gap-2">
                            <div className="flex items-center gap-1 rounded bg-green-500/20 px-2 py-1 text-sm font-medium text-green-600 dark:text-green-400">
                                <TrendingUp className="h-3.5 w-3.5" />
                                <span>+{preview.upgrade_details.bonus_days} bonus days</span>
                            </div>
                            <ArrowRight className="h-4 w-4 text-muted-foreground" />
                            <div className="flex items-center gap-1 rounded bg-primary/10 px-2 py-1 text-sm font-medium text-primary">
                                <Clock className="h-3.5 w-3.5" />
                                <span>{preview.upgrade_details.total_duration_days} days total</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    // Show downgrade warning
    if (preview.is_downgrade) {
        return (
            <div
                className={cn(
                    'rounded-lg border-2 border-amber-500/30 bg-gradient-to-r from-amber-500/10 to-orange-500/10 p-4',
                    className
                )}
            >
                <div className="flex items-start gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-amber-500/20">
                        <Clock className="h-5 w-5 text-amber-500" />
                    </div>
                    <div className="flex-1">
                        <h4 className="font-semibold text-amber-600 dark:text-amber-400">
                            ⚠️ Plan Change Notice
                        </h4>
                        <p className="mt-1 text-sm text-muted-foreground">
                            You&apos;re switching to a lower tier. Your payment will{' '}
                            <span className="font-medium text-foreground">
                                extend your current {preview.current_plan.name} plan
                            </span>{' '}
                            by 30 days instead.
                        </p>
                        <p className="mt-1 text-xs text-muted-foreground">
                            To switch to {preview.new_plan.name}, wait for your current plan
                            to expire.
                        </p>
                    </div>
                </div>
            </div>
        );
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
                            New total: {preview.upgrade_details.total_duration_days} days
                        </p>
                    </div>
                </div>
            </div>
        );
    }

    return null;
}

export default UpgradeBanner;
