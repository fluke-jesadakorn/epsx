/**
 * Consolidated User Actions
 * Combines: user-list-actions.ts, user-list-focused-actions.ts, user-profile-actions.ts, 
 * user-permissions-actions.ts, app/actions/user-actions.ts
 */

'use server';

import { revalidatePath } from 'next/cache';
import { redirect } from 'next/navigation';
import { makeAuthenticatedRequest, createSuccessResult, createErrorResult, type ActionResult } from './shared-utils';
import type { UnifiedUserData, UserOperationResult } from '@/lib/types/unified-user';

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

export interface UserListFilters {
  search: string;
  status: string;
  role: string;
  page: number;
  limit: number;
  sortBy: string;
  sortOrder: 'asc' | 'desc';
}

export interface CreateUserRequest {
  email: string;
  name?: string;
  role?: string;
  permissions?: string[];
  sendInvite?: boolean;
}

export interface UpdateUserRequest {
  id: string;
  email?: string;
  name?: string;
  role?: string;
  isActive?: boolean;
  permissions?: string[];
}

export interface UserPermissionChange {
  userId: string;
  permissions: string[];
  action: 'grant' | 'revoke' | 'replace';
  expiresAt?: string;
  reason?: string;
}

// ============================================================================
// USER LIST OPERATIONS
// ============================================================================

/**
 * Get paginated user list with filters
 */
export async function getUserList(filters: UserListFilters): Promise<ActionResult<{
  users: UnifiedUserData[];
  totalCount: number;
  currentPage: number;
  totalPages: number;
}>> {
  try {
    const params = new URLSearchParams({
      search: filters.search,
      status: filters.status,
      role: filters.role,
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sortBy: filters.sortBy,
      sortOrder: filters.sortOrder
    });

    const response = await makeAuthenticatedRequest(`/admin/users?${params.toString()}`);
    
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to fetch user list:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch users');
  }
}

/**
 * Search users with enhanced filtering
 */
export async function searchUsers(query: string, filters?: Partial<UserListFilters>): Promise<ActionResult<UnifiedUserData[]>> {
  try {
    const params = new URLSearchParams({
      q: query,
      ...filters && Object.fromEntries(
        Object.entries(filters).map(([key, value]) => [key, String(value)])
      )
    });

    const response = await makeAuthenticatedRequest(`/admin/users/search?${params.toString()}`);
    
    return createSuccessResult(response.users || []);
  } catch (error) {
    console.error('Failed to search users:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to search users');
  }
}

/**
 * Get user statistics for dashboard
 */
export async function getUserStats(): Promise<ActionResult<{
  totalUsers: number;
  activeUsers: number;
  newUsersThisMonth: number;
  totalAdmins: number;
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/users/stats');
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to fetch user stats:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch user statistics');
  }
}

// ============================================================================
// USER PROFILE OPERATIONS
// ============================================================================

/**
 * Get user profile by ID
 */
export async function getUserProfile(userId: string): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}`);
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to fetch user profile:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch user profile');
  }
}

/**
 * Create new user
 */
export async function createUser(userData: CreateUserRequest): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/users', {
      method: 'POST',
      body: JSON.stringify(userData)
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'User created successfully');
  } catch (error) {
    console.error('Failed to create user:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to create user');
  }
}

/**
 * Update user profile
 */
export async function updateUser(userData: UpdateUserRequest): Promise<ActionResult<UnifiedUserData>> {
  try {
    const { id, ...updateData } = userData;
    
    const response = await makeAuthenticatedRequest(`/admin/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updateData)
    });

    revalidatePath('/users');
    revalidatePath(`/users/${id}`);
    return createSuccessResult(response, 'User updated successfully');
  } catch (error) {
    console.error('Failed to update user:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update user');
  }
}

/**
 * Delete user
 */
export async function deleteUser(userId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${userId}`, {
      method: 'DELETE'
    });

    revalidatePath('/users');
    return createSuccessResult(undefined, 'User deleted successfully');
  } catch (error) {
    console.error('Failed to delete user:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to delete user');
  }
}

/**
 * Toggle user active status
 */
export async function toggleUserStatus(userId: string): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/toggle-status`, {
      method: 'PATCH'
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
    return createSuccessResult(response, 'User status updated successfully');
  } catch (error) {
    console.error('Failed to toggle user status:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update user status');
  }
}

// ============================================================================
// USER PERMISSION OPERATIONS
// ============================================================================

/**
 * Get user permissions
 */
export async function getUserPermissions(userId: string): Promise<ActionResult<{
  permissions: string[];
  roles: string[];
  profiles: string[];
}>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/permissions`);
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to fetch user permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch user permissions');
  }
}

/**
 * Update user permissions
 */
export async function updateUserPermissions(change: UserPermissionChange): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${change.userId}/permissions`, {
      method: 'PUT',
      body: JSON.stringify({
        permissions: change.permissions,
        action: change.action,
        expiresAt: change.expiresAt,
        reason: change.reason
      })
    });

    revalidatePath('/users');
    revalidatePath(`/users/${change.userId}`);
    return createSuccessResult(undefined, 'User permissions updated successfully');
  } catch (error) {
    console.error('Failed to update user permissions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update user permissions');
  }
}

/**
 * Bulk permission updates
 */
export async function bulkUpdateUserPermissions(changes: UserPermissionChange[]): Promise<ActionResult<{
  successful: number;
  failed: number;
  errors: string[];
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/users/permissions/bulk', {
      method: 'PUT',
      body: JSON.stringify({ changes })
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'Bulk permission update completed');
  } catch (error) {
    console.error('Failed to perform bulk permission update:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update permissions');
  }
}

// ============================================================================
// BULK USER OPERATIONS
// ============================================================================

/**
 * Bulk delete users
 */
export async function bulkDeleteUsers(userIds: string[]): Promise<ActionResult<{
  successful: number;
  failed: number;
  errors: string[];
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/users/bulk-delete', {
      method: 'DELETE',
      body: JSON.stringify({ userIds })
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'Bulk delete completed');
  } catch (error) {
    console.error('Failed to perform bulk delete:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to delete users');
  }
}

/**
 * Export users to CSV
 */
export async function exportUsers(filters?: Partial<UserListFilters>): Promise<ActionResult<{
  downloadUrl: string;
  filename: string;
}>> {
  try {
    const params = filters ? new URLSearchParams(
      Object.fromEntries(
        Object.entries(filters).map(([key, value]) => [key, String(value)])
      )
    ).toString() : '';

    const response = await makeAuthenticatedRequest(`/admin/users/export?${params}`);
    return createSuccessResult(response);
  } catch (error) {
    console.error('Failed to export users:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to export users');
  }
}