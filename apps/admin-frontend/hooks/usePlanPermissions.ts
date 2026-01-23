/**
 * Permission Plans Management Hooks
 * Provides hooks for managing permission plans and available permissions
 */

import { useCallback, useEffect, useMemo, useState } from 'react';

import {
  assignUserToPlanAction,
  createPlanAction,
  deletePlanAction,
  getAvailablePermissionsAction,
  getPlanMembershipsAction,
  getPlansAction,
  getUserPlansAction,
  removeUserFromPlanAction,
  updatePlanAction
} from '@/app/wallet-management/plan-actions';
import { AssignUserToPlanRequest, CreatePlanRequest, PermissionPlan as Plan, PlanAssignmentHistory, UpdatePlanRequest, UserPlanMembership } from '@/lib/api/plan-management-client';

// ============================================================================
// PERMISSION PLANS HOOK
// ============================================================================

interface UsePlansReturn {
  plans: Plan[];
  loading: boolean;
  error: string | null;
  createPlan: (request: CreatePlanRequest) => Promise<Plan>;
  updatePlan: (planId: string, request: UpdatePlanRequest) => Promise<Plan>;
  deletePlan: (planId: string) => Promise<void>;
  refreshPlans: () => Promise<void>;
}

export type UsePermissionPlansReturn = UsePlansReturn;

export const usePlanPermissions = usePlans;

/**
 *
 */
export function usePlans(): UsePlansReturn {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadPlans = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const fetchedPlans = await getPlansAction();
      setPlans(fetchedPlans);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load permission plans');
    } finally {
      setLoading(false);
    }
  }, []);

  const createPlan = useCallback(async (request: CreatePlanRequest): Promise<Plan> => {
    try {
      setError(null);
      // @ts-ignore - return type mismatch on action vs client type expectations? Action returns mapped object
      const newPlan = await createPlanAction(request);
      // Refresh plans after creation
      await loadPlans();
      // @ts-ignore
      return newPlan;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create plan';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadPlans]);

  const updatePlan = useCallback(async (planId: string, request: UpdatePlanRequest): Promise<Plan> => {
    try {
      setError(null);
      // @ts-ignore
      const updatedPlan = await updatePlanAction(planId, request);
      // Refresh plans after update
      await loadPlans();
      // @ts-ignore
      return updatedPlan;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update plan';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadPlans]);

  const deletePlan = useCallback(async (planId: string): Promise<void> => {
    try {
      setError(null);
      await deletePlanAction(planId);
      // Refresh plans after deletion
      await loadPlans();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete plan';
      setError(errorMessage);
      throw new Error(errorMessage);
    }
  }, [loadPlans]);

  // Load plans on mount
  useEffect(() => {
    loadPlans();
  }, [loadPlans]);

  return {
    plans,
    loading,
    error,
    createPlan,
    updatePlan,
    deletePlan,
    refreshPlans: loadPlans,
  };
}

// ============================================================================
// AVAILABLE PERMISSIONS HOOK
// ============================================================================

interface UseAvailablePermissionsReturn {
  permissions: string[];
  isLoading: boolean;
  error: string | null;
  refreshPermissions: () => Promise<void>;
}

/**
 *
 */
export function useAvailablePermissions(): UseAvailablePermissionsReturn {
  const [permissions, setPermissions] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadPermissions = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const fetchedPermissions = await getAvailablePermissionsAction();
      setPermissions(fetchedPermissions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load available permissions');
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load permissions on mount
  useEffect(() => {
    loadPermissions();
  }, [loadPermissions]);

  return {
    permissions,
    isLoading,
    error,
    refreshPermissions: loadPermissions,
  };
}

// ============================================================================
// PLAN ANALYTICS HOOK
// ============================================================================

interface PlanAnalyticsStats {
  totalPlans: number;
  activePlans: number;
  systemPlans: number;
  totalUsers: number;
  avgPermissionsPerPlan: number;
  recentActivity: number;
  health_score: number;
}

interface UsePlanAnalyticsReturn {
  stats: PlanAnalyticsStats;
  loading: boolean;
  error: string | null;
  refreshStats: () => Promise<void>;
}

/**
 *
 */
export function usePlanAnalytics(): UsePlanAnalyticsReturn {
  const [stats, setStats] = useState<PlanAnalyticsStats>({
    totalPlans: 0,
    activePlans: 0,
    systemPlans: 0,
    totalUsers: 0,
    avgPermissionsPerPlan: 0,
    recentActivity: 0,
    health_score: 0,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadStats = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // For now, provide basic analytics - can be enhanced later
      const plans = await getPlansAction();
      const totalPlans = plans.length;
      const activePlans = plans.filter((g: Plan) => g.is_active).length;
      const systemPlans = plans.filter((g: Plan) => (g.plan_type === 'system')).length;
      const totalPermissions = plans.reduce((sum: number, g: Plan) => sum + g.permissions.length, 0);

      setStats({
        totalPlans,
        activePlans,
        systemPlans,
        totalUsers: 0, // Would need separate API call
        avgPermissionsPerPlan: totalPlans > 0 ? Math.round(totalPermissions / totalPlans) : 0,
        recentActivity: 0, // Would need separate API call
        health_score: totalPlans > 0 ? Math.round((activePlans / totalPlans) * 100) : 100,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load plan analytics');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  return {
    stats,
    loading,
    error,
    refreshStats: loadStats,
  };
}

// ============================================================================
// WEB3 ASSIGNMENT RULES HOOK
// ============================================================================

interface UseWeb3AssignmentRulesReturn {
  rules: any[]; // Would be Web3AssignmentRule[] in full implementation
  loading: boolean;
  error: string | null;
  processWallet: (walletAddress: string) => Promise<void>;
  verifyWalletAssets: (walletAddress: string) => Promise<boolean>;
  refreshRules: () => Promise<void>;
}

/**
 *
 */
export function useWeb3AssignmentRules(): UseWeb3AssignmentRulesReturn {
  const [rules, setRules] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadRules = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      // For now, return empty rules - would need specific API endpoint
      setRules([]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load Web3 assignment rules');
    } finally {
      setLoading(false);
    }
  }, []);

  const processWallet = useCallback(async (_walletAddress: string): Promise<void> => {
    try {
      setError(null);
      // For now, just log - would need specific API endpoint
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to process wallet');
      throw err;
    }
  }, []);

  const verifyWalletAssets = useCallback(async (_walletAddress: string): Promise<boolean> => {
    try {
      setError(null);
      // For now, return true - would need actual verification
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to verify wallet assets');
      return false;
    }
  }, []);

  useEffect(() => {
    loadRules();
  }, [loadRules]);

  return {
    rules,
    loading,
    error,
    processWallet,
    verifyWalletAssets,
    refreshRules: loadRules,
  };
}

// Backward compatibility exports
export const usePermissionPlans = usePlans;

/**
 *
 * @param userId
 */
export function useUserPlanMemberships(userId: string | null) {
  const [memberships, setMemberships] = useState<UserPlanMembership[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadMemberships = useCallback(async () => {
    if (!userId) {
      setMemberships([]);
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      const data = await getUserPlansAction(userId);
      setMemberships(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load user memberships');
      // setMemberships([]); // Keep existing on error?
    } finally {
      setIsLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    loadMemberships();
  }, [loadMemberships]);

  const activeMemberships = useMemo(() =>
    memberships.filter(m => m.is_active),
    [memberships]
  );

  const expiringMemberships = useMemo(() => {
    const now = Date.now();
    const sevenDaysFromNow = now + (7 * 24 * 60 * 60 * 1000);
    return memberships.filter(m =>
      m.is_active &&
      m.expires_at &&
      new Date(m.expires_at).getTime() <= sevenDaysFromNow
    );
  }, [memberships]);

  const assignUserToPlan = useCallback(async (request: AssignUserToPlanRequest) => {
    await assignUserToPlanAction(request.user_id, request.plan_id, request.expires_at);
    await loadMemberships();
  }, [loadMemberships]);

  const removeUserFromPlan = useCallback(async (planId: string) => {
    if (!userId) { return; }
    await removeUserFromPlanAction(userId, planId);
    await loadMemberships();
  }, [userId, loadMemberships]);

  const refreshMemberships = loadMemberships;

  return {
    memberships,
    activeMemberships,
    expiringMemberships,
    isLoading,
    error,
    assignUserToPlan,
    removeUserFromPlan,
    refreshMemberships,
  };
}

// ============================================================================
// PLAN MEMBERS HOOK
// ============================================================================

/**
 *
 * @param planId
 */
export function usePlanMembers(planId: string | null) {
  const [members, setMembers] = useState<UserPlanMembership[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadMembers = useCallback(async () => {
    if (!planId) {
      setMembers([]);
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      const data = await getPlanMembershipsAction(planId);
      setMembers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load plan members');
    } finally {
      setIsLoading(false);
    }
  }, [planId]);

  useEffect(() => {
    loadMembers();
  }, [loadMembers]);

  return {
    members,
    isLoading,
    error,
    refreshMembers: loadMembers,
  };
}

// ============================================================================
// PLAN ASSIGNMENT HISTORY HOOK (STUB)
// ============================================================================

/**
 *
 */
export function usePlanAssignmentHistory() {
  return {
    history: [] as PlanAssignmentHistory[],
    isLoading: false,
    refreshHistory: async () => { },
  };
}

// ============================================================================
// COMBINED HOOKS EXPORT (BACKWARD COMPATIBILITY)
// ============================================================================

export const planHooks = {
  usePlans,
  usePermissionPlans, // Alias
  useAvailablePermissions,
  usePlanAnalytics,
  useWeb3AssignmentRules,
  useUserPlanMemberships,
  usePlanAssignmentHistory,
  usePlanMembers,
};

export default planHooks;