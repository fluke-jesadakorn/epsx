'use client';

import { usePlanStats } from './use-analytics-data';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import { useQuery } from '@tanstack/react-query';

/**
 * Hook for Plan Analytics Dashboard
 * Acts as a bridge to the unified usePlanStats hook
 */
export function usePlanAnalytics() {
    const { planStats, isLoading, error, refresh } = usePlanStats();

    // Map the internal data to the format expected by PlanAnalyticsDashboard
    const stats = {
        totalPlans: planStats?.total_plans ?? 0,
        activePlans: planStats?.active_plans ?? 0,
        systemPlans: planStats?.system_plans ?? 0,
        totalMemberships: planStats?.total_memberships ?? 0,
        activeMemberships: planStats?.active_memberships ?? 0,
        avgPermissionsPerPlan: planStats?.avg_permissions_per_plan ?? 0,
        recentAssignments: planStats?.recent_assignments ?? 0,
        recentRemovals: planStats?.recent_removals ?? 0,
    };

    return {
        stats,
        loading: isLoading,
        error,
        refreshStats: refresh,
    };
}

// Re-export as the name requested by PlanAnalyticsDashboard.tsx:10
// import { usePlanAnalytics } from '@/hooks/use-plan-permissions';
export default usePlanAnalytics;

export function useAvailablePermissions() {
    const { data, isLoading, error } = useQuery({
        queryKey: ['available-permissions-list'],
        queryFn: async () => {
            const res = await getPermissionsAction();
            if (!res.success) {throw new Error(res.error);}
            // Return permission strings
            return res.data?.map(p => p.permission_string) ?? [];
        }
    });

    return {
        permissions: data ?? [],
        isLoading,
        error
    };
}
