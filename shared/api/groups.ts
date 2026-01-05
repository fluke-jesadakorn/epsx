/**
 * UNIFIED GROUPS API CLIENT
 *
 * Group assignment and management endpoints for Web3 permission groups.
 * Consolidates group-related API calls across EPSX applications.
 *
 * Features:
 * - Group assignment/removal
 * - Group listing and filtering
 * - Group membership queries
 * - Bulk group operations
 */

import { UnifiedApiClient, ApiResponse, PaginatedResponse } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface Group {
  group_id: string;
  name: string;
  description?: string;
  permissions: string[];
  member_count?: number;
  created_at: string;
  updated_at?: string;
  is_active: boolean;
  metadata?: Record<string, any>;
}

export interface GroupMembership {
  group_id: string;
  name: string;
  wallet_address: string;
  assigned_at: string;
  assigned_by?: string;
  expires_at?: number;
  is_active: boolean;
  permissions: string[];
}

export interface GroupFilters {
  search?: string;
  is_active?: boolean;
  has_permission?: string;
  min_members?: number;
  max_members?: number;
  limit?: number;
  offset?: number;
}

export interface MembershipFilters {
  wallet_address?: string;
  group_id?: string;
  is_active?: boolean;
  assigned_after?: string;
  expires_before?: number;
  limit?: number;
  offset?: number;
}

export interface AssignGroupRequest {
  wallet_address: string;
  group_id: string;
  expires_at?: number;
  notes?: string;
  notify_user?: boolean;
}

export interface RemoveGroupRequest {
  wallet_address: string;
  group_id: string;
  reason?: string;
  notify_user?: boolean;
}

export interface BulkAssignRequest {
  wallet_addresses: string[];
  group_ids: string[];
  expires_at?: number;
  notes?: string;
}

export interface BulkRemoveRequest {
  wallet_addresses: string[];
  group_ids: string[];
  reason?: string;
}

export interface CreateGroupRequest {
  name: string;
  description?: string;
  permissions: string[];
  metadata?: Record<string, any>;
}

export interface UpdateGroupRequest {
  name?: string;
  description?: string;
  permissions?: string[];
  is_active?: boolean;
  metadata?: Record<string, any>;
}

export interface GroupStats {
  total_groups: number;
  active_groups: number;
  total_memberships: number;
  active_memberships: number;
  by_group: Record<string, number>;
  recent_assignments: number;
  recent_removals: number;
}

// ============================================================================
// GROUPS API CLASS
// ============================================================================

export class GroupsApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  // ============================================================================
  // GROUP LISTING
  // ============================================================================

  /**
   * List all groups
   * GET /api/admin/groups
   */
  async listGroups(filters?: GroupFilters): Promise<ApiResponse<PaginatedResponse<Group>>> {
    return this.client.get<PaginatedResponse<Group>>('/api/admin/groups', filters);
  }

  /**
   * Get group by ID
   * GET /api/admin/groups/{group_id}
   */
  async getGroup(group_id: string): Promise<ApiResponse<Group>> {
    return this.client.get<Group>(`/api/admin/groups/${group_id}`);
  }

  /**
   * Get group members
   * GET /api/admin/groups/{group_id}/members
   */
  async getGroupMembers(group_id: string, filters?: { limit?: number; offset?: number }): Promise<ApiResponse<PaginatedResponse<GroupMembership>>> {
    return this.client.get<PaginatedResponse<GroupMembership>>(`/api/admin/groups/${group_id}/members`, filters);
  }

  // ============================================================================
  // GROUP MANAGEMENT (Admin only)
  // ============================================================================

  /**
   * Create new group
   * POST /api/admin/groups
   */
  async createGroup(data: CreateGroupRequest): Promise<ApiResponse<Group>> {
    return this.client.post<Group>('/api/admin/groups', data);
  }

  /**
   * Update group
   * PUT /api/admin/groups/{group_id}
   */
  async updateGroup(group_id: string, data: UpdateGroupRequest): Promise<ApiResponse<Group>> {
    return this.client.put<Group>(`/api/admin/groups/${group_id}`, data);
  }

  /**
   * Delete group
   * DELETE /api/admin/groups/{group_id}
   */
  async deleteGroup(group_id: string): Promise<ApiResponse<{ deleted: boolean }>> {
    return this.client.delete<{ deleted: boolean }>(`/api/admin/groups/${group_id}`);
  }

  // ============================================================================
  // MEMBERSHIP MANAGEMENT
  // ============================================================================

  /**
   * Assign wallet to group
   * POST /api/admin/groups/assign
   */
  async assignToGroup(data: AssignGroupRequest): Promise<ApiResponse<{ assigned: boolean; membership: GroupMembership }>> {
    return this.client.post<{ assigned: boolean; membership: GroupMembership }>('/api/admin/groups/assign', data);
  }

  /**
   * Remove wallet from group
   * POST /api/admin/groups/remove
   */
  async removeFromGroup(data: RemoveGroupRequest): Promise<ApiResponse<{ removed: boolean }>> {
    return this.client.post<{ removed: boolean }>('/api/admin/groups/remove', data);
  }

  /**
   * Assign wallets to groups in bulk
   * POST /api/admin/groups/bulk/assign
   */
  async bulkAssignToGroups(data: BulkAssignRequest): Promise<ApiResponse<{ assigned_count: number; failed: string[] }>> {
    return this.client.post<{ assigned_count: number; failed: string[] }>('/api/admin/groups/bulk/assign', data);
  }

  /**
   * Remove wallets from groups in bulk
   * POST /api/admin/groups/bulk/remove
   */
  async bulkRemoveFromGroups(data: BulkRemoveRequest): Promise<ApiResponse<{ removed_count: number; failed: string[] }>> {
    return this.client.post<{ removed_count: number; failed: string[] }>('/api/admin/groups/bulk/remove', data);
  }

  // ============================================================================
  // MEMBERSHIP QUERIES
  // ============================================================================

  /**
   * Get wallet's group memberships
   * GET /api/admin/memberships?wallet_address={address}
   */
  async getWalletMemberships(wallet_address: string): Promise<ApiResponse<GroupMembership[]>> {
    return this.client.get<GroupMembership[]>('/api/admin/memberships', { wallet_address });
  }

  /**
   * List all memberships with filters
   * GET /api/admin/memberships
   */
  async listMemberships(filters?: MembershipFilters): Promise<ApiResponse<PaginatedResponse<GroupMembership>>> {
    return this.client.get<PaginatedResponse<GroupMembership>>('/api/admin/memberships', filters);
  }

  /**
   * Check if wallet is in group
   * POST /api/groups/check-membership
   */
  async checkMembership(wallet_address: string, group_id: string): Promise<ApiResponse<{ is_member: boolean; membership?: GroupMembership }>> {
    return this.client.post<{ is_member: boolean; membership?: GroupMembership }>('/api/groups/check-membership', {
      wallet_address,
      group_id
    });
  }

  // ============================================================================
  // STATISTICS
  // ============================================================================

  /**
   * Get group statistics
   * GET /api/admin/groups/stats
   */
  async getStats(): Promise<ApiResponse<GroupStats>> {
    return this.client.get<GroupStats>('/api/admin/groups/stats');
  }

  /**
   * Get group activity history
   * GET /api/admin/groups/{group_id}/history
   */
  async getGroupHistory(group_id: string, filters?: { limit?: number }): Promise<ApiResponse<Array<{
    action: 'assigned' | 'removed';
    wallet_address: string;
    timestamp: string;
    performed_by?: string;
  }>>> {
    return this.client.get(`/api/admin/groups/${group_id}/history`, filters);
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Groups API client
 */
export function createGroupsClient(client: UnifiedApiClient): GroupsApi {
  return new GroupsApi(client);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default GroupsApi;
