'use client';

import { createPlansClient } from '@/shared/api/plans';
import type { PlanAccessData } from '@/shared/types/payment';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';

interface UsePlanAccessResult {
    planAccess: PlanAccessData | null;
    loading: boolean;
    error: string | null;
    refetch: () => Promise<void>;
}

const DEFAULT_FREE_TIER: PlanAccessData = {
    wallet_address: '',
    current_plan_id: null,
    plan_name: null,
    plan_expires_at: null,
    days_remaining: 0,
    status: 'no_plan',
    rankings_view_limit: 3,
    can_upgrade: true,
};

/**
 * Hook to fetch current user's plan access data including rankings limit
 * Uses shared PlansAPIClient to call backend directly
 */
export function usePlanAccess(): UsePlanAccessResult {
    const [planAccess, setPlanAccess] = useState<PlanAccessData | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchPlanAccess = async (): Promise<void> => {
        try {
            setLoading(true);
            setError(null);

            const plansClient = createPlansClient(createFrontendApiClient());
            // Use shorter timeout for plan access - fail fast if backend unreachable
            const response = await plansClient.getMyPlanAccess();

            if (response.success && response.data) {
                setPlanAccess(response.data);
            } else {
                // User not logged in or no plan - return default free tier
                setPlanAccess(DEFAULT_FREE_TIER);
            }
        } catch (err) {
            console.error('Failed to fetch plan access:', err);
            setError(err instanceof Error ? err.message : 'Failed to fetch plan');
            // Set default on error
            setPlanAccess(DEFAULT_FREE_TIER);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchPlanAccess();
    }, []);

    return {
        planAccess,
        loading,
        error,
        refetch: fetchPlanAccess,
    };
}

export default usePlanAccess;
