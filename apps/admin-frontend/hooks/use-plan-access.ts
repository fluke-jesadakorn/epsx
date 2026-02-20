'use client';

import { createPlansClient } from '@/shared/api/plans';
import type { PlanAccessData } from '@/shared/types/payment';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';

import { FREE_PLAN_RANKING_OFFSET, FREE_PLAN_TIER_LEVEL } from '@/shared/config/constants';

interface UsePlanAccessResult {
    planAccess: PlanAccessData | null;
    loading: boolean;
    error: string | null;
    refetch: () => Promise<void>;
}

const DEFAULT_FREE_TIER: PlanAccessData = {
    wallet_address: '',
    plan_name: null,
    plan_expires_at: null,
    days_remaining: 0,
    status: 'no_plan',
    ranking_offset: FREE_PLAN_RANKING_OFFSET,
    can_upgrade: true,
    tier_level: FREE_PLAN_TIER_LEVEL,
};

export function usePlanAccess(): UsePlanAccessResult {
    const [planAccess, setPlanAccess] = useState<PlanAccessData | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchPlanAccess = async (): Promise<void> => {
        try {
            setLoading(true);
            setError(null);

            const plansClient = createPlansClient(createAdminApiClient());
            const response = await plansClient.getMyPlanAccess();

            if (response.success && response.data) {
                setPlanAccess(response.data);
            } else {
                setPlanAccess(DEFAULT_FREE_TIER);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to fetch plan');
            setPlanAccess(DEFAULT_FREE_TIER);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        void fetchPlanAccess();
    }, []);

    return {
        planAccess,
        loading,
        error,
        refetch: fetchPlanAccess,
    };
}

export default usePlanAccess;
