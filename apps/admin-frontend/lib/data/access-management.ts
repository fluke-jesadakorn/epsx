/**
 * Server-side data fetching for Access Management
 * 
 * These functions are designed to be called from Server Components
 * to fetch initial data for the unified access management page.
 */

import {
  DEFAULT_POLICY_STATS,
  groupToPolicy,
  planToPolicy,
  type AccessPolicy,
  type PolicyStats,
  type PolicyType,
} from '@/components/access-control/types';
import { createPlansClient, type Plan } from '@/shared/api/plans';
import { isApiSuccess } from '@/shared/types/api';
import { createAdminApiClient } from '@/shared/utils/api-client';

// Extended Plan type with analytics fields
interface PlanResponse extends Plan {
  revenue_last_30_days?: string | number;
  subscriber_count?: number;
}

interface GroupData {
  group_type: string;
  is_active?: boolean;
}

interface PlansApiResponse {
  plans?: PlanResponse[];
  data?: {
    plans?: PlanResponse[];
  };
}

interface AnalyticsData {
  total_active_memberships?: number;
  expiring_soon_count?: number;
}

// ============================================================================
// TYPES
// ============================================================================

export interface AccessManagementData {
  policies: AccessPolicy[];
  stats: PolicyStats;
  permissionCount: number;
  platformCount: number;
}

export interface PermissionDefinitionDto {
  id: string;
  permission: string;
  platform: string;
  category?: string;
  description?: string;
  is_system?: boolean;
}

export interface WalletStats {
  total_users: number;
  active_users: number;
  inactive_users: number;
  new_users_30_days: number;
  active_users_30_days: number;
  growth_rate: number;
}

// ============================================================================
// API ROUTES
// ============================================================================

const API_ROUTES_LOCAL = {
  PERMISSIONS: {
    PLANS: '/api/permissions/plans',
    DEFINITIONS: '/api/permissions/definitions',
    ANALYTICS: '/api/admin/analytics/permissions',
  },
  WALLETS: {
    STATS: '/api/admin/wallets/stats',
  }
} as const;

// ============================================================================
// SERVER-SIDE DATA FETCHERS
// ============================================================================

/**
 * Fetch wallet statistics for server-side rendering
 */
export async function fetchWalletStats(): Promise<WalletStats> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const response = await apiClient.get<WalletStats>(API_ROUTES_LOCAL.WALLETS.STATS);

    if (response.success && response.data) {
      return response.data;
    }

    return {
      total_users: 0,
      active_users: 0,
      inactive_users: 0,
      new_users_30_days: 0,
      active_users_30_days: 0,
      growth_rate: 0
    };
  } catch {
    // Silently fail
    return {
      total_users: 0,
      active_users: 0,
      inactive_users: 0,
      new_users_30_days: 0,
      active_users_30_days: 0,
      growth_rate: 0
    };
  }
}

/**
 * Fetch all access policies (plans + groups) for server-side rendering
 */
export async function fetchPolicies(): Promise<AccessPolicy[]> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const plansClient = createPlansClient(apiClient);

    const [plansRes, groupsRes] = await Promise.all([
      plansClient.listPlans({ limit: 100 }),
      apiClient.get<PlansApiResponse>(API_ROUTES_LOCAL.PERMISSIONS.PLANS),
    ]);

    const policies: AccessPolicy[] = [];

    // Transform plans to policies
    if (isApiSuccess(plansRes)) {
      const backendResponse = plansRes.data as PlansApiResponse;
      const plans: PlanResponse[] = backendResponse?.data?.plans ?? backendResponse?.plans ?? [];
      plans.forEach(plan => {
        policies.push(planToPolicy(plan));
      });
    }

    // Transform groups to policies (exclude subscription type - handled by plans)
    if (groupsRes.success && groupsRes.data) {
      const groups = groupsRes.data.plans ?? groupsRes.data ?? [];
      const groupsArray = Array.isArray(groups) ? groups : [];
      groupsArray
        .filter((g: GroupData) => g.group_type !== 'subscription')
        .forEach((group: GroupData) => {
          policies.push(groupToPolicy(group));
        });
    }

    return policies;
  } catch {
    // Silently fail
    return [];
  }
}

/**
 * Fetch policy stats for server-side rendering
 */
export async function fetchPolicyStats(): Promise<PolicyStats> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const plansClient = createPlansClient(apiClient);

    const [plansRes, groupsRes, analyticsRes] = await Promise.all([
      plansClient.listPlans({ limit: 100 }),
      apiClient.get<PlansApiResponse>(API_ROUTES_LOCAL.PERMISSIONS.PLANS),
      apiClient.get<AnalyticsData>(API_ROUTES_LOCAL.PERMISSIONS.ANALYTICS),
    ]);

    const stats: PolicyStats = { ...DEFAULT_POLICY_STATS };

    // Process plans
    if (isApiSuccess(plansRes)) {
      const backendResponse = plansRes.data as PlansApiResponse;
      const plans: PlanResponse[] = backendResponse?.data?.plans ?? backendResponse?.plans ?? [];

      stats.byType.subscription = plans.length;
      stats.activeSubscriptions = plans.filter(p => p.is_active).length;

      // Calculate MRR
      stats.totalMRR = plans.reduce((sum, plan) => {
        const revenue = typeof plan.revenue_last_30_days === 'string'
          ? parseFloat(plan.revenue_last_30_days)
          : plan.revenue_last_30_days ?? 0;
        return sum + (isNaN(revenue as number) ? 0 : (revenue as number));
      }, 0);

      // Count subscribers
      const planMembers = plans.reduce((sum, p) => sum + (p.subscriber_count ?? 0), 0);
      stats.totalMembers += planMembers;
    }

    // Process groups (exclude subscription type)
    if (groupsRes.success && groupsRes.data) {
      const groups = groupsRes.data.plans ?? groupsRes.data ?? [];
      const groupsArray = Array.isArray(groups) ? groups : [];
      const nonSubGroups = groupsArray.filter((g: GroupData) => g.group_type !== 'subscription');
      stats.activeGroups = nonSubGroups.filter((g: GroupData) => g.is_active).length;

      // Count by type
      nonSubGroups.forEach((group: GroupData) => {
        const typeMap: Record<string, PolicyType> = {
          manual: 'manual',
          web3_asset: 'web3_asset',
          dao_membership: 'dao',
          dao: 'dao',
          admin: 'system',
          system: 'system',
        };
        const policyType = typeMap[group.group_type] ?? 'manual';
        stats.byType[policyType] = (stats.byType[policyType] ?? 0) + 1;
      });
    }

    // Add analytics data
    if (analyticsRes.success && analyticsRes.data) {
      const analytics = analyticsRes.data as AnalyticsData;
      stats.totalMembers += analytics.total_active_memberships ?? 0;
      stats.expiringSoon = analytics.expiring_soon_count ?? 0;
    }

    // Calculate total policies
    stats.totalPolicies = Object.values(stats.byType).reduce((a, b) => a + b, 0);

    return stats;
  } catch {
    // Silently fail
    return DEFAULT_POLICY_STATS;
  }
}

/**
 * Fetch permission definitions count and platform count
 */
export async function fetchPermissionStats(): Promise<{ count: number; platformCount: number }> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const response = await apiClient.get<PermissionDefinitionDto[]>(API_ROUTES_LOCAL.PERMISSIONS.DEFINITIONS);

    if (response.success && response.data) {
      const definitions = Array.isArray(response.data) ? response.data : [];
      const platforms = new Set(definitions.map(d => d.platform));
      return {
        count: definitions.length,
        platformCount: platforms.size,
      };
    }

    return { count: 0, platformCount: 0 };
  } catch {
    // Silently fail
    return { count: 0, platformCount: 0 };
  }
}

/**
 * Fetch all access management data in parallel for server-side rendering
 */
export async function fetchAccessManagementData(): Promise<AccessManagementData> {
  const [policies, stats, permissionStats] = await Promise.all([
    fetchPolicies(),
    fetchPolicyStats(),
    fetchPermissionStats(),
  ]);

  return {
    policies,
    stats,
    permissionCount: permissionStats.count,
    platformCount: permissionStats.platformCount,
  };
}
