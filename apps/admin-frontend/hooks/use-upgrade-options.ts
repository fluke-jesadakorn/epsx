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
            if (accessLoading) {return;}

            try {
                setLoading(true);
                const plansClient = createPlansClient(createAdminApiClient());
                const response = await plansClient.getPublicPlans();

                if (response.success && response.data) {
                    const plans = response.data;
                    const currentPlanName = planAccess?.plan_name;

                    const sortedPlans = plans
                        .map(p => ({
                            id: Number(p.id) || 0,
                            name: p.name,
                            price: Number(p.effective_price || p.current_price),
                            features: p.features
                        }))
                        .sort((a, b) => a.price - b.price);

                    const candidates = sortedPlans.filter(p => p.price > 0);

                    const isFreePlan = currentPlanName === null || currentPlanName === '' || currentPlanName === FREE_PLAN_NAME;
                    let upgradeTarget: UpgradeOption | null = null;

                    if (isFreePlan) {
                        upgradeTarget = candidates[0] ?? null;
                    } else {
                        const currentIndex = candidates.findIndex(p => p.name === currentPlanName);
                        const hasNextPlan = currentIndex >= 0 && currentIndex < candidates.length - 1;

                        if (hasNextPlan) {
                            upgradeTarget = candidates[currentIndex + 1] ?? null;
                        } else {
                            upgradeTarget = candidates.find(p => p.price > (candidates[currentIndex]?.price ?? 0)) ?? null;
                        }
                    }

                    if (upgradeTarget !== null) {
                        setNextPlan(upgradeTarget);
                    } else if (candidates.length > 0 && isFreePlan) {
                        setNextPlan(candidates[0] ?? null);
                    }

                    const rec = candidates.find(p => p.name.includes('Pro') || p.name.includes('Growth'));
                    setRecommendedPlan(rec ?? candidates[Math.min(1, candidates.length - 1)] ?? null);
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
