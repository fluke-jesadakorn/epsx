/**
 * UNIFIED ACCESS POLICY CLIENT
 * 
 * Wraps PlansAPIClient and groupMgmt to provide a unified interface
 * for managing both subscription plans and permission groups as "Access Policies"
 */

'use client';

import {
  DEFAULT_POLICY_STATS,
  groupToPolicy,
  planToPolicy,
  type AccessPolicy,
  type PolicyFilters,
  type PolicyStats,
  type PolicyType,
} from '@/components/access-control/types';
import { createPlansClient, type Plan } from '@/shared/api/plans';
import { isApiSuccess } from '@/shared/types/api';
import { createAdminApiClient } from '@/shared/utils/api-client';
import {
  planMgmt,
  type CreatePlanRequest as CreateGroupRequest,
  type PlanAnalytics as GroupAnalytics,
  type PermissionPlan as PermissionGroup,
  type UpdatePlanRequest as UpdateGroupRequest
} from './plan-management-client';

// Extended Plan type with analytics fields
interface PlanResponse extends Plan {
  revenue_last_30_days?: string | number;
  subscriber_count?: number;
}

// ============================================================================
// ACCESS POLICY CLIENT
// ============================================================================

export const accessPolicyClient = {
  /**
   * Fetch all policies (plans + groups) and transform to AccessPolicy[]
   */
  async getPolicies(): Promise<AccessPolicy[]> {
    const apiClient = createAdminApiClient();
    const plansClient = createPlansClient(apiClient);

    const [plansRes, groups] = await Promise.all([
      plansClient.listPlans({ limit: 100 }),
      planMgmt.getPlans(),
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
    groups
      .filter((g: PermissionGroup) => g.plan_type !== 'subscription')
      .forEach((group: PermissionGroup) => {
        policies.push(groupToPolicy(group));
      });

    return policies;
  },

  /**
   * Fetch a single policy by ID
   * ID format: "plan-{id}" or "group-{id}"
   */
  async getPolicy(policyId: string): Promise<AccessPolicy | null> {
    const [sourceType, sourceId] = policyId.split('-', 2) as [string, string];

    if (sourceType === 'plan') {
      const apiClient = createAdminApiClient();
      const plansClient = createPlansClient(apiClient);
      const res = await plansClient.getPlan(sourceId);

      if (isApiSuccess(res) && res.data) {
        return planToPolicy(res.data);
      }
      return null;
    }

    if (sourceType === 'group') {
      const group = await planMgmt.getPlan(sourceId);
      return groupToPolicy(group);
    }

    return null;
  },

  /**
   * Get aggregated stats for the dashboard
   */
  async getStats(): Promise<PolicyStats> {
    const apiClient = createAdminApiClient();
    const plansClient = createPlansClient(apiClient);

    const [plansRes, groups, analytics] = await Promise.all([
      plansClient.listPlans({ limit: 100 }),
      planMgmt.getPlans(),
      planMgmt.getPlanAnalytics(),
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
    const nonSubGroups = groups.filter((g: PermissionGroup) => g.plan_type !== 'subscription');
    stats.activeGroups = nonSubGroups.filter((g: PermissionGroup) => g.is_active).length;

    // Count by type
    nonSubGroups.forEach((group: PermissionGroup) => {
      const typeMap: Record<string, PolicyType> = {
        manual: 'manual',
        web3_asset: 'web3_asset',
        dao_membership: 'dao',
        dao: 'dao',
        admin: 'system',
        system: 'system',
      };
      const policyType = typeMap[group.plan_type] || 'manual';
      stats.byType[policyType] = (stats.byType[policyType] || 0) + 1;
    });

    // Add group members from analytics
    stats.totalMembers += analytics.total_active_memberships || 0;
    stats.expiringSoon = analytics.expiring_soon_count || 0;

    // Calculate total policies
    stats.totalPolicies = Object.values(stats.byType).reduce((a, b) => a + b, 0);

    return stats;
  },

  /**
   * Filter and sort policies
   */
  filterPolicies(policies: AccessPolicy[], filters: PolicyFilters): AccessPolicy[] {
    let result = [...policies];

    // Search filter
    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      result = result.filter(policy =>
        policy.name.toLowerCase().includes(searchLower) ||
        policy.description.toLowerCase().includes(searchLower) ||
        policy.permissions.some(p => p.toLowerCase().includes(searchLower))
      );
    }

    // Type filter
    if (filters.types !== 'all') {
      result = result.filter(policy => filters.types.includes(policy.type));
    }

    // Status filter
    if (filters.status !== 'all') {
      const isActive = filters.status === 'active';
      result = result.filter(policy => policy.isActive === isActive);
    }

    // Sort
    result.sort((a, b) => {
      let comparison = 0;

      switch (filters.sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'members':
          comparison = a.memberCount - b.memberCount;
          break;
        case 'created_at':
          comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
          break;
        case 'revenue':
          comparison = (a.revenue || 0) - (b.revenue || 0);
          break;
        case 'type':
          comparison = a.type.localeCompare(b.type);
          break;
      }

      return filters.sortOrder === 'asc' ? comparison : -comparison;
    });

    return result;
  },

  // ============================================================================
  // MUTATIONS - Delegate to original clients
  // ============================================================================

  /**
   * Create a new group (manual, web3, dao, system)
   */
  async createGroup(data: CreateGroupRequest): Promise<PermissionGroup> {
    return planMgmt.createPlan(data);
  },

  /**
   * Update an existing group
   */
  async updateGroup(planId: string, data: UpdateGroupRequest): Promise<PermissionGroup> {
    return planMgmt.updatePlan(planId, data);
  },

  /**
   * Delete a group
   */
  async deleteGroup(planId: string): Promise<void> {
    return planMgmt.deletePlan(planId);
  },

  /**
   * Delete a policy by its unified ID
   */
  async deletePolicy(policyId: string): Promise<void> {
    const [sourceType, sourceId] = policyId.split('-', 2);
    if (!sourceId) return;

    if (sourceType === 'plan') {
      const apiClient = createAdminApiClient();
      const plansClient = createPlansClient(apiClient);
      await plansClient.deletePlan(sourceId);
      return;
    }

    if (sourceType === 'group') {
      await planMgmt.deletePlan(sourceId);
      return;
    }

    throw new Error(`Unknown policy type: ${sourceType}`);
  },

  /**
   * Update a policy (plan or group) by its unified ID
   */
  async updatePolicy(policyId: string, updates: { permissions?: string[]; name?: string; description?: string }): Promise<void> {
    const [sourceType, sourceId] = policyId.split('-', 2);
    if (!sourceId) return;

    if (sourceType === 'plan') {
      const apiClient = createAdminApiClient();
      const plansClient = createPlansClient(apiClient);
      await plansClient.updatePlan(sourceId, {
        name: updates.name,
        description: updates.description,
        permissions: updates.permissions
      });
      return;
    }

    if (sourceType === 'group') {
      await planMgmt.updatePlan(sourceId, {
        name: updates.name,
        description: updates.description,
        permissions: updates.permissions
      });
      return;
    }

    throw new Error(`Unknown policy type: ${sourceType}`);
  },

  /**
   * Get group memberships
   */
  async getGroupMembers(planId: string) {
    return planMgmt.getPlanMemberships(planId);
  },

  /**
   * Assign user to group
   */
  async assignUserToGroup(userId: string, planId: string, expiresAt?: string, reason?: string) {
    return planMgmt.assignUserToPlan({
      user_id: userId,
      plan_id: planId,
      expires_at: expiresAt,
      reason,
    });
  },

  /**
   * Remove user from group
   */
  async removeUserFromGroup(userId: string, planId: string) {
    return planMgmt.removeUserFromPlan(userId, planId);
  },

  /**
   * Get expiring memberships
   */
  async getExpiringMemberships(days = 7) {
    return planMgmt.getExpiringMemberships(days);
  },

  /**
   * Get analytics for a specific group
   */
  async getGroupAnalytics(): Promise<GroupAnalytics> {
    return planMgmt.getPlanAnalytics();
  },
};

// Export type for the client
export type AccessPolicyClient = typeof accessPolicyClient;
