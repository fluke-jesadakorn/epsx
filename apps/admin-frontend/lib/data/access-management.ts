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

import { redirectOnForbidden } from '@/lib/api-error';
import { type PermissionPlan } from '@/lib/api/plan-management-client';

// Constants
const WALLET_MGMT_ROUTE = '/wallet-management';

// Extended Plan type with analytics fields
interface PlanResponse extends Plan {
  revenue_last_30_days?: string | number;
  subscriber_count?: number;
}

interface GroupData extends Partial<PermissionPlan> {
  group_type?: string;
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
    redirectOnForbidden(response, WALLET_MGMT_ROUTE);

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
  } catch (e) {
    if (typeof e === 'object' && e !== null && 'digest' in e) { throw e; }
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
 * Helper to extract plans from API response
 */
function extractPlans(data: PlansApiResponse | null | undefined): PlanResponse[] {
  if (!data) {
    return [];
  }
  return data.data?.plans ?? data.plans ?? [];
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
    redirectOnForbidden(groupsRes, WALLET_MGMT_ROUTE);

    const policies: AccessPolicy[] = [];

    // Transform plans to policies
    if (isApiSuccess(plansRes)) {
      const plans = extractPlans(plansRes.data as PlansApiResponse);
      plans.forEach(plan => {
        policies.push(planToPolicy(plan));
      });
    }

    // Transform groups to policies (exclude subscription type - handled by plans)
    if (groupsRes.success && groupsRes.data) {
      const rawGroups = groupsRes.data.plans ?? groupsRes.data;
      const groupsArray = (Array.isArray(rawGroups) ? rawGroups : []) as unknown as GroupData[];
      groupsArray
        .filter((g) => g.group_type !== 'subscription')
        .forEach((group) => {
          // Add plan_type if missing (mapping group_type to plan_type for groupToPolicy)
          const planType = group.plan_type ?? group.group_type;
          const normalizedGroup = {
            ...group,
            plan_type: planType ?? 'manual'
          } as unknown as PermissionPlan;

          policies.push(groupToPolicy(normalizedGroup));
        });
    }

    return policies;
  } catch (e) {
    if (typeof e === 'object' && e !== null && 'digest' in e) { throw e; }
    return [];
  }
}

/**
 * Process plan statistics
 */
function processPlansStats(stats: PolicyStats, plans: PlanResponse[]): void {
  stats.byType.subscription = plans.length;
  stats.activeSubscriptions = plans.filter(p => p.is_active === true).length;

  // Calculate MRR
  stats.totalMRR = plans.reduce((sum, plan) => {
    const rawRevenue = plan.revenue_last_30_days;
    const revenue = typeof rawRevenue === 'string'
      ? parseFloat(rawRevenue)
      : (rawRevenue ?? 0);
    return sum + (isNaN(revenue) ? 0 : (revenue));
  }, 0);

  // Count subscribers
  const planMembers = plans.reduce((sum, p) => sum + (p.subscriber_count ?? 0), 0);
  stats.totalMembers += planMembers;
}

/**
 * Process group statistics
 */
function processGroupsStats(stats: PolicyStats, groupsResData: PlansApiResponse): void {
  const rawGroups = groupsResData.plans ?? groupsResData;
  const groupsArray = (Array.isArray(rawGroups) ? rawGroups : []) as unknown as GroupData[];
  const nonSubGroups = groupsArray.filter((g) => g.group_type !== 'subscription');
  stats.activeGroups = nonSubGroups.filter((g) => g.is_active === true).length;

  // Count by type
  nonSubGroups.forEach((group) => {
    const typeMap: Record<string, PolicyType | undefined> = {
      manual: 'manual',
      web3_asset: 'web3_asset',
      dao_membership: 'dao',
      dao: 'dao',
      admin: 'system',
      system: 'system',
    };
    const typeKey = group.group_type ?? group.plan_type ?? 'manual';
    const policyType = typeMap[typeKey] ?? 'manual';
    stats.byType[policyType] = stats.byType[policyType] + 1;
  });
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
    redirectOnForbidden(groupsRes, WALLET_MGMT_ROUTE);

    const stats: PolicyStats = { ...DEFAULT_POLICY_STATS };

    // Process plans
    if (isApiSuccess(plansRes)) {
      const plans = extractPlans(plansRes.data as PlansApiResponse);
      processPlansStats(stats, plans);
    }

    // Process groups (exclude subscription type)
    if (groupsRes.success && groupsRes.data) {
      processGroupsStats(stats, groupsRes.data);
    }

    // Add analytics data
    if (analyticsRes.success && analyticsRes.data) {
      const analytics = analyticsRes.data;
      stats.totalMembers += analytics.total_active_memberships ?? 0;
      stats.expiringSoon = analytics.expiring_soon_count ?? 0;
    }

    // Calculate total policies
    const total = Object.values(stats.byType).reduce((acc, count) => acc + count, 0);
    stats.totalPolicies = total;

    return stats;
  } catch (e) {
    if (typeof e === 'object' && e !== null && 'digest' in e) { throw e; }
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
    redirectOnForbidden(response, WALLET_MGMT_ROUTE);

    if (response.success && response.data) {
      const definitions = Array.isArray(response.data) ? response.data : [];
      const platforms = new Set(definitions.map(d => d.platform));
      return {
        count: definitions.length,
        platformCount: platforms.size,
      };
    }

    return { count: 0, platformCount: 0 };
  } catch (e) {
    // Let Next.js redirect errors propagate
    if (typeof e === 'object' && e !== null && 'digest' in e) { throw e; }
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
