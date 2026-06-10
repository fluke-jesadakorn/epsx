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

    const ps = planStats ?? {
        total_plans: 0,
        active_plans: 0,
        system_plans: 0,
        total_memberships: 0,
        active_memberships: 0,
        avg_permissions_per_plan: 0,
        recent_assignments: 0,
        recent_removals: 0,
    };

    // Map the internal data to the format expected by PlanAnalyticsDashboard
    const stats = {
        totalPlans: ps.total_plans,
        activePlans: ps.active_plans,
        systemPlans: ps.system_plans,
        totalMemberships: ps.total_memberships,
        activeMemberships: ps.active_memberships,
        avgPermissionsPerPlan: ps.avg_permissions_per_plan,
        recentAssignments: ps.recent_assignments,
        recentRemovals: ps.recent_removals,
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
            if (!res.success) { throw new Error(res.error); }
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
