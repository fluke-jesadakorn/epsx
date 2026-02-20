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
import { logger } from '@/lib/logger';
import type { Permission, UnifiedWalletData } from '@/lib/types/wallet';

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

    const response = await makeAuthenticatedRequest<{
      users: UnifiedWalletData[];
      totalCount: number;
      currentPage: number;
      totalPages: number;
    }>(`/api/admin/wallets?${params.toString()}`);

    return createSuccessResult(response);
  } catch (_error) {

    logger.error('Failed to fetch user list:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to fetch users');
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

    const response = await makeAuthenticatedRequest<{ users?: UnifiedWalletData[] }>(`/api/admin/wallets/search?${params.toString()}`);

    return createSuccessResult(response.users ?? []);
  } catch (_error) {

    logger.error('Failed to search users:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to search users');
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
    const response = await makeAuthenticatedRequest<{
      totalUsers: number;
      activeUsers: number;
      newUsersThisMonth: number;
      totalAdmins: number;
    }>('/api/admin/wallets/stats');
    return createSuccessResult(response);
  } catch (_error) {

    logger.error('Failed to fetch user stats:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to fetch user statistics');
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
    const response = await makeAuthenticatedRequest<UnifiedWalletData>(`/api/admin/wallets/${walletAddress}`);
    return createSuccessResult(response);
  } catch (_error) {

    logger.error('Failed to fetch user profile:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to fetch user profile');
  }
}

/**
 * Create new user
 * @param userData
 */
export async function createWallet(userData: CreateWalletRequest): Promise<ActionResult<UnifiedWalletData>> {
  try {
    const response = await makeAuthenticatedRequest<UnifiedWalletData>('/api/admin/wallets', {
      method: 'POST',
      body: JSON.stringify(userData)
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'User created successfully');
  } catch (_error) {

    logger.error('Failed to create user:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to create user');
  }
}

/**
 * Update user profile
 * @param userData
 */
export async function updateWallet(userData: UpdateWalletRequest): Promise<ActionResult<UnifiedWalletData>> {
  try {
    const { id, ...updateData } = userData;

    const response = await makeAuthenticatedRequest<UnifiedWalletData>(`/api/admin/wallets/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updateData)
    });

    revalidatePath('/users');
    revalidatePath(`/users/${id}`);
    return createSuccessResult(response, 'User updated successfully');
  } catch (_error) {

    logger.error('Failed to update user:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to update user');
  }
}

/**
 * Delete user
 * @param walletAddress
 */
export async function deleteWallet(walletAddress: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest<void>(`/api/admin/wallets/${walletAddress}`, {
      method: 'DELETE'
    });

    revalidatePath('/users');
    return createSuccessResult(undefined, 'User deleted successfully');
  } catch (_error) {

    logger.error('Failed to delete user:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to delete user');
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
    const response = await makeAuthenticatedRequest<{
      permissions: string[];
      groups: string[];
      profiles: string[];
    }>(`/api/admin/permissions/wallets/${walletAddress}/permissions`);
    return createSuccessResult(response);
  } catch (_error) {

    logger.error('Failed to fetch user permissions:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to fetch user permissions');
  }
}

/**
 * Update user permissions
 * @param change
 */
export async function updateWalletPermissions(change: WalletPermissionChange): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest<void>(`/api/admin/permissions/wallets/${change.walletAddress}/permissions`, {
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

    logger.error('Failed to update user permissions:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to update user permissions');
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
    const response = await makeAuthenticatedRequest<{
      successful: number;
      failed: number;
      errors: string[];
    }>('/api/admin/wallets/permissions/bulk', {
      method: 'PUT',
      body: JSON.stringify({ changes })
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'Bulk permission update completed');
  } catch (_error) {

    logger.error('Failed to perform bulk permission update:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to update permissions');
  }
}

// ============================================================================
// BULK USER OPERATIONS
// ============================================================================

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
    await makeAuthenticatedRequest<void>('/api/admin/casbin/groups', {
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

    logger.error('Failed to assign user group:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to assign group');
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
    await makeAuthenticatedRequest<void>('/api/admin/casbin/groups', {
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

    logger.error('Failed to remove user group:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to remove group');
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
    await makeAuthenticatedRequest<void>('/api/admin/permission-profiles/assign', {
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

    logger.error('Failed to assign permission profile:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to assign permission profile');
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
    await makeAuthenticatedRequest<void>('/api/admin/casbin/policies', {
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

    logger.error('Failed to add custom permission:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to add permission');
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
    await makeAuthenticatedRequest<void>('/api/admin/casbin/policies', {
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

    logger.error('Failed to remove custom permission:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to remove permission');
  }
}

// ============================================================================
// BULK PERMISSION OPERATIONS
// ============================================================================

/**
 * Bulk assign permissions to multiple users
 * @param data
 * @param data.walletAddresses
 * @param data.permissions
 * @param data.reason
 */
export async function bulkAssignPermissions(data: {
  walletAddresses: string[];
  permissions: { resource: string; action: string }[];
  reason?: string;
}): Promise<ActionResult<{ succeeded: string[]; failed: { walletAddress: string; error: string }[] }>> {
  try {
    const response = await makeAuthenticatedRequest<{ succeeded: string[]; failed: { walletAddress: string; error: string }[] }>('/api/admin/casbin/bulk-assign', {
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
    logger.error('Failed to bulk assign permissions:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to assign permissions');
  }
}

/**
 * Bulk remove permissions from multiple users
 * @param data
 * @param data.walletAddresses
 * @param data.permissions
 * @param data.reason
 */
export async function bulkRemovePermissions(data: {
  walletAddresses: string[];
  permissions: { resource: string; action: string }[];
  reason?: string;
}): Promise<ActionResult<{ succeeded: string[]; failed: { walletAddress: string; error: string }[] }>> {
  try {
    const response = await makeAuthenticatedRequest<{ succeeded: string[]; failed: { walletAddress: string; error: string }[] }>('/api/admin/casbin/bulk-remove', {
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
    logger.error('Failed to bulk remove permissions:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to remove permissions');
  }
}

// ... (skipping to validatePermissionAssignment)

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
}): Promise<ActionResult<{ conflicts: Permission[]; warnings: string[] }>> {
  try {
    const response = await makeAuthenticatedRequest<{ conflicts?: Permission[]; warnings?: string[] }>('/api/admin/casbin/validate-assignment', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.walletAddress,
        object: data.resource,
        action: data.action
      })
    });

    return createSuccessResult({ conflicts: response.conflicts ?? [], warnings: response.warnings ?? [] });
  } catch (_error) {
    logger.error('Failed to validate permission assignment:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to validate permission');
  }
}

// ... (skipping to searchWalletsAction)

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
  users: UnifiedWalletData[];
  total: number;
  page: number;
  per_page: number;
}>> {
  try {
    const queryParams = new URLSearchParams();

    Object.entries(searchParams).forEach(([key, value]) => {

      if (typeof value === 'string' && value === '') {
        return;
      }
      queryParams.append(key, value.toString());
    });

    const response = await makeAuthenticatedRequest<{
      users: UnifiedWalletData[];
      total: number;
      page: number;
      per_page: number;
    }>(`/api/admin/wallets/search?${queryParams.toString()}`);
    return createSuccessResult(response);
  } catch (_error) {
    logger.error('Failed to search users:', { error: _error });
    return createErrorResult<never>(_error instanceof Error ? _error.message : 'Failed to search users');
  }
}