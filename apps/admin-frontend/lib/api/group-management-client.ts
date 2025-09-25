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
    const response = await this.get<PermissionGroup[]>('/admin/permission-groups');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission groups');
    }
    return response.data;
  }

  async getPermissionGroup(groupId: string): Promise<PermissionGroup> {
    const response = await this.get<PermissionGroup>(`/admin/permission-groups/${groupId}`);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch permission group');
    }
    return response.data;
  }

  async createPermissionGroup(request: CreateGroupRequest): Promise<PermissionGroup> {
    const response = await this.post<PermissionGroup>('/admin/permission-groups', request);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create permission group');
    }
    return response.data;
  }

  async updatePermissionGroup(groupId: string, request: UpdateGroupRequest): Promise<PermissionGroup> {
    const response = await this.put<PermissionGroup>(`/admin/permission-groups/${groupId}`, request);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update permission group');
    }
    return response.data;
  }

  async deletePermissionGroup(groupId: string): Promise<void> {
    const response = await this.delete(`/admin/permission-groups/${groupId}`);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to delete permission group');
    }
  }

  // ============================================================================
  // USER GROUP MEMBERSHIP OPERATIONS
  // ============================================================================

  async getUserGroups(userId: string): Promise<UserGroupMembership[]> {
    const response = await this.get<UserGroupMembership[]>(`/admin/users/${userId}/groups`);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user groups');
    }
    return response.data;
  }

  async getUserPermissions(userId: string): Promise<string[]> {
    const response = await this.get<string[]>(`/api/auth/web3/groups/permissions/${userId}`);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user permissions');
    }
    return response.data;
  }

  async assignUserToGroup(request: AssignUserToGroupRequest): Promise<void> {
    const response = await this.post('/admin/groups/assign', request);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to assign user to group');
    }
  }

  async removeUserFromGroup(userId: string, groupId: string): Promise<void> {
    const response = await this.delete(`/admin/groups/${groupId}/users/${userId}`);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to remove user from group');
    }
  }

  async getGroupMemberships(groupId: string): Promise<UserGroupMembership[]> {
    const response = await this.get<UserGroupMembership[]>(`/admin/groups/${groupId}/memberships`);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch group memberships');
    }
    return response.data;
  }

  // ============================================================================
  // WEB3 AUTO-ASSIGNMENT OPERATIONS
  // ============================================================================

  async getWeb3AssignmentRules(): Promise<Web3AssignmentRule[]> {
    const response = await this.get<Web3AssignmentRule[]>('/api/auth/web3/assignment/rules');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch Web3 assignment rules');
    }
    return response.data;
  }

  async createWeb3AssignmentRule(request: CreateWeb3RuleRequest): Promise<Web3AssignmentRule> {
    const response = await this.post<Web3AssignmentRule>('/api/auth/web3/assignment/rules', request);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create Web3 assignment rule');
    }
    return response.data;
  }

  async deleteWeb3AssignmentRule(ruleId: string): Promise<void> {
    const response = await this.delete(`/api/auth/web3/assignment/rules/${ruleId}`);
    
    if (!response.success) {
      throw new Error(response.error || 'Failed to delete Web3 assignment rule');
    }
  }

  async processWalletAssignment(request: ProcessWalletRequest): Promise<string[]> {
    const response = await this.post<string[]>('/api/auth/web3/assignment/process-wallet', request);
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to process wallet assignment');
    }
    return response.data;
  }

  async verifyWalletAssets(walletAddress: string): Promise<any> {
    const response = await this.post<any>('/api/auth/web3/assignment/verify-assets', { 
      wallet_address: walletAddress 
    });
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to verify wallet assets');
    }
    return response.data;
  }

  async bulkProcessWallets(request: BulkProcessRequest): Promise<any> {
    const response = await this.post<any>('/api/auth/web3/assignment/bulk-process', request);
    
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
      if (filters.operation_type) params.operation_type = filters.operation_type;
      if (filters.operation_source) params.operation_source = filters.operation_source;
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
    const response = await this.post<{ removed_count: number }>('/admin/groups/cleanup-expired', {});
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to cleanup expired memberships');
    }
    return response.data;
  }

  // ============================================================================
  // ANALYTICS AND INSIGHTS
  // ============================================================================

  async getGroupAnalytics(): Promise<GroupAnalytics> {
    const response = await this.get<GroupAnalytics>('/admin/groups/analytics');
    
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch group analytics');
    }
    return response.data;
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

  async checkUserPermission(userId: string, permission: string): Promise<boolean> {
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
      throw new Error(response.error || 'Failed to fetch available permissions');
    }
    return response.data;
  }
}

// ============================================================================
// SINGLETON INSTANCE
// ============================================================================

export const groupManagementClient = new GroupManagementClient();