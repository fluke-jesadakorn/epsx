'use client';

import { createPlansClient } from '@/shared/api/plans';
import { useAuth } from '@/shared/components/auth/provider';
import type { PlanAccessData } from '@/shared/types/payment';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { useCallback, useEffect, useState } from 'react';

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
    ranking_offset: FREE_PLAN_RANKING_OFFSET, // Free tier sees ranks 101+
    can_upgrade: true,
    tier_level: FREE_PLAN_TIER_LEVEL, // Free tier uses tier 0 styling
};

/**
 * @deprecated Use getMyPlanAccessAction() server action for new code.
 * This hook is maintained for legacy client-side components.
 * For server components, use `getMyPlanAccessAction()` directly.
 *
 * @example Server-side usage (recommended):
 * const planAccess = await getMyPlanAccessAction();
 *
 * @example Client-side usage (legacy):
 * const { planAccess } = usePlanAccess();
 */
export function usePlanAccess(): UsePlanAccessResult {
    const { isAuthenticated, user } = useAuth();
    const [planAccess, setPlanAccess] = useState<PlanAccessData | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchPlanAccess = useCallback(async (): Promise<void> => {
        if (!isAuthenticated) {
            setPlanAccess(DEFAULT_FREE_TIER);
            setLoading(false);
            return;
        }

        try {
            setLoading(true);
            setError(null);

            const plansClient = createPlansClient(createFrontendApiClient({ token: user?.access }));
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
    }, [isAuthenticated, user?.access]);

    useEffect(() => {
        void fetchPlanAccess();
    }, [isAuthenticated]);

    return {
        planAccess,
        loading,
        error,
        refetch: fetchPlanAccess,
    };
}

export default usePlanAccess;
