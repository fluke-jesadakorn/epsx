/**
 * Server-side data fetching for Access Management
 * 
 * These functions are designed to be called from Server Components
 * to fetch initial data for the unified access management page.
 */

import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans';
import { createPromotionsClient, isApiSuccess as isPromoSuccess, type Promotion } from '@/shared/api/promotions';
import { createAdminApiClient } from '@/shared/utils/api-client';
import {
  type AccessPolicy,
  type PolicyStats,
  type PolicyType,
  DEFAULT_POLICY_STATS,
  planToPolicy,
  groupToPolicy,
} from '@/components/access-control/types';

// ============================================================================
// TYPES
// ============================================================================

export interface AccessManagementData {
  policies: AccessPolicy[];
  stats: PolicyStats;
  promotions: DisplayPromotion[];
  permissionCount: number;
  platformCount: number;
}

export interface DisplayPromotion {
  id: number;
  name: string;
  code: string;
  description?: string;
  discountType: 'percentage' | 'fixed';
  discountValue: number;
  maxDiscountAmount: number | null;
  minPurchaseAmount: number;
  usageLimit?: number;
  currentUsage: number;
  isActive: boolean;
  startDate: string;
  endDate?: string;
  applicablePlans: string[];
  totalRevenue: number;
  conversionRate: number;
}

export interface PermissionDefinitionDto {
  id: string;
  permission: string;
  platform: string;
  category?: string;
  description?: string;
  is_system?: boolean;
}

// ============================================================================
// API ROUTES
// ============================================================================

const API_ROUTES = {
  PERMISSIONS: {
    GROUPS: '/admin/permissions/groups',
    DEFINITIONS: '/admin/permissions/definitions',
    ANALYTICS: '/admin/permissions/analytics',
  },
} as const;

// ============================================================================
// SERVER-SIDE DATA FETCHERS
// ============================================================================

/**
 * Fetch all access policies (plans + groups) for server-side rendering
 */
export async function fetchPolicies(): Promise<AccessPolicy[]> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const plansClient = createPlansClient(apiClient);

    const [plansRes, groupsRes] = await Promise.all([
      plansClient.getPlans({ limit: 100 }),
      apiClient.get<{ groups: any[] }>(API_ROUTES.PERMISSIONS.GROUPS),
    ]);

    const policies: AccessPolicy[] = [];

    // Transform plans to policies
    if (isApiSuccess(plansRes)) {
      const backendResponse = plansRes.data as any;
      const plans: PlanResponse[] = backendResponse?.data?.plans || backendResponse?.plans || [];
      plans.forEach(plan => {
        policies.push(planToPolicy(plan));
      });
    }

    // Transform groups to policies (exclude subscription type - handled by plans)
    if (groupsRes.success && groupsRes.data) {
      const groups = groupsRes.data.groups || groupsRes.data || [];
      (Array.isArray(groups) ? groups : [])
        .filter((g: any) => g.group_type !== 'subscription')
        .forEach((group: any) => {
          policies.push(groupToPolicy(group));
        });
    }

    return policies;
  } catch (error) {
    console.error('[fetchPolicies] Error:', error);
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
      plansClient.getPlans({ limit: 100 }),
      apiClient.get<{ groups: any[] }>(API_ROUTES.PERMISSIONS.GROUPS),
      apiClient.get<any>(API_ROUTES.PERMISSIONS.ANALYTICS),
    ]);

    const stats: PolicyStats = { ...DEFAULT_POLICY_STATS };

    // Process plans
    if (isApiSuccess(plansRes)) {
      const backendResponse = plansRes.data as any;
      const plans: PlanResponse[] = backendResponse?.data?.plans || backendResponse?.plans || [];
      
      stats.byType.subscription = plans.length;
      stats.activeSubscriptions = plans.filter(p => p.is_active).length;
      
      // Calculate MRR
      stats.totalMRR = plans.reduce((sum, plan) => {
        const revenue = typeof plan.revenue_last_30_days === 'string'
          ? parseFloat(plan.revenue_last_30_days)
          : plan.revenue_last_30_days;
        return sum + (isNaN(revenue) ? 0 : revenue);
      }, 0);

      // Count subscribers
      const planMembers = plans.reduce((sum, p) => sum + (p.subscriber_count || 0), 0);
      stats.totalMembers += planMembers;
    }

    // Process groups (exclude subscription type)
    if (groupsRes.success && groupsRes.data) {
      const groups = groupsRes.data.groups || groupsRes.data || [];
      const groupsArray = Array.isArray(groups) ? groups : [];
      const nonSubGroups = groupsArray.filter((g: any) => g.group_type !== 'subscription');
      stats.activeGroups = nonSubGroups.filter((g: any) => g.is_active).length;

      // Count by type
      nonSubGroups.forEach((group: any) => {
        const typeMap: Record<string, PolicyType> = {
          manual: 'manual',
          web3_asset: 'web3_asset',
          dao_membership: 'dao',
          dao: 'dao',
          admin: 'system',
          system: 'system',
        };
        const policyType = typeMap[group.group_type] || 'manual';
        stats.byType[policyType] = (stats.byType[policyType] || 0) + 1;
      });
    }

    // Add analytics data
    if (analyticsRes.success && analyticsRes.data) {
      const analytics = analyticsRes.data;
      stats.totalMembers += analytics.total_active_memberships || 0;
      stats.expiringSoon = analytics.expiring_soon_count || 0;
    }

    // Calculate total policies
    stats.totalPolicies = Object.values(stats.byType).reduce((a, b) => a + b, 0);

    return stats;
  } catch (error) {
    console.error('[fetchPolicyStats] Error:', error);
    return DEFAULT_POLICY_STATS;
  }
}

/**
 * Fetch promotions for server-side rendering
 */
export async function fetchPromotions(): Promise<DisplayPromotion[]> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const promotionsClient = createPromotionsClient(apiClient);

    const response = await promotionsClient.getPromotions({ limit: 100 });

    if (isPromoSuccess(response)) {
      const promos = response.data?.promotions || [];
      return promos.map((p: Promotion) => ({
        id: p.id,
        name: p.name,
        code: p.code,
        description: p.description,
        discountType: p.discountType,
        discountValue: parseFloat(p.discountValue),
        maxDiscountAmount: p.maxDiscountAmount ? parseFloat(p.maxDiscountAmount) : null,
        minPurchaseAmount: parseFloat(p.minPurchaseAmount || '0'),
        usageLimit: p.usageLimit,
        currentUsage: p.currentUsage,
        isActive: p.isActive,
        startDate: p.startDate,
        endDate: p.endDate,
        applicablePlans: p.applicablePlans,
        totalRevenue: parseFloat(p.totalRevenue),
        conversionRate: p.conversionRate,
      }));
    }

    return [];
  } catch (error) {
    console.error('[fetchPromotions] Error:', error);
    return [];
  }
}

/**
 * Fetch permission definitions count and platform count
 */
export async function fetchPermissionStats(): Promise<{ count: number; platformCount: number }> {
  try {
    const apiClient = createAdminApiClient({ serverSide: true });
    const response = await apiClient.get<PermissionDefinitionDto[]>(API_ROUTES.PERMISSIONS.DEFINITIONS);

    if (response.success && response.data) {
      const definitions = Array.isArray(response.data) ? response.data : [];
      const platforms = new Set(definitions.map(d => d.platform));
      return {
        count: definitions.length,
        platformCount: platforms.size,
      };
    }

    return { count: 0, platformCount: 0 };
  } catch (error) {
    console.error('[fetchPermissionStats] Error:', error);
    return { count: 0, platformCount: 0 };
  }
}

/**
 * Fetch all access management data in parallel for server-side rendering
 */
export async function fetchAccessManagementData(): Promise<AccessManagementData> {
  const [policies, stats, promotions, permissionStats] = await Promise.all([
    fetchPolicies(),
    fetchPolicyStats(),
    fetchPromotions(),
    fetchPermissionStats(),
  ]);

  return {
    policies,
    stats,
    promotions,
    permissionCount: permissionStats.count,
    platformCount: permissionStats.platformCount,
  };
}
