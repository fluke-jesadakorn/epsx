'use client';

import { createPlansClient } from '@/shared/api/plans';
import { FREE_PLAN_NAME } from '@/shared/config/constants';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';
import { usePlanAccess } from './use-plan-access';

export interface UpgradeOption {
    id: number;
    name: string;
    price: number;
    features: string[];
}

function calculateNextPlan(candidatePlans: UpgradeOption[], isFreePlan: boolean, currentPlanName: string | null | undefined): UpgradeOption | null {
    if (isFreePlan) {
        return candidatePlans[0] ?? null;
    }
    const currentIndex = candidatePlans.findIndex(p => p.name === currentPlanName);
    if (currentIndex >= 0 && currentIndex < candidatePlans.length - 1) {
        return candidatePlans[currentIndex + 1] ?? null;
    }
    if (currentIndex >= 0) {
        return candidatePlans.find(p => p.price > (candidatePlans[currentIndex]?.price ?? 0)) ?? null;
    }
    return null;
}

function toUpgradeOption(p: { id: string | number; name: string; effective_price?: number | null; current_price?: number | null; features: string[] }): UpgradeOption {
    return {
        id: Number(p.id) || 0,
        name: p.name,
        price: Number(p.effective_price ?? p.current_price),
        features: p.features,
    };
}

function pickRecommended(candidatePlans: UpgradeOption[]): UpgradeOption | null {
    return candidatePlans.find(p => p.name.includes('Pro') || p.name.includes('Growth'))
        ?? candidatePlans[Math.min(1, candidatePlans.length - 1)]
        ?? null;
}

interface UseUpgradeOptionsResult {
    nextPlan: UpgradeOption | null;
    recommendedPlan: UpgradeOption | null;
    loading: boolean;
    error: string | null;
}

export function useUpgradeOptions(): UseUpgradeOptionsResult {
    const [nextPlan, setNextPlan] = useState<UpgradeOption | null>(null);
    const [recommendedPlan, setRecommendedPlan] = useState<UpgradeOption | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const { planAccess, loading: accessLoading } = usePlanAccess();

    useEffect(() => {
        const fetchUpgradeOptions = async () => {
            if (accessLoading) { return; }

            try {
                setLoading(true);
                const plansClient = createPlansClient(createAdminApiClient());
                const response = await plansClient.getPublicPlans();

                if (response.success && response.data) {
                    const currentPlanName = planAccess?.plan_name;
                    const candidatePlans = response.data
                        .map(toUpgradeOption)
                        .sort((a, b) => a.price - b.price)
                        .filter(p => p.price > 0);

                    const isFreePlan = currentPlanName === undefined || currentPlanName === null || currentPlanName === '' || currentPlanName === FREE_PLAN_NAME;
                    const upgradeTarget = calculateNextPlan(candidatePlans, isFreePlan, currentPlanName);
                    setNextPlan(upgradeTarget ?? (isFreePlan ? (candidatePlans[0] ?? null) : null));
                    setRecommendedPlan(pickRecommended(candidatePlans));
                }
            } catch (_err) {
                setError('Failed to load upgrade options');
            } finally {
                setLoading(false);
            }
        };

        void fetchUpgradeOptions();
    }, [accessLoading, planAccess]);

    return {
        nextPlan,
        recommendedPlan,
        loading,
        error
    };
}
