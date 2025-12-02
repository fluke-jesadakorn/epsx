/**
 * Group Management API
 * Direct fetch to backend
 */

'use client';

import { apiDelete, apiGet, apiPost, apiPut } from '../api-fetch';
import { API_ROUTES } from '../../../../shared/config/route-constants';

export interface PermissionGroup {
  id: string;
  name: string;
  permissions: string[];
  is_system_group: boolean;
  default_expiry_days: number | null;
  priority_level: number;
  description?: string;
  created_at: string;
  updated_at: string;
  member_count?: number;
}

export interface UserGroupMembership {
  id: string;
  user_id: string;
  group_id: string;
  granted_by: string;
  granted_at: string;
  expires_at: string | null;
  is_active: boolean;
  group?: PermissionGroup;
}

export interface Web3AssignmentRule {
  id: string;
  group_id: string;
  blockchain_network:
    | 'bsc_mainnet'
    | 'bsc_testnet'
    | 'ethereum_mainnet'
    | 'polygon_mainnet'
    | 'arbitrum_mainnet'
    | 'optimism_mainnet';
  verification_type: 'nft_ownership' | 'token_balance' | 'dao_membership';
  contract_address?: string;
  token_id?: string;
  minimum_balance?: string;
  is_active: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
  group?: PermissionGroup;
}

export interface GroupAssignmentHistory {
  id: string;
  user_id: string;
  user_email?: string;
  user_name?: string;
  group_id: string;
  group_name?: string;
  operation_type: 'assign' | 'remove' | 'expire' | 'cleanup';
  operation_source:
    | 'manual'
    | 'web3_automatic'
    | 'system_cleanup'
    | 'bulk_operation';
  performed_by?: string;
  performed_by_name?: string;
  reason?: string;
  expires_at?: string;
  metadata?: Record<string, any>;
  created_at: string;
  group?: PermissionGroup;
}

export interface GroupAnalytics {
  total_groups: number;
  total_active_memberships: number;
  expiring_soon_count: number;
  most_popular_groups: Array<{ group_name: string; member_count: number }>;
  permission_distribution: Record<string, number>;
}

export const groupMgmt = {
  async getPermissionGroups(): Promise<PermissionGroup[]> {
    const res = await apiGet<any>(API_ROUTES.PERMISSIONS.GROUPS);
    const groups = res.data || res;
    if (!Array.isArray(groups)) throw new Error('Invalid response');
    return groups;
  },

  async getPermissionGroup(groupId: string): Promise<PermissionGroup> {
    return apiGet(`/api/admin/permissions/groups/${groupId}`);
  },

  async createPermissionGroup(req: {
    name: string;
    permissions: string[];
    description?: string;
    default_expiry_days?: number;
    priority_level?: number;
  }): Promise<PermissionGroup> {
    return apiPost(API_ROUTES.PERMISSIONS.GROUPS, req);
  },

  async updatePermissionGroup(
    groupId: string,
    req: {
      name?: string;
      permissions?: string[];
      description?: string;
      default_expiry_days?: number;
      priority_level?: number;
    }
  ): Promise<PermissionGroup> {
    return apiPut(`/api/admin/permissions/groups/${groupId}`, req);
  },

  async deletePermissionGroup(groupId: string): Promise<void> {
    return apiDelete(`/api/admin/permissions/groups/${groupId}`);
  },

  async getUserGroups(userId: string): Promise<UserGroupMembership[]> {
    return apiGet(`/api/admin/wallets/${userId}/assignments`);
  },

  async getUserPermissions(userId: string): Promise<string[]> {
    return apiGet(`/api/auth/web3/groups/permissions/${userId}`);
  },

  async assignUserToGroup(req: {
    user_id: string;
    group_id: string;
    expires_at?: string | null;
    reason?: string;
  }): Promise<void> {
    return apiPost(API_ROUTES.PERMISSIONS.ASSIGNMENTS, {
      wallet_address: req.user_id,
      group_id: req.group_id,
      expires_at: req.expires_at,
      assignment_reason: req.reason,
      assignment_source: 'manual',
    });
  },

  async removeUserFromGroup(userId: string, groupId: string): Promise<void> {
    return apiDelete(`/api/admin/wallet-assignments/${userId}/${groupId}`);
  },

  async getGroupMemberships(groupId: string): Promise<UserGroupMembership[]> {
    return apiGet(`/api/admin/group-memberships/${groupId}`);
  },

  async getWeb3AssignmentRules(): Promise<Web3AssignmentRule[]> {
    return apiGet('/api/auth/web3/assignment/rules');
  },

  async createWeb3AssignmentRule(req: {
    group_id: string;
    blockchain_network: string;
    verification_type: string;
    contract_address?: string;
    token_id?: string;
    minimum_balance?: string;
  }): Promise<Web3AssignmentRule> {
    return apiPost('/api/auth/web3/assignment/rules', req);
  },

  async deleteWeb3AssignmentRule(ruleId: string): Promise<void> {
    return apiDelete(`/api/auth/web3/assignment/rules/${ruleId}`);
  },

  async processWalletAssignment(req: {
    wallet_address: string;
  }): Promise<string[]> {
    return apiPost('/api/auth/web3/assignment/process-wallet', req);
  },

  async verifyWalletAssets(walletAddress: string): Promise<any> {
    return apiPost('/api/auth/web3/assignment/verify-assets', {
      wallet_address: walletAddress,
    });
  },

  async bulkProcessWallets(req: { wallet_addresses: string[] }): Promise<any> {
    return apiPost('/api/auth/web3/assignment/bulk-process', req);
  },

  async getGroupAssignmentHistory(filters?: {
    operation_type?: string;
    operation_source?: string;
    group_id?: string;
    user_search?: string;
    date_from?: string;
    date_to?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ history: GroupAssignmentHistory[]; total: number }> {
    const params = new URLSearchParams();
    if (filters?.operation_type)
      params.set('operation_type', filters.operation_type);
    if (filters?.operation_source)
      params.set('operation_source', filters.operation_source);
    if (filters?.group_id) params.set('group_id', filters.group_id);
    if (filters?.user_search) params.set('user_search', filters.user_search);
    if (filters?.date_from) params.set('date_from', filters.date_from);
    if (filters?.date_to) params.set('date_to', filters.date_to);
    if (filters?.limit) params.set('limit', filters.limit.toString());
    if (filters?.offset) params.set('offset', filters.offset.toString());
    return apiGet(`/api/admin/groups/history?${params}`);
  },

  async cleanupExpiredMemberships(): Promise<{ removed_count: number }> {
    return apiPost('/api/admin/groups/cleanup-expired', {});
  },

  async listUsers(params?: {
    page?: number;
    limit?: number;
    search?: string;
    tier?: string;
    status?: string;
    sort_by?: string;
    sort_order?: string;
  }): Promise<any> {
    const q = new URLSearchParams();
    if (params?.page) q.set('page', params.page.toString());
    if (params?.limit) q.set('limit', params.limit.toString());
    if (params?.search) q.set('search', params.search);
    if (params?.tier) q.set('tier', params.tier);
    if (params?.status) q.set('status', params.status);
    if (params?.sort_by) q.set('sort_by', params.sort_by);
    if (params?.sort_order) q.set('sort_order', params.sort_order);
    return apiGet(`/api/admin/users?${q}`);
  },

  async getUser(walletAddress: string): Promise<any> {
    return apiGet(`/api/admin/users/${walletAddress}`);
  },

  async updateUser(
    walletAddress: string,
    updates: {
      is_active?: boolean;
      metadata?: any;
    }
  ): Promise<any> {
    return apiPut(`/api/admin/users/${walletAddress}`, updates);
  },

  async getUserStats(): Promise<any> {
    return apiGet('/api/admin/wallets/stats');
  },

  async getPlatformOverview(period?: string): Promise<any> {
    return apiGet(
      `${API_ROUTES.ADMIN.ANALYTICS_OVERVIEW}${period ? `?period=${period}` : ''}`
    );
  },

  async getUserAnalytics(period?: string): Promise<any> {
    return apiGet(
      `${API_ROUTES.ADMIN.ANALYTICS_USERS}${period ? `?period=${period}` : ''}`
    );
  },

  async getPermissionAnalytics(): Promise<any> {
    return apiGet(API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS);
  },

  async getRevenueAnalytics(period?: string): Promise<any> {
    return apiGet(
      `${API_ROUTES.ADMIN.ANALYTICS_REVENUE}${period ? `?period=${period}` : ''}`
    );
  },

  async getGroupAnalytics(): Promise<GroupAnalytics> {
    const response = await apiGet<any>(API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS);
    const data = response.data || response; // Handle both nested and direct response formats
    return {
      total_groups: data.total_groups || 0,
      total_active_memberships: data.active_permissions || data.total_permissions || 0,
      expiring_soon_count: data.expiring_permissions?.length || 0,
      most_popular_groups:
        data.group_membership?.slice(0, 3).map((g: any) => ({
          group_name: g.group_name,
          member_count: g.member_count,
        })) || [],
      permission_distribution:
        data.permission_usage?.reduce((acc: any, p: any) => {
          acc[p.permission] = p.users_count;
          return acc;
        }, {}) || {},
    };
  },

  async getExpiringMemberships(days = 7): Promise<UserGroupMembership[]> {
    return apiGet(`/api/admin/groups/expiring-memberships?days=${days}`);
  },

  async checkUserPermission(
    userId: string,
    permission: string
  ): Promise<boolean> {
    const res = await apiGet<{ has_permission: boolean }>(
      `/api/admin/users/${userId}/check-permission?permission=${permission}`
    );
    return res.has_permission;
  },

  async getAvailablePermissions(): Promise<string[]> {
    return apiGet('/api/admin/permissions/available');
  },
};
