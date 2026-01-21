/**
 * UNIFIED ACCESS POLICY CLIENT
 * 
 * Wraps PlansAPIClient and groupMgmt to provide a unified interface
 * for managing both subscription plans and permission groups as "Access Policies"
 */

'use client';

import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { 
  groupMgmt, 
  type PermissionGroup, 
  type GroupAnalytics,
  type CreateGroupRequest,
  type UpdateGroupRequest,
} from './group-management-client';
import {
  type AccessPolicy,
  type PolicyStats,
  type PolicyFilters,
  type PolicyType,
  DEFAULT_POLICY_STATS,
  planToPolicy,
  groupToPolicy,
} from '@/components/access-control/types';

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
      plansClient.getPlans({ limit: 100 }),
      groupMgmt.getPermissionGroups(),
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
      .filter(g => g.group_type !== 'subscription')
      .forEach(group => {
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
      const group = await groupMgmt.getPermissionGroup(sourceId);
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
      plansClient.getPlans({ limit: 100 }),
      groupMgmt.getPermissionGroups(),
      groupMgmt.getGroupAnalytics(),
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
    const nonSubGroups = groups.filter(g => g.group_type !== 'subscription');
    stats.activeGroups = nonSubGroups.filter(g => g.is_active).length;

    // Count by type
    nonSubGroups.forEach(group => {
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
    return groupMgmt.createPermissionGroup(data);
  },

  /**
   * Update an existing group
   */
  async updateGroup(groupId: string, data: UpdateGroupRequest): Promise<PermissionGroup> {
    return groupMgmt.updatePermissionGroup(groupId, data);
  },

  /**
   * Delete a group
   */
  async deleteGroup(groupId: string): Promise<void> {
    return groupMgmt.deletePermissionGroup(groupId);
  },

  /**
   * Delete a policy by its unified ID
   */
  async deletePolicy(policyId: string): Promise<void> {
    const [sourceType, sourceId] = policyId.split('-', 2);

    if (sourceType === 'plan') {
      const apiClient = createAdminApiClient();
      const plansClient = createPlansClient(apiClient);
      await plansClient.deletePlan(sourceId);
      return;
    }

    if (sourceType === 'group') {
      await groupMgmt.deletePermissionGroup(sourceId);
      return;
    }

    throw new Error(`Unknown policy type: ${sourceType}`);
  },

  /**
   * Get group memberships
   */
  async getGroupMembers(groupId: string) {
    return groupMgmt.getGroupMemberships(groupId);
  },

  /**
   * Assign user to group
   */
  async assignUserToGroup(userId: string, groupId: string, expiresAt?: string, reason?: string) {
    return groupMgmt.assignUserToGroup({
      user_id: userId,
      group_id: groupId,
      expires_at: expiresAt,
      reason,
    });
  },

  /**
   * Remove user from group
   */
  async removeUserFromGroup(userId: string, groupId: string) {
    return groupMgmt.removeUserFromGroup(userId, groupId);
  },

  /**
   * Get expiring memberships
   */
  async getExpiringMemberships(days = 7) {
    return groupMgmt.getExpiringMemberships(days);
  },

  /**
   * Get analytics for a specific group
   */
  async getGroupAnalytics(): Promise<GroupAnalytics> {
    return groupMgmt.getGroupAnalytics();
  },
};

// Export type for the client
export type AccessPolicyClient = typeof accessPolicyClient;
