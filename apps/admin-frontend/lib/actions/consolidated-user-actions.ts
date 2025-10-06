/**
 * Consolidated User Actions
 * Combines: user-list-actions.ts, user-list-focused-actions.ts, user-profile-actions.ts, 
 * user-permissions-actions.ts, app/actions/user-actions.ts, and users.ts
 * 
 * This file contains ALL user management operations including:
 * - User CRUD operations
 * - Permission management (roles, profiles, custom permissions)
 * - Bulk operations
 * - Temporary permissions
 * - Activity logging and history
 * - Permission analysis and validation
 * - User search functionality
 */

'use server';

import { revalidatePath } from 'next/cache';
import { redirect } from 'next/navigation';

import { makeAuthenticatedRequest } from './shared-utils';

import { createSuccessResult, createErrorResult, type ActionResult } from '@/lib/action-utils';
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

export interface UserProfileUpdateData {
  displayName?: string;
  firstName?: string;
  lastName?: string;
  phoneNumber?: string;
  timezone?: string;
  language?: string;
}

export interface UserStatusUpdateData {
  status: 'active' | 'inactive' | 'suspended';
  reason?: string;
}

export interface UserRoleUpdateData {
  roles: string[];
  reason?: string;
}

export interface ModuleAccessUpdateData {
  modules: string[];
  quotas?: Record<string, number>;
  reason?: string;
}

export interface PermissionHistoryEntry {
  id: string;
  userId: string;
  action: 'granted' | 'revoked' | 'modified';
  type: 'role' | 'permission' | 'profile';
  resource?: string;
  permission?: string;
  role?: string;
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
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to search users:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to search users');
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
 * @param userId
 */
export async function getUserProfile(userId: string): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}`);
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
export async function createUser(userData: CreateUserRequest): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/users', {
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
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update user');
  }
}

/**
 * Delete user
 * @param userId
 */
export async function deleteUser(userId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${userId}`, {
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
 * @param userId
 */
export async function toggleUserStatus(userId: string): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/toggle-status`, {
      method: 'PATCH'
    });

    revalidatePath('/users');
    revalidatePath(`/users/${userId}`);
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
 * @param userId
 */
export async function getUserPermissions(userId: string): Promise<ActionResult<{
  permissions: string[];
  roles: string[];
  profiles: string[];
}>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/permissions`);
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
 * @param userIds
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
 * @param userId
 */
export async function getUnifiedUserData(userId: string): Promise<ActionResult<UnifiedUserData>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/unified`);
    
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
      
      roles: [],
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
 * @param userId
 * @param data
 */
export async function updateUserProfile(userId: string, data: UserProfileUpdateData): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${userId}/profile`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${userId}`);
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
 * @param userId
 * @param data
 */
export async function updateUserStatus(userId: string, data: UserStatusUpdateData): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${userId}/status`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Status updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user status:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update status');
  }
}

/**
 * Update user roles
 * @param userId
 * @param data
 */
export async function updateUserRoles(userId: string, data: UserRoleUpdateData): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${userId}/roles`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Roles updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update user roles:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update roles');
  }
}

/**
 * Update module access
 * @param userId
 * @param data
 */
export async function updateModuleAccess(userId: string, data: ModuleAccessUpdateData): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/users/${userId}/modules`, {
      method: 'PUT',
      body: JSON.stringify(data)
    });

    revalidatePath(`/users/${userId}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Module access updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update module access:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update module access');
  }
}

// ============================================================================
// ROLE MANAGEMENT OPERATIONS
// ============================================================================

/**
 * Assign role to user
 * @param data
 * @param data.userId
 * @param data.role
 * @param data.reason
 */
export async function assignUserRole(data: { userId: string; role: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/casbin/roles', {
      method: 'POST',
      body: JSON.stringify({
        user: data.userId,
        role: data.role
      })
    });

    revalidatePath(`/users/${data.userId}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Role assigned successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to assign user role:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to assign role');
  }
}

/**
 * Remove role from user
 * @param data
 * @param data.userId
 * @param data.role
 * @param data.reason
 */
export async function removeUserRole(data: { userId: string; role: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/casbin/roles', {
      method: 'DELETE',
      body: JSON.stringify({
        user: data.userId,
        role: data.role
      })
    });

    revalidatePath(`/users/${data.userId}`);
    revalidatePath('/users');
    return createSuccessResult(undefined, 'Role removed successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to remove user role:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to remove role');
  }
}

// ============================================================================
// PERMISSION PROFILE OPERATIONS
// ============================================================================

/**
 * Assign permission profile to user
 * @param data
 * @param data.userId
 * @param data.profileId
 * @param data.reason
 */
export async function assignPermissionProfile(data: { userId: string; profileId: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/permission-profiles/assign', {
      method: 'POST',
      body: JSON.stringify({
        profile_id: data.profileId,
        user_ids: [data.userId]
      })
    });

    revalidatePath(`/users/${data.userId}`);
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
 * @param data.userId
 * @param data.resource
 * @param data.action
 * @param data.reason
 */
export async function addCustomPermission(data: { userId: string; resource: string; action: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/casbin/policies', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action
      })
    });

    revalidatePath(`/users/${data.userId}`);
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
 * @param data.userId
 * @param data.resource
 * @param data.action
 * @param data.reason
 */
export async function removeCustomPermission(data: { userId: string; resource: string; action: string; reason?: string }): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/casbin/policies', {
      method: 'DELETE',
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action
      })
    });

    revalidatePath(`/users/${data.userId}`);
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
 * @param data.userIds
 * @param data.permissions
 * @param data.reason
 */
export async function bulkAssignPermissions(data: {
  userIds: string[];
  permissions: { resource: string; action: string }[];
  reason?: string;
}): Promise<ActionResult<{ succeeded: string[]; failed: { userId: string; error: string }[] }>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/casbin/bulk-assign', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: data.userIds,
        policies: data.permissions.map(p => ({
          object: p.resource,
          action: p.action
        })),
        reason: data.reason
      })
    });

    data.userIds.forEach(userId => {
      revalidatePath(`/users/${userId}`);
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
 * @param data.userIds
 * @param data.permissions
 * @param data.reason
 */
export async function bulkRemovePermissions(data: {
  userIds: string[];
  permissions: { resource: string; action: string }[];
  reason?: string;
}): Promise<ActionResult<{ succeeded: string[]; failed: { userId: string; error: string }[] }>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/casbin/bulk-remove', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: data.userIds,
        policies: data.permissions.map(p => ({
          object: p.resource,
          action: p.action
        })),
        reason: data.reason
      })
    });

    data.userIds.forEach(userId => {
      revalidatePath(`/users/${userId}`);
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
 * @param data.userId
 * @param data.resource
 * @param data.action
 * @param data.expires
 * @param data.reason
 */
export async function assignTemporaryPermission(data: {
  userId: string;
  resource: string;
  action: string;
  expires: Date;
  reason?: string;
}): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/casbin/temporary-policies', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.userId,
        object: data.resource,
        action: data.action,
        expires_at: data.expires.toISOString(),
        reason: data.reason
      })
    });

    revalidatePath(`/users/${data.userId}`);
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
    const response = await makeAuthenticatedRequest(`/admin/casbin/expiring-permissions?days=${days}`);
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
 * @param data.userId
 * @param data.resource
 * @param data.action
 */
export async function validatePermissionAssignment(data: {
  userId: string;
  resource: string;
  action: string;
}): Promise<ActionResult<{ conflicts: any[]; warnings: string[] }>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/casbin/validate-assignment', {
      method: 'POST',
      body: JSON.stringify({
        subject: data.userId,
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
 * @param userId
 */
export async function getPermissionImpact(userId: string): Promise<ActionResult<{ canAccess: string[]; cannotAccess: string[]; totalResources: number }>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/permission-impact`);
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
 * @param userId
 * @param limit
 */
export async function getPermissionHistory(userId: string, limit = 50): Promise<ActionResult<PermissionHistoryEntry[]>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/activity?limit=${limit}`);
    
    const history: PermissionHistoryEntry[] = (response.activities || [])
      .filter((activity: any) => 
        activity.action?.includes('permission') || 
        activity.action?.includes('role') || 
        activity.action?.includes('profile')
      )
      .map((activity: any) => ({
        id: activity.id,
        userId,
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
 * @param userId
 * @param params
 */
export async function getUserActivityLogs(userId: string, params: ActivityLogParams = {}): Promise<ActionResult<{
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
    if (params.limit) {queryParams.append('limit', params.limit.toString());}
    if (params.offset) {queryParams.append('offset', params.offset.toString());}
    if (params.start_date) {queryParams.append('start_date', params.start_date);}
    if (params.end_date) {queryParams.append('end_date', params.end_date);}
    if (params.action_type) {queryParams.append('action_type', params.action_type);}
    
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/activity?${queryParams.toString()}`);
    
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
export async function searchUsersAction(searchParams: {
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
    
    const response = await makeAuthenticatedRequest(`/admin/users/search?${queryParams.toString()}`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to search users:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to search users');
  }
}