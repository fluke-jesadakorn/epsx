/**
 * Consolidated Wallet Actions
 * Combines: wallet-list-actions.ts, wallet-list-focused-actions.ts, wallet-profile-actions.ts,
 * wallet-permissions-actions.ts, app/actions/wallet-actions.ts, and wallets.ts
 *
 * This file contains ALL wallet management operations including:
 * - Wallet CRUD operations
 * - Permission management (roles, profiles, custom permissions)
 * - Bulk operations
 * - Temporary permissions
 * - Activity logging and history
 * - Permission analysis and validation
 * - Wallet search functionality
 */

'use server';

import { revalidatePath } from 'next/cache';

import { makeAuthenticatedRequest } from './shared-utils';

import { createErrorResult, createSuccessResult, type ActionResult } from '@/lib/action-utils';
import type { UnifiedWalletData } from '@/lib/types/unified-wallet';

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

export interface WalletListFilters {
  search: string;
  status: string;
  group: string;
  page: number;
  limit: number;
  sortBy: string;
  sortOrder: 'asc' | 'desc';
}

export interface CreateWalletRequest {
  email: string;
  name?: string;
  group?: string;
  permissions?: string[];
  sendInvite?: boolean;
}

export interface UpdateWalletRequest {
  id: string;
  email?: string;
  name?: string;
  group?: string;
  isActive?: boolean;
  permissions?: string[];
}

export interface WalletPermissionChange {
  walletAddress: string;
  permissions: string[];
  action: 'grant' | 'revoke' | 'replace';
  expiresAt?: string;
  reason?: string;
}

export interface WalletProfileUpdateData {
  displayName?: string;
  firstName?: string;
  lastName?: string;
  phoneNumber?: string;
  timezone?: string;
  language?: string;
}

export interface WalletStatusUpdateData {
  status: 'active' | 'inactive' | 'suspended';
  reason?: string;
}

export interface WalletGroupUpdateData {
  groups: string[];
  reason?: string;
}

export interface ModuleAccessUpdateData {
  modules: string[];
  quotas?: Record<string, number>;
  reason?: string;
}

export interface PermissionHistoryEntry {
  id: string;
  walletAddress: string;
  action: 'granted' | 'revoked' | 'modified';
  type: 'group' | 'permission' | 'profile';
  resource?: string;
  permission?: string;
  group?: string;
  profileId?: string;
  grantedBy: string;
  grantedAt: Date;
  reason?: string;
  expires?: Date;
}

export interface ActivityLogEntry {
  id: string;
  action: string;
  resource_type: string;
  resource_id: string;
  result: 'success' | 'failure' | 'partial_success' | 'denied' | 'error';
  timestamp: Date;
  client_ip?: string;
  user_agent?: string;
  session_id?: string;
  metadata: {
    previous_values?: Record<string, string>;
    new_values?: Record<string, string>;
    error_message?: string;
    duration_ms?: number;
    additional_data?: Record<string, string>;
  };
}

export interface ActivityLogParams {
  limit?: number;
  offset?: number;
  start_date?: string;
  end_date?: string;
  action_type?: string;
}

// ============================================================================
// USER LIST OPERATIONS
// ============================================================================

/**
 * Get paginated user list with filters
 * @param filters
 */
export async function getWalletList(filters: WalletListFilters): Promise<ActionResult<{
  users: UnifiedWalletData[];
  totalCount: number;
  currentPage: number;
  totalPages: number;
}>> {
  try {
    const params = new URLSearchParams({
      search: filters.search,
      status: filters.status,
      group: filters.group,
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sortBy: filters.sortBy,
      sortOrder: filters.sortOrder
    });

    const response = await makeAuthenticatedRequest(`/api/admin/wallets?${params.toString()}`);

    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch user list:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch users');
  }
}

/**
 * Search users with enhanced filtering
 * @param query
 * @param filters
 */
export async function searchWallets(query: string, filters?: Partial<WalletListFilters>): Promise<ActionResult<UnifiedWalletData[]>> {
  try {
    const params = new URLSearchParams({
      q: query,
      ...filters && Object.fromEntries(
        Object.entries(filters).map(([key, value]) => [key, String(value)])
      )
    });

    const response = await makeAuthenticatedRequest(`/api/admin/wallets/search?${params.toString()}`);

    return createSuccessResult(response.users || []);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to search users:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to search users');
  }
}

/**
 * Get user statistics for dashboard
 */
export async function getWalletStats(): Promise<ActionResult<{
  totalUsers: number;
  activeUsers: number;
  newUsersThisMonth: number;
  totalAdmins: number;
}>> {
  try {
    const response = await makeAuthenticatedRequest('/api/admin/wallets/stats');
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch user stats:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch user statistics');
  }
}

// ============================================================================
// USER PROFILE OPERATIONS
// ============================================================================

/**
 * Get user profile by ID
 * @param walletAddress
 */
export async function getWalletProfile(walletAddress: string): Promise<ActionResult<UnifiedWalletData>> {
  try {
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch user profile:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch user profile');
  }
}

/**
 * Create new user
 * @param userData
 */
export async function createWallet(userData: CreateWalletRequest): Promise<ActionResult<UnifiedWalletData>> {
  try {
    const response = await makeAuthenticatedRequest('/api/admin/wallets', {
      method: 'POST',
      body: JSON.stringify(userData)
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'User created successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to create user:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to create user');
  }
}

/**
 * Update user profile
 * @param userData
 */
export async function updateWallet(userData: UpdateWalletRequest): Promise<ActionResult<UnifiedWalletData>> {
  try {
    const { id, ...updateData } = userData;

    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updateData)
    });

    revalidatePath('/users');
    revalidatePath(`/users/${id}`);
    return createSuccessResult(response, 'User updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update user');
  }
}

/**
 * Delete user
 * @param walletAddress
 */
export async function deleteWallet(walletAddress: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}`, {
      method: 'DELETE'
    });

    revalidatePath('/users');
    return createSuccessResult(undefined, 'User deleted successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to delete user:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to delete user');
  }
}

/**
 * Toggle user active status
 * @param walletAddress
 */
export async function toggleWalletStatus(walletAddress: string): Promise<ActionResult<UnifiedWalletData>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/toggle-status - DEAD CODE
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/toggle-status`, {
      method: 'PATCH'
    });

    revalidatePath('/users');
    revalidatePath(`/users/${walletAddress}`);
    return createSuccessResult(response, 'User status updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to toggle user status:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update user status');
  }
}

// ============================================================================
// USER PERMISSION OPERATIONS
// ============================================================================

/**
 * Get user permissions
 * @param walletAddress
 */
export async function getWalletPermissions(walletAddress: string): Promise<ActionResult<{
  permissions: string[];
  groups: string[];
  profiles: string[];
}>> {
  try {
    const response = await makeAuthenticatedRequest(`/api/admin/permissions/wallets/${walletAddress}/permissions`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch user permissions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch user permissions');
  }
}

/**
 * Update user permissions
 * @param change
 */
export async function updateWalletPermissions(change: WalletPermissionChange): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/api/admin/permissions/wallets/${change.walletAddress}/permissions`, {
      method: 'PUT',
      body: JSON.stringify({
        permissions: change.permissions,
        action: change.action,
        expiresAt: change.expiresAt,
        reason: change.reason
      })
    });

    revalidatePath('/users');
    revalidatePath(`/users/${change.walletAddress}`);
    return createSuccessResult(undefined, 'User permissions updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user permissions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update user permissions');
  }
}

/**
 * Bulk permission updates
 * @param changes
 */
export async function bulkUpdateWalletPermissions(changes: WalletPermissionChange[]): Promise<ActionResult<{
  successful: number;
  failed: number;
  errors: string[];
}>> {
  try {
    // NOTE: Should use /permissions/bulk/grant or /permissions/bulk/revoke instead
    const response = await makeAuthenticatedRequest('/api/admin/wallets/permissions/bulk', {
      method: 'PUT',
      body: JSON.stringify({ changes })
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'Bulk permission update completed');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to perform bulk permission update:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update permissions');
  }
}

// ============================================================================
// BULK USER OPERATIONS
// ============================================================================

/**
 * Bulk delete users
 * @param walletAddresss
 */
export async function bulkDeleteWallets(walletAddresses: string[]): Promise<ActionResult<{
  successful: number;
  failed: number;
  errors: string[];
}>> {
  try {
    // NOTE: Backend does not implement /wallets/bulk-delete - DEAD CODE
    const response = await makeAuthenticatedRequest('/api/admin/wallets/bulk-delete', {
      method: 'DELETE',
      body: JSON.stringify({ walletAddresses })
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'Bulk delete completed');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to perform bulk delete:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to delete users');
  }
}

/**
 * Export users to CSV
 * @param filters
 */
export async function exportWallets(filters?: Partial<WalletListFilters>): Promise<ActionResult<{
  downloadUrl: string;
  filename: string;
}>> {
  try {
    const params = filters ? new URLSearchParams(
      Object.fromEntries(
        Object.entries(filters).map(([key, value]) => [key, String(value)])
      )
    ).toString() : '';

    // NOTE: Backend does not implement /wallets/export - DEAD CODE
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/export?${params}`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to export users:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to export users');
  }
}

// ============================================================================
// DETAILED USER PROFILE OPERATIONS
// ============================================================================

/**
 * Get unified detailed user data by ID
 * @param walletAddress
 */
export async function getUnifiedWalletData(walletAddress: string): Promise<ActionResult<UnifiedWalletData>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/unified - DEAD CODE
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/unified`);

    // Transform backend response to match frontend interface
    const userData = {
      id: response.user.id,
      email: response.user.email,
      displayName: response.user.display_name || response.user.email.split('@')[0],
      firstName: null,
      lastName: null,
      avatar: response.user.profile_picture,
      emailVerified: true,
      createdAt: new Date(response.user.created_at),
      updatedAt: new Date(response.user.updated_at),
      lastLogin: response.user.last_login ? new Date(response.user.last_login) : undefined,

      status: response.user.is_active ? 'active' : 'inactive' as const,
      phoneNumber: null,
      timezone: null,
      language: 'en',
      twoFactorEnabled: false,

      groups: [],
      customPermissions: response.permissions?.individual_permissions || [],
      permissionProfiles: response.permissions?.permission_profiles?.map((profile: any) => ({
        id: profile.id,
        name: profile.name,
        description: profile.description,
        permissions: profile.permissions,
        assignedAt: new Date(profile.assigned_at),
        expiresAt: profile.expires_at ? new Date(profile.expires_at) : null
      })) || [],

      moduleAccess: response.modules?.enabled_modules || [],
      moduleQuotas: response.modules?.quotas ? [
        {
          moduleId: 'api',
          quotaType: 'api_calls',
          limit: response.modules.quotas.api_calls_per_day || 1000,
          used: response.modules.quotas.api_calls_used || 0,
          period: 'daily'
        }
      ] : [],
      stockRankingPackages: [],

      apiKeys: [],
      recentActivity: [],
      loginHistory: [],
      usageMetrics: {
        apiCallsThisMonth: response.modules?.quotas?.api_calls_used || 0,
        storageUsed: 0,
        lastActiveDate: response.activity?.last_activity ? new Date(response.activity.last_activity) : new Date(),
        sessionsThisMonth: response.activity?.total_logins || 0,
        averageSessionDuration: 0
      },

      billing: {
        tier: response.billing?.tier || response.user?.subscription_tier || 'free',
        status: response.billing?.status || 'active',
        monthlySpend: response.billing?.monthly_spend || 0,
        nextBillingDate: response.billing?.next_billing_date ? new Date(response.billing.next_billing_date) : null
      }
    };

    return createSuccessResult(userData as any);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get unified user data:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch user data');
  }
}

/**
 * Update user profile information
 * @param walletAddress
 * @param data
 */
export async function updateWalletProfile(walletAddress: string, data: WalletProfileUpdateData): Promise<ActionResult<void>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/profile - DEAD CODE
    await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/profile`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Profile updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user profile:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update profile');
  }
}

/**
 * Update user status
 * @param walletAddress
 * @param data
 */
export async function updateWalletStatus(walletAddress: string, data: WalletStatusUpdateData): Promise<ActionResult<void>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/status - DEAD CODE
    await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/status`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Status updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user status:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update status');
  }
}

/**
 * Update user groups
 * @param walletAddress
 * @param data
 */
export async function updateWalletGroups(walletAddress: string, data: WalletGroupUpdateData): Promise<ActionResult<void>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/groups - DEAD CODE
    await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/groups`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Groups updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user groups:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update groups');
  }
}

/**
 * Update module access
 * @param walletAddress
 * @param data
 */
export async function updateModuleAccess(walletAddress: string, data: ModuleAccessUpdateData): Promise<ActionResult<void>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/modules - DEAD CODE
    await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/modules`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Module access updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update module access:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update module access');
  }
}

// ============================================================================
// GROUP MANAGEMENT OPERATIONS
// ============================================================================

/**
 * Assign group to user
 * @param data
 * @param data.walletAddress
 * @param data.group
 * @param data.reason
 */
export async function assignWalletGroup(data: { walletAddress: string; group: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/api/admin/casbin/groups', {
      method: 'POST',
      body: JSON.stringify({
        user: data.walletAddress,
        group: data.group
      })
    });

    revalidatePath(`/users/${data.walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Group assigned successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to assign user group:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to assign group');
  }
}

/**
 * Remove group from user
 * @param data
 * @param data.walletAddress
 * @param data.group
 * @param data.reason
 */
export async function removeWalletGroup(data: { walletAddress: string; group: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/api/admin/casbin/groups', {
      method: 'DELETE',
      body: JSON.stringify({
        user: data.walletAddress,
        group: data.group
      })
    });

    revalidatePath(`/users/${data.walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Group removed successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to remove user group:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to remove group');
  }
}

// ============================================================================
// PERMISSION PROFILE OPERATIONS
// ============================================================================

/**
 * Assign permission profile to user
 * @param data
 * @param data.walletAddress
 * @param data.profileId
 * @param data.reason
 */
export async function assignPermissionProfile(data: { walletAddress: string; profileId: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/api/admin/permission-profiles/assign', {
      method: 'POST',
      body: JSON.stringify({
        profile_id: data.profileId,
        user_ids: [data.walletAddress]
      })
    });

    revalidatePath(`/users/${data.walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Permission profile assigned successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to assign permission profile:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to assign permission profile');
  }
}

// ============================================================================
// CUSTOM PERMISSION OPERATIONS
// ============================================================================

/**
 * Add custom permission to user
 * @param data
 * @param data.walletAddress
 * @param data.resource
 * @param data.action
 * @param data.reason
 */
export async function addCustomPermission(data: { walletAddress: string; resource: string; action: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/api/admin/casbin/policies', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.walletAddress,
        object: data.resource,
        action: data.action
      })
    });

    revalidatePath(`/users/${data.walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Permission added successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to add custom permission:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to add permission');
  }
}

/**
 * Remove custom permission from user
 * @param data
 * @param data.walletAddress
 * @param data.resource
 * @param data.action
 * @param data.reason
 */
export async function removeCustomPermission(data: { walletAddress: string; resource: string; action: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/api/admin/casbin/policies', {
      method: 'DELETE',
      body: JSON.stringify({
        subject: data.walletAddress,
        object: data.resource,
        action: data.action
      })
    });

    revalidatePath(`/users/${data.walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Permission removed successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to remove custom permission:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to remove permission');
  }
}

// ============================================================================
// BULK PERMISSION OPERATIONS
// ============================================================================

/**
 * Bulk assign permissions to multiple users
 * @param data
 * @param data.walletAddresss
 * @param data.permissions
 * @param data.reason
 */
export async function bulkAssignPermissions(data: {
  walletAddresses: string[];
  permissions: { resource: string; action: string }[];
  reason?: string;
}): Promise<ActionResult<{ succeeded: string[]; failed: { walletAddress: string; error: string }[] }>> {
  try {
    const response = await makeAuthenticatedRequest('/api/admin/casbin/bulk-assign', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: data.walletAddresses,
        policies: data.permissions.map(p => ({
          object: p.resource,
          action: p.action
        })),
        reason: data.reason
      })
    });

    data.walletAddresses.forEach((addr: string) => {
      revalidatePath(`/users/${addr}`);
    });
    revalidatePath('/users');

    return createSuccessResult(response, 'Bulk permission assignment completed');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to bulk assign permissions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to assign permissions');
  }
}

/**
 * Bulk remove permissions from multiple users
 * @param data
 * @param data.walletAddresss
 * @param data.permissions
 * @param data.reason
 */
export async function bulkRemovePermissions(data: {
  walletAddresses: string[];
  permissions: { resource: string; action: string }[];
  reason?: string;
}): Promise<ActionResult<{ succeeded: string[]; failed: { walletAddress: string; error: string }[] }>> {
  try {
    const response = await makeAuthenticatedRequest('/api/admin/casbin/bulk-remove', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: data.walletAddresses,
        policies: data.permissions.map(p => ({
          object: p.resource,
          action: p.action
        })),
        reason: data.reason
      })
    });

    data.walletAddresses.forEach((addr: string) => {
      revalidatePath(`/users/${addr}`);
    });
    revalidatePath('/users');

    return createSuccessResult(response, 'Bulk permission removal completed');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to bulk remove permissions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to remove permissions');
  }
}

// ============================================================================
// TEMPORARY PERMISSION OPERATIONS
// ============================================================================

/**
 * Assign temporary permission with expiration
 * @param data
 * @param data.walletAddress
 * @param data.resource
 * @param data.action
 * @param data.expires
 * @param data.reason
 */
export async function assignTemporaryPermission(data: {
  walletAddress: string;
  resource: string;
  action: string;
  expires: Date;
  reason?: string;
}): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/api/admin/casbin/temporary-policies', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.walletAddress,
        object: data.resource,
        action: data.action,
        expires_at: data.expires.toISOString(),
        reason: data.reason
      })
    });

    revalidatePath(`/users/${data.walletAddress}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Temporary permission assigned successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to assign temporary permission:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to assign temporary permission');
  }
}

/**
 * Get all permissions that are expiring soon
 * @param days
 */
export async function getExpiringPermissions(days = 7): Promise<ActionResult<any[]>> {
  try {
    const response = await makeAuthenticatedRequest(`/api/admin/casbin/expiring-permissions?days=${days}`);
    return createSuccessResult(response.permissions || []);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get expiring permissions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch expiring permissions');
  }
}

// ============================================================================
// PERMISSION ANALYSIS OPERATIONS
// ============================================================================

/**
 * Validate permission assignment for conflicts
 * @param data
 * @param data.walletAddress
 * @param data.resource
 * @param data.action
 */
export async function validatePermissionAssignment(data: {
  walletAddress: string;
  resource: string;
  action: string;
}): Promise<ActionResult<{ conflicts: any[]; warnings: string[] }>> {
  try {
    const response = await makeAuthenticatedRequest('/api/admin/casbin/validate-assignment', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.walletAddress,
        object: data.resource,
        action: data.action
      })
    });

    return createSuccessResult({ conflicts: response.conflicts || [], warnings: response.warnings || [] });
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to validate permission assignment:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to validate permission');
  }
}

/**
 * Get permission impact analysis for a user
 * @param walletAddress
 */
export async function getPermissionImpact(walletAddress: string): Promise<ActionResult<{ canAccess: string[]; cannotAccess: string[]; totalResources: number }>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/permission-impact - DEAD CODE
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/permission-impact`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get permission impact:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to get permission impact');
  }
}

// ============================================================================
// ACTIVITY & HISTORY OPERATIONS
// ============================================================================

/**
 * Get permission history for a user
 * @param walletAddress
 * @param limit
 */
export async function getPermissionHistory(walletAddress: string, limit = 50): Promise<ActionResult<PermissionHistoryEntry[]>> {
  try {
    // NOTE: Backend does not implement /wallets/:wallet_address/activity - DEAD CODE
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/activity?limit=${limit}`);

    const history: PermissionHistoryEntry[] = (response.activities || [])
      .filter((activity: any) =>
        activity.action?.includes('permission') ||
        activity.action?.includes('role') ||
        activity.action?.includes('profile')
      )
      .map((activity: any) => ({
        id: activity.id,
        walletAddress,
        action: activity.action?.includes('granted') ? 'granted' :
          activity.action?.includes('revoked') ? 'revoked' : 'modified',
        type: activity.action?.includes('role') ? 'role' :
          activity.action?.includes('profile') ? 'profile' : 'permission',
        resource: activity.resource || '',
        permission: activity.details?.permission || '',
        role: activity.details?.role || '',
        profileId: activity.details?.profile_id || '',
        reason: activity.details?.reason || '',
        grantedBy: activity.performed_by || 'System',
        grantedAt: new Date(activity.created_at || activity.timestamp),
        expires: activity.expires_at ? new Date(activity.expires_at) : undefined
      }));

    return createSuccessResult(history);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get permission history:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch permission history');
  }
}

/**
 * Get comprehensive activity logs for a user
 * @param walletAddress
 * @param params
 */
export async function getWalletActivityLogs(walletAddress: string, params: ActivityLogParams = {}): Promise<ActionResult<{
  activities: ActivityLogEntry[];
  statistics: {
    total_activities: number;
    login_activities: number;
    failed_activities: number;
    recent_activities: number;
    activity_breakdown: Record<string, number>;
  };
  pagination: {
    limit: number;
    offset: number;
    total: number;
  };
}>> {
  try {
    const queryParams = new URLSearchParams();
    if (params.limit) { queryParams.append('limit', params.limit.toString()); }
    if (params.offset) { queryParams.append('offset', params.offset.toString()); }
    if (params.start_date) { queryParams.append('start_date', params.start_date); }
    if (params.end_date) { queryParams.append('end_date', params.end_date); }
    if (params.action_type) { queryParams.append('action_type', params.action_type); }

    // NOTE: Backend does not implement /wallets/:wallet_address/activity - DEAD CODE
    const response = await makeAuthenticatedRequest(`/api/admin/wallets/${walletAddress}/activity?${queryParams.toString()}`);

    const activities: ActivityLogEntry[] = (response.activities || []).map((activity: any) => ({
      id: activity.id,
      action: activity.action,
      resource_type: activity.resource_type,
      resource_id: activity.resource_id,
      result: activity.result,
      timestamp: new Date(activity.timestamp),
      client_ip: activity.client_ip,
      user_agent: activity.user_agent,
      session_id: activity.session_id,
      metadata: activity.metadata || {}
    }));

    return createSuccessResult({
      activities,
      statistics: {
        total_activities: response.statistics?.total_activities || activities.length,
        login_activities: response.statistics?.login_activities || 0,
        failed_activities: response.statistics?.failed_activities || 0,
        recent_activities: response.statistics?.recent_activities || 0,
        activity_breakdown: response.statistics?.activity_breakdown || {}
      },
      pagination: {
        limit: params.limit || 50,
        offset: params.offset || 0,
        total: response.statistics?.total_activities || activities.length
      }
    });
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to get user activity logs:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch activity logs');
  }
}

// ============================================================================
// USER SEARCH OPERATIONS
// ============================================================================

/**
 * Search users with server-side authentication
 * @param searchParams
 * @param searchParams.search
 * @param searchParams.email
 * @param searchParams.package_tier
 * @param searchParams.status
 * @param searchParams.page
 * @param searchParams.per_page
 * @param searchParams.sort_by
 * @param searchParams.sort_order
 */
export async function searchWalletsAction(searchParams: {
  search?: string;
  email?: string;
  package_tier?: string;
  status?: string;
  page?: number;
  per_page?: number;
  sort_by?: string;
  sort_order?: string;
}): Promise<ActionResult<{
  users: any[];
  total: number;
  page: number;
  per_page: number;
}>> {
  try {
    const queryParams = new URLSearchParams();

    Object.entries(searchParams).forEach(([key, value]) => {
      if (value !== undefined && value !== null && value !== '') {
        queryParams.append(key, value.toString());
      }
    });

    const response = await makeAuthenticatedRequest(`/api/admin/wallets/search?${queryParams.toString()}`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to search users:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to search users');
  }
}