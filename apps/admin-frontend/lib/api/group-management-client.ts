/**
 * ADMIN GROUP MANAGEMENT CLIENT
 *
 * Re-exports types from shared and provides admin-specific group operations.
 * This wrapper adds admin-frontend specific functionality on top of the shared GroupsApi.
 */

'use client';

import { API_ROUTES } from '@/shared/config/route-constants';
import { adminApiClient } from '../api-client';

// Re-export shared types
export {
  createGroupsClient, GroupsApi, type AssignGroupRequest, type BulkAssignRequest,
  type BulkRemoveRequest, type GroupFilters, type GroupStats, type RemoveGroupRequest, type CreateGroupRequest as SharedCreateGroupRequest, type Group as SharedGroup,
  type GroupMembership as SharedGroupMembership, type UpdateGroupRequest as SharedUpdateGroupRequest
} from '@/shared/api/groups';

// ============================================================================
// ADMIN-SPECIFIC TYPES
// ============================================================================

/**
 * Permission Group (Admin extended version)
 */
export interface PermissionGroup {
  id: string;
  name: string;
  slug: string;
  description: string;
  group_type: string;
  permissions: string[];
  price?: number;
  currency?: string;
  billing_cycle?: string;
  is_active: boolean;
  is_system_group?: boolean;
  is_promoted?: boolean;
  display_order?: number;
  max_members?: number | null;
  auto_assign_enabled?: boolean;
  group_metadata?: Record<string, any>;
  default_expiry_days?: number;
  priority_level?: number;
  created_at: string;
  updated_at: string;
  member_count?: number;
}

export type Group = PermissionGroup;

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
  blockchain_network: 'bsc_mainnet' | 'bsc_testnet' | 'ethereum_mainnet' | 'polygon_mainnet' | 'arbitrum_mainnet' | 'optimism_mainnet';
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
  operation_source: 'manual' | 'web3_automatic' | 'system_cleanup' | 'bulk_operation';
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

export interface AssignUserToGroupRequest {
  user_id: string;
  group_id: string;
  expires_at?: string | null;
  reason?: string;
}

export interface CreateGroupRequest {
  name: string;
  permissions: string[];
  description?: string;
  default_expiry_days?: number;
  priority_level?: number;
}

export interface UpdateGroupRequest {
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

function mapAssignmentToMembership(assignment: any): UserGroupMembership {
  return {
    id: assignment.id,
    user_id: assignment.wallet_address,
    group_id: assignment.group_id,
    granted_by: assignment.assigned_by || 'system',
    granted_at: assignment.assigned_at,
    expires_at: assignment.expires_at,
    is_active: assignment.is_active,
    group: {
      id: assignment.group_id,
      name: assignment.group_name,
      slug: assignment.group_slug || '',
      description: assignment.group_description || assignment.group_type || '',
      group_type: assignment.group_type || 'manual',
      permissions: [],
      is_active: true,
      created_at: assignment.assigned_at,
      updated_at: assignment.assigned_at,
      default_expiry_days: assignment.default_expiry_days,
      priority_level: assignment.priority_level,
    } as PermissionGroup
  };
}

// ============================================================================
// GROUP MANAGEMENT API
// ============================================================================

export const groupMgmt = {
  async getPermissionGroups(): Promise<PermissionGroup[]> {
    const res = await adminApiClient.get<any>(API_ROUTES.PERMISSIONS.GROUPS);
    const groups = res.data?.data || res.data;
    if (!Array.isArray(groups)) { throw new Error('Invalid response'); }
    return groups;
  },

  async getPermissionGroup(groupId: string): Promise<PermissionGroup> {
    const res = await adminApiClient.get<any>(`${API_ROUTES.PERMISSIONS.GROUPS}/${groupId}`);
    const group = res.data?.data || res.data;
    if (!group) { throw new Error('Group not found'); }
    return group;
  },

  async createPermissionGroup(req: CreateGroupRequest): Promise<PermissionGroup> {
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
      group_type: 'manual',
      permissions: req.permissions,
      display_order: req.priority_level,
    };

    const res = await adminApiClient.post<PermissionGroup>(API_ROUTES.PERMISSIONS.GROUPS, backendRequest);
    return res.data!;
  },

  async updatePermissionGroup(groupId: string, req: UpdateGroupRequest): Promise<PermissionGroup> {
    const res = await adminApiClient.put<PermissionGroup>(`${API_ROUTES.PERMISSIONS.GROUPS}/${groupId}`, req);
    return res.data!;
  },

  async deletePermissionGroup(groupId: string): Promise<void> {
    await adminApiClient.delete(`/api/permissions/groups/${groupId}`);
  },

  async getUserGroups(userId: string): Promise<UserGroupMembership[]> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      wallet_address: userId,
      is_active: true,
      limit: 100,
    });
    const assignments = res.data?.data || res.data || [];
    if (!Array.isArray(assignments)) { return []; }
    return assignments.map(mapAssignmentToMembership);
  },

  async getUserPermissions(userId: string): Promise<string[]> {
    const res = await adminApiClient.get<string[]>(`/api/auth/web3/groups/permissions/${userId}`);
    return res.data!;
  },

  async assignUserToGroup(req: { user_id: string; group_id: string; expires_at?: string | null; reason?: string }): Promise<void> {
    await adminApiClient.post(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      wallet_address: req.user_id,
      group_id: req.group_id,
      expires_at: req.expires_at,
      assignment_reason: req.reason,
      assignment_source: 'manual',
    });
  },

  async removeUserFromGroup(userId: string, groupId: string): Promise<void> {
    const assignments = await this.getUserGroups(userId);
    const assignment = assignments.find(a => a.group_id === groupId);
    if (assignment) {
      await adminApiClient.delete(`${API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS}/${assignment.id}`);
    } else {
      console.warn(`Assignment not found for user ${userId} and group ${groupId}`);
    }
  },

  async getGroupMemberships(groupId: string): Promise<UserGroupMembership[]> {
    const res = await adminApiClient.get<any>(API_ROUTES.ADMIN.PERMISSION_ASSIGNMENTS, {
      group_id: groupId,
      limit: 100,
      is_active: true
    });
    const assignments = res.data?.data || res.data || [];
    if (!Array.isArray(assignments)) { return []; }
    return assignments.map(mapAssignmentToMembership);
  },

  async getWeb3AssignmentRules(): Promise<Web3AssignmentRule[]> {
    const res = await adminApiClient.get<Web3AssignmentRule[]>('/api/auth/web3/assignment/rules');
    return res.data!;
  },

  async createWeb3AssignmentRule(req: { group_id: string; blockchain_network: string; verification_type: string; contract_address?: string; token_id?: string; minimum_balance?: string }): Promise<Web3AssignmentRule> {
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

  async getGroupAssignmentHistory(filters?: { operation_type?: string; operation_source?: string; group_id?: string; user_search?: string; date_from?: string; date_to?: string; limit?: number; offset?: number }): Promise<{ history: GroupAssignmentHistory[]; total: number }> {
    const params = filters ? Object.fromEntries(Object.entries(filters).filter(([, v]) => v !== undefined)) : undefined;
    if (params) {
      if (params.date_from && typeof params.date_from === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(params.date_from)) {
        params.date_from = `${params.date_from}T00:00:00Z`;
      }
      if (params.date_to && typeof params.date_to === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(params.date_to)) {
        params.date_to = `${params.date_to}T23:59:59Z`;
      }
    }
    const res = await adminApiClient.get<{ history: GroupAssignmentHistory[]; total: number }>('/api/admin/groups/history', params);
    return res.data!;
  },

  async cleanupExpiredMemberships(): Promise<{ removed_count: number }> {
    const res = await adminApiClient.post<{ removed_count: number }>('/api/admin/groups/cleanup-expired', {});
    return res.data!;
  },

  async listUsers(params?: { page?: number; limit?: number; search?: string; tier?: string; status?: string; sort_by?: string; sort_order?: string }): Promise<any> {
    const res = await adminApiClient.get<any>('/api/admin/users', params);
    return res.data;
  },

  async searchUsers(query: string, limit = 10, excludeGroupId?: string): Promise<Array<{ wallet_address: string; user_id?: string; tier?: string; permissions?: string[]; groups?: string[] }>> {
    const queryLower = query.toLowerCase();
    try {
      const params = new URLSearchParams({ search: query, limit: '50' });
      if (excludeGroupId) params.append('exclude_group_id', excludeGroupId);
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
            groups: wallet.groups?.map((g: any) => g.group_name || g) || [],
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

  async getGroupAnalytics(): Promise<GroupAnalytics> {
    const response = await adminApiClient.get<any>(API_ROUTES.ADMIN.ANALYTICS_PERMISSIONS);
    const data = response.data?.data || response.data;
    return {
      total_groups: data.total_groups || 0,
      total_active_memberships: data.active_permissions || data.total_permissions || 0,
      expiring_soon_count: data.expiring_permissions?.length || 0,
      most_popular_groups: data.group_membership?.slice(0, 3).map((g: any) => ({ group_name: g.group_name, member_count: g.member_count })) || [],
      permission_distribution: data.permission_usage?.reduce((acc: any, p: any) => { acc[p.permission] = p.users_count; return acc; }, {}) || {},
    };
  },

  async getExpiringMemberships(days = 7): Promise<UserGroupMembership[]> {
    const res = await adminApiClient.get<any>('/api/permissions/assignments/expiring', { days });
    const assignments = res.data?.assignments || res.data || [];
    if (!Array.isArray(assignments)) { return []; }
    return assignments.map(mapAssignmentToMembership);
  },

  async checkUserPermission(userId: string, permission: string): Promise<boolean> {
    const res = await adminApiClient.get<{ has_permission: boolean }>(`/api/admin/users/${userId}/check-permission`, { permission });
    return res.data!.has_permission;
  },

  async getAvailablePermissions(): Promise<string[]> {
    const res = await adminApiClient.get<any>('/api/admin/permissions/available');
    const data = res.data?.data || res.data;
    return Array.isArray(data) ? data : [];
  },

  async getPermissionDefinitions(): Promise<PermissionDefinitionDto[]> {
    const res = await adminApiClient.get<PermissionDefinitionDto[]>('/api/permissions/definitions');
    const data = res.data;
    if (Array.isArray(data)) { return data; }
    if ((data as any)?.data && Array.isArray((data as any).data)) { return (data as any).data; }
    return [];
  },

  async createPermissionDefinition(req: CreatePermissionDefinitionRequest): Promise<PermissionDefinitionDto> {
    const res = await adminApiClient.post<PermissionDefinitionDto>('/api/permissions/definitions', req);
    const data = res.data;
    if ((data as any)?.data) { return (data as any).data; }
    return data!;
  },

  async deletePermissionDefinition(id: string): Promise<void> {
    await adminApiClient.delete(`/api/permissions/definitions/${id}`);
  },

  async deletePermissionByName(permission: string): Promise<void> {
    const encoded = encodeURIComponent(permission);
    await adminApiClient.delete(`/api/permissions/definitions/by-name/${encoded}`);
  },

  // Aliases
  getGroups: function () { return this.getPermissionGroups(); },
  getGroup: function (groupId: string) { return this.getPermissionGroup(groupId); },
  createGroup: function (req: CreateGroupRequest) { return this.createPermissionGroup(req); },
  updateGroup: function (groupId: string, req: UpdateGroupRequest) { return this.updatePermissionGroup(groupId, req); },
  deleteGroup: function (groupId: string) { return this.deletePermissionGroup(groupId); },
};
