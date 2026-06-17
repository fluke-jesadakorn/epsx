'use client';

import { getPublicPlansAction } from '@/app/actions/plans';
import { fmtAmt } from '@/shared/utils/formatting/currency';
import { logger } from '@/shared/utils/logger';
import { useCallback, useEffect, useRef, useState } from 'react';
import type { PricingCardData } from '@/shared/components/plans/pricing-card';

export interface RawPlan {
    id: string;
    name: string;
    current_price: string | number;
    base_price?: string | number;
    is_active: boolean;
    display_order?: number;
    is_highlighted?: boolean;
    is_promoted?: boolean;
    features?: string[] | string;
    tier_level?: number;
    plan_type?: string;
    description?: string;
    effective_price?: number;
    promotion_active?: boolean;
    promotion_discount?: number;
    promotion_ends_at?: string;
    currency?: string;
}

export interface PaymentToken {
    symbol: string;
    name: string;
    decimals: number;
}

export const PAYMENT_TOKENS: PaymentToken[] = [
    { symbol: 'USDT', name: 'Tether USD', decimals: 18 },
    { symbol: 'USDC', name: 'USD Coin', decimals: 18 },
    { symbol: 'DAI', name: 'Dai Stablecoin', decimals: 18 },
];

interface UsePaymentPlansCtx {
    preselectedId?: string;
    initialPlans: RawPlan[];
    urlPlanId: string | null;
    urlToken: string | null;
}

export function usePaymentPlans({ preselectedId, initialPlans, urlPlanId, urlToken }: UsePaymentPlansCtx) {
    const [plans, setPlans] = useState<PricingCardData[]>([]);
    const [loading, setLoading] = useState(initialPlans.length === 0);
    const [error, setError] = useState<string | null>(null);
    const [selectedPlan, setSelectedPlan] = useState<PricingCardData | null>(null);
    const [showAllPlans, setShowAllPlans] = useState(false);
    const [selectedToken, setSelectedToken] = useState<PaymentToken>(() => {
        if (urlToken) {
            const found = PAYMENT_TOKENS.find(tk => tk.symbol === urlToken);
            if (found) { return found; }
        }
        return PAYMENT_TOKENS[0];
    });

    const urlPlanIdRef = useRef(urlPlanId);

    const transformPlans = useCallback((rawPlans: RawPlan[]): PricingCardData[] => {
        return rawPlans
            .filter((plan) => plan.is_active)
            .sort((a, b) => (a.display_order ?? 0) - (b.display_order ?? 0))
            .map((plan) => {
                const basePrice = typeof plan.current_price === 'string' ? parseFloat(plan.current_price) : plan.current_price;
                const hasPromo = plan.promotion_active === true && plan.effective_price !== undefined && plan.effective_price < basePrice;
                const displayPrice = hasPromo ? (plan.effective_price ?? basePrice) : basePrice;
                const isFree = displayPrice === 0;
                const currency = plan.currency ?? 'USD';
                const discount = plan.promotion_discount ?? 0;
                const savedAmt = hasPromo ? basePrice - displayPrice : 0;
                const parsedFeatures = (Array.isArray(plan.features) ? plan.features : typeof plan.features === 'string' ? JSON.parse(plan.features) : []) as Array<string | { text: string; included: boolean }>;
                return {
                    id: plan.id,
                    title: plan.name,
                    price: isFree ? 'Free' : `$${fmtAmt(displayPrice)} ${currency}`,
                    originalPrice: hasPromo ? `$${fmtAmt(basePrice)} ${currency}` : undefined,
                    features: parsedFeatures.map((f) => typeof f === 'string' ? { text: f, included: true } : f),
                    highlight: plan.is_highlighted ?? plan.is_promoted,
                    buttonText: isFree ? 'Start Free' : 'Select Plan',
                    promotions: hasPromo ? [`${Math.round(discount)}% OFF`] : [],
                    savings: hasPromo ? `Save $${fmtAmt(savedAmt)}` : undefined,
                    promotion_ends_at: hasPromo ? plan.promotion_ends_at : undefined,
                    tier_level: plan.tier_level ?? 0,
                    plan_type: plan.plan_type,
                    description: plan.description,
                };
            });
    }, []);

    const applyPreselect = useCallback((transformed: PricingCardData[]) => {
        const selectId = urlPlanIdRef.current ?? preselectedId;
        if (selectId) {
            const found = transformed.find((p) => String(p.id) === String(selectId));
            if (found) { setSelectedPlan(found); }
        }
    }, [preselectedId]);

    const fetchPlans = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const result = await getPublicPlansAction();
            if (result.success && result.data !== null && Array.isArray(result.data)) {
                const transformed = transformPlans(result.data as RawPlan[]);
                setPlans(transformed);
                applyPreselect(transformed);
            } else {
                throw new Error(result.error?.message ?? 'Invalid API response format');
            }
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Failed to load plans';
            setError(`${msg}. Please refresh or try again.`);
            logger.error('[Payment] Failed to fetch plans:', err);
        } finally {
            setLoading(false);
        }
    }, [transformPlans, applyPreselect]);

    useEffect(() => {
        if (initialPlans.length > 0) {
            const transformed = transformPlans(initialPlans);
            setPlans(transformed);
            setLoading(false);
            applyPreselect(transformed);
        } else {
            void fetchPlans();
        }
    }, [initialPlans, transformPlans, fetchPlans, applyPreselect]);

    useEffect(() => {
        const timeout = setTimeout(() => {
            if (loading) { setError('Loading timeout - please refresh the page'); setLoading(false); }
        }, 10000);
        return () => clearTimeout(timeout);
    }, [loading]);

    return { plans, loading, error, setError, selectedPlan, setSelectedPlan, selectedToken, setSelectedToken, showAllPlans, setShowAllPlans, fetchPlans };
}
