/**
 * ADMIN PLAN MANAGEMENT CLIENT
 *
 * Re-exports types from shared and provides admin-specific plan operations.
 * This wrapper adds admin-frontend specific functionality on top of the shared PlansApi.
 */

'use client';

import {
  extractArray,
  extractArrayOrEmpty,
  extractData,
  extractObject,
} from '@/shared/api';
import { API_ROUTES } from '@/shared/config/route-constants';
import { adminApiClient } from '../api-client';

// Re-export shared types
export {
  createPlansClient, PlansApi, type AssignPlanRequest, type BulkAssignRequest,
  type BulkRemoveRequest, type PlanFilters, type PlanStats, type RemovePlanRequest, type CreatePlanRequest as SharedCreatePlanRequest, type Plan as SharedPlan,
  type PlanMembership as SharedPlanMembership, type UpdatePlanRequest as SharedUpdatePlanRequest
} from '@/shared_deploy/api/plans';

// ============================================================================
// ADMIN-SPECIFIC TYPES
// ============================================================================

/**
 * Permission Plan (Admin extended version)
 */
export interface PermissionPlan {
  id: string;
  name: string;
  slug: string;
  description: string;
  plan_type: string;
  permissions: string[];
  price?: number;
  currency?: string;
  billing_cycle?: string;
  is_active: boolean;
  is_system_plan?: boolean;
  is_promoted?: boolean;
  display_order?: number;
  max_members?: number | null;
  auto_assign_enabled?: boolean;
  plan_metadata?: Record<string, any>;
  default_expiry_days?: number;
  priority_level?: number;
  created_at: string;
  updated_at: string;
  member_count?: number;
}

export type Plan = PermissionPlan;

export interface UserPlanMembership {
  id: string;
  user_id: string;
  plan_id: string;
  granted_by: string;
  granted_at: string;
  expires_at: string | null;
  is_active: boolean;
  plan?: PermissionPlan;
}

export interface Web3AssignmentRule {
  id: string;
  plan_id: string;
  blockchain_network: 'bsc_mainnet' | 'bsc_testnet' | 'ethereum_mainnet' | 'polygon_mainnet' | 'arbitrum_mainnet' | 'optimism_mainnet';
  verification_type: 'nft_ownership' | 'token_balance' | 'dao_membership';
  contract_address?: string;
  token_id?: string;
  minimum_balance?: string;
  is_active: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
  plan?: PermissionPlan;
}

export interface PlanAssignmentHistory {
  id: string;
  user_id: string;
  user_email?: string;
  user_name?: string;
  plan_id: string;
  plan_name?: string;
  operation_type: 'assign' | 'remove' | 'expire' | 'cleanup';
  operation_source: 'manual' | 'web3_automatic' | 'system_cleanup' | 'bulk_operation';
  performed_by?: string;
  performed_by_name?: string;
  reason?: string;
  expires_at?: string;
  metadata?: Record<string, any>;
  created_at: string;
  plan?: PermissionPlan;
}

export interface PlanAnalytics {
  total_plans: number;
  total_active_memberships: number;
  expiring_soon_count: number;
  most_popular_plans: Array<{ plan_name: string; member_count: number }>;
  permission_distribution: Record<string, number>;
}

export interface AssignUserToPlanRequest {
  user_id: string;
  plan_id: string;
  expires_at?: string | null;
  reason?: string;
}

export interface CreatePlanRequest {
  name: string;
  permissions: string[];
  description?: string;
  default_expiry_days?: number;
  priority_level?: number;
}

export interface UpdatePlanRequest {
  name?: string;
  permissions?: string[];
  description?: string;
  default_expiry_days?: number;
  priority_level?: number;
}

export interface PermissionDefinitionDto {
  id: string;
  permission: string;
  name?: string | null;
  description?: string | null;
  platform: string;
  category?: string | null;
  is_system: boolean;
  is_active: boolean;
  created_at: string;
}

export interface CreatePermissionDefinitionRequest {
  permission: string;
  name?: string;
  description?: string;
  platform?: string;
  category?: string;
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function mapAssignmentToMembership(assignment: any): UserPlanMembership {
  return {
    id: assignment.id,
    user_id: assignment.wallet_address,
    plan_id: assignment.plan_id,
    granted_by: assignment.assigned_by || 'system',
    granted_at: assignment.assigned_at,
    expires_at: assignment.expires_at,
    is_active: assignment.is_active,
    plan: {
      id: assignment.plan_id,
      name: assignment.plan_name,
      slug: assignment.plan_slug || '',
      description: assignment.plan_description || assignment.plan_type || '',
      plan_type: assignment.plan_type || 'manual',
      permissions: [],
      is_active: true,
      created_at: assignment.assigned_at,
      updated_at: assignment.assigned_at,
      default_expiry_days: assignment.default_expiry_days,
      priority_level: assignment.priority_level,
    } as PermissionPlan
  };
}

// ============================================================================
// PLAN MANAGEMENT API
// ============================================================================

export const planMgmt = {
  async getPermissionPlans(): Promise<PermissionPlan[]> {
    const res = await adminApiClient.get<any>(API_ROUTES.PERMISSIONS.PLANS);
    return extractArray<PermissionPlan>(res, 'getPermissionPlans');
  },

  async getPermissionPlan(planId: string): Promise<PermissionPlan> {
    const res = await adminApiClient.get<any>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`);
    return extractObject<PermissionPlan>(res, 'getPermissionPlan');
  },

  async createPermissionPlan(req: CreatePlanRequest): Promise<PermissionPlan> {
    const slug = req.name
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-')
      .trim();

    const backendRequest = {
      name: req.name,
      slug: slug,
      description: req.description || '',
      plan_type: 'manual',
      permissions: req.permissions,
      display_order: req.priority_level,
    };

    const res = await adminApiClient.post<PermissionPlan>(API_ROUTES.PERMISSIONS.PLANS, backendRequest);
    return res.data!;
  },

  async updatePermissionPlan(planId: string, req: UpdatePlanRequest): Promise<PermissionPlan> {
    const res = await adminApiClient.put<PermissionPlan>(`${API_ROUTES.PERMISSIONS.PLANS}/${planId}`, req);
    return res.data!;
  },

  async deletePermissionPlan(planId: string): Promise<void> {
    await adminApiClient.delete(`/api/permissions/plans/${planId}`);
  },

  async getUserPlans(userId: string): Promise<UserPlanMembership[]> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      wallet_address: userId,
      is_active: true,
      limit: 100,
    });
    return extractArrayOrEmpty<any>(res).map(mapAssignmentToMembership);
  },

  async getUserPermissions(userId: string): Promise<string[]> {
    const res = await adminApiClient.get<string[]>(`/api/auth/web3/plans/permissions/${userId}`);
    return res.data!;
  },

  async assignUserToPlan(req: { user_id: string; plan_id: string; expires_at?: string | null; reason?: string }): Promise<void> {
    await adminApiClient.post(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      wallet_address: req.user_id,
      plan_id: req.plan_id,
      expires_at: req.expires_at,
      assignment_reason: req.reason,
      assignment_source: 'manual',
    });
  },

  async removeUserFromPlan(userId: string, planId: string): Promise<void> {
    const assignments = await this.getUserPlans(userId);
    const assignment = assignments.find(a => a.plan_id === planId);
    if (assignment) {
      await adminApiClient.delete(`${API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS}/${assignment.id}`);
    } else {
      console.warn(`Assignment not found for user ${userId} and plan ${planId}`);
    }
  },

  async getPlanMemberships(planId: string): Promise<UserPlanMembership[]> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      plan_id: planId,
      limit: 100,
      is_active: true
    });
    return extractArrayOrEmpty<any>(res).map(mapAssignmentToMembership);
  },

  async getWeb3AssignmentRules(): Promise<Web3AssignmentRule[]> {
    const res = await adminApiClient.get<Web3AssignmentRule[]>('/api/auth/web3/assignment/rules');
    return res.data!;
  },

  async createWeb3AssignmentRule(req: { plan_id: string; blockchain_network: string; verification_type: string; contract_address?: string; token_id?: string; minimum_balance?: string }): Promise<Web3AssignmentRule> {
    const res = await adminApiClient.post<Web3AssignmentRule>('/api/auth/web3/assignment/rules', req);
    return res.data!;
  },

  async deleteWeb3AssignmentRule(ruleId: string): Promise<void> {
    await adminApiClient.delete(`/api/auth/web3/assignment/rules/${ruleId}`);
  },

  async processWalletAssignment(req: { wallet_address: string }): Promise<string[]> {
    const res = await adminApiClient.post<string[]>('/api/auth/web3/assignment/process-wallet', req);
    return res.data!;
  },

  async verifyWalletAssets(walletAddress: string): Promise<any> {
    const res = await adminApiClient.post<any>('/api/auth/web3/assignment/verify-assets', { wallet_address: walletAddress });
    return res.data;
  },

  async bulkProcessWallets(req: { wallet_addresses: string[] }): Promise<any> {
    const res = await adminApiClient.post<any>('/api/auth/web3/assignment/bulk-process', req);
    return res.data;
  },

  async getPlanAssignmentHistory(filters?: { operation_type?: string; operation_source?: string; plan_id?: string; user_search?: string; date_from?: string; date_to?: string; limit?: number; offset?: number }): Promise<{ history: PlanAssignmentHistory[]; total: number }> {
    const params = filters ? Object.fromEntries(Object.entries(filters).filter(([, v]) => v !== undefined)) : undefined;
    if (params) {
      if (params.date_from && typeof params.date_from === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(params.date_from)) {
        params.date_from = `${params.date_from}T00:00:00Z`;
      }
      if (params.date_to && typeof params.date_to === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(params.date_to)) {
        params.date_to = `${params.date_to}T23:59:59Z`;
      }
    }
    const res = await adminApiClient.get<{ history: PlanAssignmentHistory[]; total: number }>('/api/admin/plans/history', params);
    return res.data!;
  },

  async cleanupExpiredMemberships(): Promise<{ removed_count: number }> {
    const res = await adminApiClient.post<{ removed_count: number }>('/api/admin/plans/cleanup-expired', {});
    return res.data!;
  },

  async listUsers(params?: { page?: number; limit?: number; search?: string; tier?: string; status?: string; sort_by?: string; sort_order?: string }): Promise<any> {
    const res = await adminApiClient.get<any>('/api/admin/users', params);
    return res.data;
  },

  async searchUsers(query: string, limit = 10, excludePlanId?: string): Promise<Array<{ wallet_address: string; user_id?: string; tier?: string; permissions?: string[]; plans?: string[] }>> {
    const queryLower = query.toLowerCase();
    try {
      const params = new URLSearchParams({ search: query, limit: '50' });
      if (excludePlanId) params.append('exclude_plan_id', excludePlanId);
      const apiUrl = `/api/admin/wallets/search?${params.toString()}`;
      const res = await adminApiClient.get<any>(apiUrl);
      const wallets = res.data?.wallets || res.data?.data?.wallets || res.data || [];
      if (Array.isArray(wallets) && wallets.length > 0) {
        return wallets
          .filter((wallet: any) => wallet.wallet_address?.toLowerCase().includes(queryLower))
          .slice(0, limit)
          .map((wallet: any) => ({
            wallet_address: wallet.wallet_address,
            user_id: wallet.wallet_address,
            tier: wallet.tier,
            permissions: wallet.permissions?.map((p: any) => p.permission || p) || [],
            plans: wallet.plans?.map((g: any) => g.plan_name || g) || [],
          }));
      }
      return [];
    } catch (error) {
      console.warn('Wallet search failed:', error);
      return [];
    }
  },

  async getUser(walletAddress: string): Promise<any> {
    const res = await adminApiClient.get<any>(`/api/admin/users/${walletAddress}`);
    return res.data;
  },

  async updateUser(walletAddress: string, updates: { is_active?: boolean; metadata?: any }): Promise<any> {
    const res = await adminApiClient.put<any>(`/api/admin/users/${walletAddress}`, updates);
    return res.data;
  },

  async getUserStats(): Promise<any> {
    const res = await adminApiClient.get<any>('/api/admin/wallets/stats');
    return res.data;
  },

  async getPlatformOverview(period?: string): Promise<any> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.ANALYTICS_OVERVIEW, period ? { period } : undefined);
    return res.data;
  },

  async getUserAnalytics(period?: string): Promise<any> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.ANALYTICS_USERS, period ? { period } : undefined);
    return res.data;
  },

  async getPermissionAnalytics(): Promise<any> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS);
    return res.data;
  },

  async getRevenueAnalytics(period?: string): Promise<any> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.ANALYTICS_REVENUE, period ? { period } : undefined);
    return res.data;
  },

  async getPlanAnalytics(): Promise<PlanAnalytics> {
    const response = await adminApiClient.get<any>(API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS);
    const data = extractData<any>(response) || {};
    return {
      total_plans: data.total_plans || 0,
      total_active_memberships: data.active_permissions || data.total_permissions || 0,
      expiring_soon_count: data.expiring_permissions?.length || 0,
      most_popular_plans: data.plan_membership?.slice(0, 3).map((g: any) => ({ plan_name: g.plan_name, member_count: g.member_count })) || [],
      permission_distribution: data.permission_usage?.reduce((acc: any, p: any) => { acc[p.permission] = p.users_count; return acc; }, {}) || {},
    };
  },

  async getExpiringMemberships(days = 7): Promise<UserPlanMembership[]> {
    const res = await adminApiClient.get<any>('/api/permissions/assignments/expiring', { days });
    // Note: This endpoint returns { assignments: [...] } directly, so use extractData
    const data = extractData<{ assignments?: any[] }>(res);
    const assignments = data?.assignments || [];
    return assignments.map(mapAssignmentToMembership);
  },

  async checkUserPermission(userId: string, permission: string): Promise<boolean> {
    const res = await adminApiClient.get<{ has_permission: boolean }>(`/api/admin/users/${userId}/check-permission`, { permission });
    return res.data!.has_permission;
  },

  async getAvailablePermissions(): Promise<string[]> {
    const res = await adminApiClient.get<any>('/api/admin/permissions/available');
    return extractArrayOrEmpty<string>(res);
  },

  async getPermissionDefinitions(): Promise<PermissionDefinitionDto[]> {
    const res = await adminApiClient.get<any>('/api/permissions/definitions');
    return extractArrayOrEmpty<PermissionDefinitionDto>(res);
  },

  async createPermissionDefinition(req: CreatePermissionDefinitionRequest): Promise<PermissionDefinitionDto> {
    const res = await adminApiClient.post<any>('/api/permissions/definitions', req);
    return extractObject<PermissionDefinitionDto>(res, 'createPermissionDefinition');
  },

  async deletePermissionDefinition(id: string): Promise<void> {
    await adminApiClient.delete(`/api/permissions/definitions/${id}`);
  },

  async deletePermissionByName(permission: string): Promise<void> {
    const encoded = encodeURIComponent(permission);
    await adminApiClient.delete(`/api/permissions/definitions/by-name/${encoded}`);
  },

  // Aliases
  getPlans: function () { return this.getPermissionPlans(); },
  getPlan: function (planId: string) { return this.getPermissionPlan(planId); },
  createPlan: function (req: CreatePlanRequest) { return this.createPermissionPlan(req); },
  updatePlan: function (planId: string, req: UpdatePlanRequest) { return this.updatePermissionPlan(planId, req); },
  deletePlan: function (planId: string) { return this.deletePermissionPlan(planId); },
};
