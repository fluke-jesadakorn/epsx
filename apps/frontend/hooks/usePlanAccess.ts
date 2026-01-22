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

import { FREE_PLAN_RANKING_OFFSET, FREE_PLAN_TIER_LEVEL } from '@/shared/config/constants';

const DEFAULT_FREE_TIER: PlanAccessData = {
    wallet_address: '',
    current_plan_id: null,
    plan_name: null,
    plan_expires_at: null,
    days_remaining: 0,
    status: 'no_plan',
    ranking_offset: FREE_PLAN_RANKING_OFFSET, // Free tier sees ranks 101+
    can_upgrade: true,
    tier_level: FREE_PLAN_TIER_LEVEL, // Free tier uses tier 0 styling
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
            try {
                const response = await plansClient.getMyPlanAccess();

                if (response.success && response.data) {
                    setPlanAccess(response.data);
                } else {
                    // User not logged in or no plan - return default free tier
                    setPlanAccess(DEFAULT_FREE_TIER);
                }
            } catch (err: any) { // Use any to access status safely
                // Handle 401 Unauthorized (expired token)
                if (err?.status === 401 || err?.message?.includes('401')) {
                    console.log('Plan access 401, attempting session refresh...');
                    // Dynamic import to avoid circular dependencies if any
                    const { authService } = await import('@/lib/auth/service');
                    const refreshed = await authService.refreshSession();

                    if (refreshed) {
                        console.log('Session refreshed, retrying plan access...');
                        // Retry with new token (which should be in cookies now)
                        const retryClient = createPlansClient(createFrontendApiClient());
                        const response = await retryClient.getMyPlanAccess();
                        if (response.success && response.data) {
                            setPlanAccess(response.data);
                            return;
                        }
                    }
                }
                throw err;
            }
        } catch (err) {
            console.error('Failed to fetch plan access:', err);
            // Don't show error for 401/403 as it just means free tier/not logged in
            const isAuthError = err instanceof Error && (err.message.includes('401') || err.message.includes('403'));
            if (!isAuthError) {
                setError(err instanceof Error ? err.message : 'Failed to fetch plan');
            }
            // Set default on error (fallback to free tier)
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
