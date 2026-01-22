'use client';

import { createPlansClient } from '@/shared/api/plans';
import { FREE_PLAN_NAME } from '@/shared/config/constants';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';
import { usePlanAccess } from './usePlanAccess';

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

/**
 * Hook to fetch available upgrade options for the current user
 * Uses public plans API and client-side filtering since backend doesn't offer "next plan" endpoint
 */
export function useUpgradeOptions(): UseUpgradeOptionsResult {
    const [nextPlan, setNextPlan] = useState<UpgradeOption | null>(null);
    const [recommendedPlan, setRecommendedPlan] = useState<UpgradeOption | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const { planAccess, loading: accessLoading } = usePlanAccess();

    useEffect(() => {
        const fetchUpgradeOptions = async () => {
            // Wait for plan access to be loaded (to know current tier)
            if (accessLoading) return;

            try {
                setLoading(true);
                const plansClient = createPlansClient(createFrontendApiClient());

                // Fetch all public plans
                const response = await plansClient.getPublicPlans();

                if (response.success && response.data) {
                    const plans = response.data;
                    const currentPlanName = planAccess?.plan_name;

                    // Sort by price (value)
                    const sortedPlans = plans
                        .map(p => ({
                            id: Number(p.id) || 0,
                            name: p.name,
                            price: Number(p.effective_price || p.current_price),
                            features: p.features
                        }))
                        .sort((a, b) => a.price - b.price);

                    // Filter for candidates (paid plans)
                    const candidates = sortedPlans.filter(p => p.price > 0);

                    let upgradeTarget: UpgradeOption | null = null;

                    if (!currentPlanName || currentPlanName === FREE_PLAN_NAME) {
                        // If Free: Next is lowest price paid plan
                        upgradeTarget = candidates[0];
                    } else {
                        // If Paid: Find current plan in list and pick next one
                        const currentIndex = candidates.findIndex(p => p.name === currentPlanName);
                        if (currentIndex >= 0 && currentIndex < candidates.length - 1) {
                            upgradeTarget = candidates[currentIndex + 1];
                        } else {
                            // If current is highest or unknown, fallback to first candidate or recommended
                            // Avoid showing downgrade
                            upgradeTarget = candidates.find(p => p.price > (candidates[currentIndex]?.price || 0)) || null;
                        }
                    }

                    if (upgradeTarget) {
                        setNextPlan(upgradeTarget);
                    } else if (candidates.length > 0 && (!currentPlanName || currentPlanName === FREE_PLAN_NAME)) {
                        setNextPlan(candidates[0]);
                    }

                    // Recommended: Pro Plan or similar
                    const rec = candidates.find(p => p.name.includes('Pro') || p.name.includes('Growth'));
                    setRecommendedPlan(rec || candidates[Math.min(1, candidates.length - 1)] || null);
                }
            } catch (err) {
                console.error('Failed to fetch upgrade options:', err);
                setError('Failed to load upgrade options');
            } finally {
                setLoading(false);
            }
        };

        fetchUpgradeOptions();
    }, [accessLoading, planAccess]);

    return {
        nextPlan,
        recommendedPlan,
        loading,
        error
    };
}
