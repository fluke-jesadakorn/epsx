/**
 * Group Management API Client
 * Interfaces with the new Web3 group-based permission system
 * Extends UnifiedAdminClient for consistency with existing admin clients
 */

import { UnifiedAdminClient } from './unified-admin-client';

// ============================================================================
// GROUP MANAGEMENT TYPES
// ============================================================================

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
  member_count?: number; // Added member count from backend
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

export interface AssignUserToGroupRequest {
  user_id: string;
  group_id: string;
  expires_at?: string | null;
  reason?: string;
}

export interface CreateWeb3RuleRequest {
  group_id: string;
  blockchain_network: string;
  verification_type: string;
  contract_address?: string;
  token_id?: string;
  minimum_balance?: string;
}

export interface ProcessWalletRequest {
  wallet_address: string;
}

export interface BulkProcessRequest {
  wallet_addresses: string[];
}

export interface GroupAnalytics {
  total_groups: number;
  total_active_memberships: number;
  expiring_soon_count: number;
  most_popular_groups: Array<{ group_name: string; member_count: number }>;
  permission_distribution: Record<string, number>;
}

// ============================================================================
// GROUP MANAGEMENT API CLIENT
// ============================================================================

export class GroupManagementClient extends UnifiedAdminClient {
  constructor(baseURL?: string, token?: string, serverSide = false) {
    super(baseURL, token, serverSide);
  }

  // ============================================================================
  // PERMISSION GROUP OPERATIONS
  // ============================================================================

  async getPermissionGroups(): Promise<PermissionGroup[]> {
    console.log('🔍 ULTRA DEBUG: Starting getPermissionGroups');
    console.log(
      '🔍 ULTRA DEBUG: About to call /api/v1/admin/permission-groups'
    );

    // FIX: Use consistent API pattern to avoid Next.js route interception
    // Changed from '/admin/permission-groups' to '/api/v1/admin/permission-groups'
    // to match other admin API calls and avoid conflicts with local Next.js routes
    const response = await this.get<{
      permission_groups: PermissionGroup[];
      pagination: any;
      total: number;
    }>('/api/v1/admin/permission-groups');

    console.log(
      '🔍 ULTRA DEBUG: Raw response:',
      JSON.stringify(response, null, 2)
    );
    console.log('🔍 ULTRA DEBUG: Response type:', typeof response);
    console.log('🔍 ULTRA DEBUG: Response success:', response?.success);
    console.log('🔍 ULTRA DEBUG: Response data type:', typeof response?.data);
    console.log(
      '🔍 ULTRA DEBUG: Response data keys:',
      response?.data ? Object.keys(response.data) : 'no data'
    );
    
    // DEEP DEBUG: Check response.data directly
    console.log('🔍 DEEP DEBUG: response.data direct access:', response.data);
    console.log('🔍 DEEP DEBUG: response.data.permission_groups direct:', response.data?.permission_groups);
    console.log('🔍 DEEP DEBUG: response["data"] bracket access:', response["data"]);
    console.log('🔍 DEEP DEBUG: response["data"]["permission_groups"] bracket:', response["data"]?.["permission_groups"]);

    if (!response.success || !response.data) {
      console.log(
        '❌ ULTRA DEBUG: Response failed - success:',
        response?.success,
        'data:',
        !!response?.data
      );
      throw new Error(response.error || 'Failed to fetch permission groups');
    }

    console.log(
      '🔍 ULTRA DEBUG: permission_groups exists:',
      !!response.data.permission_groups
    );
    console.log(
      '🔍 ULTRA DEBUG: permission_groups type:',
      typeof response.data.permission_groups
    );
    console.log(
      '🔍 ULTRA DEBUG: permission_groups isArray:',
      Array.isArray(response.data.permission_groups)
    );

    console.log('Debug Data: ' + JSON.stringify(response.data));
    
    // ULTRA ROOT CAUSE DEBUG: Check if data is nested differently
    console.log('🔍 STRUCTURE DEBUG: response keys:', Object.keys(response));
    console.log('🔍 STRUCTURE DEBUG: response.data keys:', response.data ? Object.keys(response.data) : 'no data keys');
    console.log('🔍 STRUCTURE DEBUG: response.data.data exists?:', !!response.data?.data);
    console.log('🔍 STRUCTURE DEBUG: response.data.data keys:', response.data?.data ? Object.keys(response.data.data) : 'no nested data');
    
    // Check if permission_groups is nested under response.data.data
    const actualData = response.data?.data || response.data;
    console.log('🔍 ACTUAL DATA:', actualData);
    console.log('🔍 ACTUAL DATA permission_groups:', actualData?.permission_groups);

    // ROOT CAUSE FIX: Use the correct data path
    if (!actualData?.permission_groups || !Array.isArray(actualData.permission_groups)) {
      console.log(
        '❌ ULTRA DEBUG: Invalid structure - permission_groups:',
        response.data.permission_groups,
        'exists:', !!response.data.permission_groups,
        'isArray:', Array.isArray(response.data.permission_groups)
      );
      throw new Error(
        'Invalid response structure: permission_groups is not an array'
      );
    }

    console.log(
      '✅ ULTRA DEBUG: Success! Returning',
      actualData.permission_groups.length,
      'permission groups'
    );
    return actualData.permission_groups;
  }

  async getPermissionGroup(groupId: string): Promise<PermissionGroup> {
    const response = await this.get<PermissionGroup>(
      `/api/v1/admin/permission-groups/${groupId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission group');
    }
    return response.data;
  }

  async createPermissionGroup(
    request: CreateGroupRequest
  ): Promise<PermissionGroup> {
    const response = await this.post<PermissionGroup>(
      '/api/v1/admin/permission-groups',
      request
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create permission group');
    }
    return response.data;
  }

  async updatePermissionGroup(
    groupId: string,
    request: UpdateGroupRequest
  ): Promise<PermissionGroup> {
    const response = await this.put<PermissionGroup>(
      `/api/v1/admin/permission-groups/${groupId}`,
      request
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update permission group');
    }
    return response.data;
  }

  async deletePermissionGroup(groupId: string): Promise<void> {
    const response = await this.delete(
      `/api/v1/admin/permission-groups/${groupId}`
    );

    if (!response.success) {
      throw new Error(response.error || 'Failed to delete permission group');
    }
  }

  // ============================================================================
  // USER GROUP MEMBERSHIP OPERATIONS
  // ============================================================================

  async getUserGroups(userId: string): Promise<UserGroupMembership[]> {
    const response = await this.get<UserGroupMembership[]>(
      `/admin/wallets/${userId}/assignments`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user groups');
    }
    return response.data;
  }

  async getUserPermissions(userId: string): Promise<string[]> {
    const response = await this.get<string[]>(
      `/api/auth/web3/groups/permissions/${userId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user permissions');
    }
    return response.data;
  }

  async assignUserToGroup(request: AssignUserToGroupRequest): Promise<void> {
    const response = await this.post('/admin/wallet-assignments', {
      wallet_address: request.user_id,
      group_id: request.group_id,
      expires_at: request.expires_at,
      assignment_reason: request.reason,
      assignment_source: 'manual',
    });

    if (!response.success) {
      throw new Error(response.error || 'Failed to assign user to group');
    }
  }

  async removeUserFromGroup(userId: string, groupId: string): Promise<void> {
    // This endpoint might not exist yet in backend
    // TODO: Implement when backend supports removing assignments
    const response = await this.delete(
      `/admin/wallet-assignments/${userId}/${groupId}`
    );

    if (!response.success) {
      throw new Error(response.error || 'Failed to remove user from group');
    }
  }

  async getGroupMemberships(groupId: string): Promise<UserGroupMembership[]> {
    // This endpoint might not exist yet in backend
    // TODO: Implement when backend supports querying by group
    const response = await this.get<UserGroupMembership[]>(
      `/admin/group-memberships/${groupId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch group memberships');
    }
    return response.data;
  }

  // ============================================================================
  // WEB3 AUTO-ASSIGNMENT OPERATIONS
  // ============================================================================

  async getWeb3AssignmentRules(): Promise<Web3AssignmentRule[]> {
    const response = await this.get<Web3AssignmentRule[]>(
      '/api/auth/web3/assignment/rules'
    );

    if (!response.success || !response.data) {
      throw new Error(
        response.error || 'Failed to fetch Web3 assignment rules'
      );
    }
    return response.data;
  }

  async createWeb3AssignmentRule(
    request: CreateWeb3RuleRequest
  ): Promise<Web3AssignmentRule> {
    const response = await this.post<Web3AssignmentRule>(
      '/api/auth/web3/assignment/rules',
      request
    );

    if (!response.success || !response.data) {
      throw new Error(
        response.error || 'Failed to create Web3 assignment rule'
      );
    }
    return response.data;
  }

  async deleteWeb3AssignmentRule(ruleId: string): Promise<void> {
    const response = await this.delete(
      `/api/auth/web3/assignment/rules/${ruleId}`
    );

    if (!response.success) {
      throw new Error(
        response.error || 'Failed to delete Web3 assignment rule'
      );
    }
  }

  async processWalletAssignment(
    request: ProcessWalletRequest
  ): Promise<string[]> {
    const response = await this.post<string[]>(
      '/api/auth/web3/assignment/process-wallet',
      request
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to process wallet assignment');
    }
    return response.data;
  }

  async verifyWalletAssets(walletAddress: string): Promise<any> {
    const response = await this.post<any>(
      '/api/auth/web3/assignment/verify-assets',
      {
        wallet_address: walletAddress,
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to verify wallet assets');
    }
    return response.data;
  }

  async bulkProcessWallets(request: BulkProcessRequest): Promise<any> {
    const response = await this.post<any>(
      '/api/auth/web3/assignment/bulk-process',
      request
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to bulk process wallets');
    }
    return response.data;
  }

  // ============================================================================
  // AUDIT AND HISTORY OPERATIONS
  // ============================================================================

  async getGroupAssignmentHistory(filters?: {
    operation_type?: string;
    operation_source?: string;
    group_id?: string;
    user_search?: string;
    date_from?: string;
    date_to?: string;
    limit?: number;
    offset?: number;
  }): Promise<{
    success: boolean;
    data?: {
      history: GroupAssignmentHistory[];
      total: number;
    };
    error?: string;
  }> {
    const params: Record<string, string> = {};
    if (filters) {
      if (filters.operation_type)
        params.operation_type = filters.operation_type;
      if (filters.operation_source)
        params.operation_source = filters.operation_source;
      if (filters.group_id) params.group_id = filters.group_id;
      if (filters.user_search) params.user_search = filters.user_search;
      if (filters.date_from) params.date_from = filters.date_from;
      if (filters.date_to) params.date_to = filters.date_to;
      if (filters.limit) params.limit = filters.limit.toString();
      if (filters.offset) params.offset = filters.offset.toString();
    }

    const response = await this.get<{
      history: GroupAssignmentHistory[];
      total: number;
    }>('/admin/groups/history', params);

    return response;
  }

  async cleanupExpiredMemberships(): Promise<{ removed_count: number }> {
    const response = await this.post<{ removed_count: number }>(
      '/admin/groups/cleanup-expired',
      {}
    );

    if (!response.success || !response.data) {
      throw new Error(
        response.error || 'Failed to cleanup expired memberships'
      );
    }
    return response.data;
  }

  // ============================================================================
  // CONSOLIDATED USER MANAGEMENT
  // Backend-centric user operations with comprehensive data
  // ============================================================================

  async listUsers(params?: {
    page?: number;
    limit?: number;
    search?: string;
    tier?: string;
    status?: string;
    sort_by?: string;
    sort_order?: string;
  }): Promise<any> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.set('page', params.page.toString());
    if (params?.limit) queryParams.set('limit', params.limit.toString());
    if (params?.search) queryParams.set('search', params.search);
    if (params?.tier) queryParams.set('tier', params.tier);
    if (params?.status) queryParams.set('status', params.status);
    if (params?.sort_by) queryParams.set('sort_by', params.sort_by);
    if (params?.sort_order) queryParams.set('sort_order', params.sort_order);

    const url = `/admin/users${queryParams.toString() ? '?' + queryParams.toString() : ''}`;
    const response = await this.get<any>(url);

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch users');
    }

    return response.data;
  }

  async getUser(walletAddress: string): Promise<any> {
    const response = await this.get<any>(`/admin/users/${walletAddress}`);

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user details');
    }

    return response.data;
  }

  async updateUser(
    walletAddress: string,
    updates: {
      tier_level?: string;
      is_active?: boolean;
      metadata?: any;
    }
  ): Promise<any> {
    const response = await this.put<any>(
      `/admin/users/${walletAddress}`,
      updates
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update user');
    }

    return response.data;
  }

  async getUserStats(): Promise<any> {
    const response = await this.get<any>('/admin/users/stats');

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user statistics');
    }

    return response.data;
  }

  // ============================================================================
  // ANALYTICS AND BUSINESS INTELLIGENCE
  // Comprehensive data aggregation for administrative insights
  // ============================================================================

  async getPlatformOverview(period?: string): Promise<any> {
    const queryParams = period ? `?period=${period}` : '';
    const response = await this.get<any>(
      `/admin/analytics/overview${queryParams}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch platform overview');
    }

    return response.data;
  }

  async getUserAnalytics(period?: string): Promise<any> {
    const queryParams = period ? `?period=${period}` : '';
    const response = await this.get<any>(
      `/admin/analytics/users${queryParams}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user analytics');
    }

    return response.data;
  }

  async getPermissionAnalytics(): Promise<any> {
    const response = await this.get<any>('/admin/analytics/permissions');

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission analytics');
    }

    return response.data;
  }

  async getRevenueAnalytics(period?: string): Promise<any> {
    const queryParams = period ? `?period=${period}` : '';
    const response = await this.get<any>(
      `/admin/analytics/revenue${queryParams}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch revenue analytics');
    }

    return response.data;
  }

  // ============================================================================
  // LEGACY ANALYTICS (DEPRECATED - Use getPermissionAnalytics instead)
  // ============================================================================

  async getGroupAnalytics(): Promise<GroupAnalytics> {
    // Use the new consolidated permission analytics endpoint (consistent API pattern)
    const response = await this.get<any>('/api/v1/admin/analytics/permissions');

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch group analytics');
    }

    // Map the new backend response to the expected frontend format
    const data = response.data;
    return {
      total_groups: data.permission_usage?.length || 0,
      total_active_memberships: data.total_permissions || 0,
      expiring_soon_count: data.expiring_permissions?.length || 0,
      most_popular_groups:
        data.group_membership?.slice(0, 3).map((group: any) => ({
          group_name: group.group_name,
          member_count: group.member_count,
        })) || [],
      permission_distribution:
        data.permission_usage?.reduce((acc: any, perm: any) => {
          acc[perm.permission] = perm.users_count;
          return acc;
        }, {}) || {},
    };
  }

  async getExpiringMemberships(days = 7): Promise<UserGroupMembership[]> {
    const response = await this.get<UserGroupMembership[]>(
      '/admin/groups/expiring-memberships',
      { days: days.toString() }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch expiring memberships');
    }
    return response.data;
  }

  // ============================================================================
  // PERMISSION UTILITIES
  // ============================================================================

  async checkUserPermission(
    userId: string,
    permission: string
  ): Promise<boolean> {
    const response = await this.get<{ has_permission: boolean }>(
      `/admin/users/${userId}/check-permission`,
      { permission }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to check user permission');
    }
    return response.data.has_permission;
  }

  async getAvailablePermissions(): Promise<string[]> {
    const response = await this.get<string[]>('/admin/permissions/available');

    if (!response.success || !response.data) {
      throw new Error(
        response.error || 'Failed to fetch available permissions'
      );
    }
    return response.data;
  }
}

// ============================================================================
// SINGLETON INSTANCE
// ============================================================================

export const groupManagementClient = new GroupManagementClient();
