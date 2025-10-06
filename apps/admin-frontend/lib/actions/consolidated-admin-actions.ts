/**
 * Consolidated Admin Actions
 * Combines: admin-actions.ts, admin-notification-actions.ts, page-actions.ts, admin.ts, and auth.ts
 * 
 * This file contains ALL admin management operations including:
 * - Admin notifications management
 * - System configuration and health monitoring
 * - Audit logs and session management
 * - User management operations
 * - Permission profile assignments
 * - Stock ranking package management
 * - Cache and session cleanup
 * - Authentication operations (sign out)
 */

'use server';

import { revalidatePath } from 'next/cache';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

import { makeAuthenticatedRequest } from './shared-utils';

import { env } from '@/config/env';
import { createSuccessResult, createErrorResult, type ActionResult } from '@/lib/action-utils';
import { getServerSession } from '@/lib/server/auth';

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get bearer token from custom JWT session
 */
export const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

export interface AdminNotification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'warning' | 'error' | 'success';
  priority: 'low' | 'medium' | 'high';
  targetUsers?: string[];
  targetRoles?: string[];
  createdAt: string;
  expiresAt?: string;
  isActive: boolean;
  readBy: string[];
}

export interface SystemConfig {
  [key: string]: any;
}

export interface AuditLogEntry {
  id: string;
  userId: string;
  userEmail: string;
  action: string;
  resource: string;
  details: Record<string, any>;
  ipAddress: string;
  userAgent: string;
  timestamp: string;
  success: boolean;
}

export interface AssignmentResult {
  success: boolean;
  message?: string;
  data?: any;
}

export interface StockRankingAssignmentUpdateRequest {
  status?: string;
  expires_at?: string;
  package_tier?: string;
}

// ============================================================================
// ADMIN NOTIFICATION OPERATIONS
// ============================================================================

/**
 * Get all admin notifications
 */
export async function getAdminNotifications(): Promise<ActionResult<AdminNotification[]>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/notifications');
    return createSuccessResult(response.notifications || []);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch admin notifications:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch notifications');
  }
}

/**
 * Create admin notification
 * @param notification
 * @param notification.title
 * @param notification.message
 * @param notification.type
 * @param notification.priority
 * @param notification.targetUsers
 * @param notification.targetRoles
 * @param notification.expiresAt
 */
export async function createAdminNotification(notification: {
  title: string;
  message: string;
  type: 'info' | 'warning' | 'error' | 'success';
  priority: 'low' | 'medium' | 'high';
  targetUsers?: string[];
  targetRoles?: string[];
  expiresAt?: string;
}): Promise<ActionResult<AdminNotification>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/notifications', {
      method: 'POST',
      body: JSON.stringify(notification)
    });

    revalidatePath('/admin');
    return createSuccessResult(response, 'Notification created successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to create admin notification:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to create notification');
  }
}

/**
 * Update admin notification
 * @param notificationId
 * @param updates
 */
export async function updateAdminNotification(
  notificationId: string,
  updates: Partial<AdminNotification>
): Promise<ActionResult<AdminNotification>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/notifications/${notificationId}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });

    revalidatePath('/admin');
    return createSuccessResult(response, 'Notification updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update admin notification:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update notification');
  }
}

/**
 * Delete admin notification
 * @param notificationId
 */
export async function deleteAdminNotification(notificationId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/notifications/${notificationId}`, {
      method: 'DELETE'
    });

    revalidatePath('/admin');
    return createSuccessResult(undefined, 'Notification deleted successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to delete admin notification:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to delete notification');
  }
}

/**
 * Mark notification as read
 * @param notificationId
 */
export async function markNotificationAsRead(notificationId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/notifications/${notificationId}/read`, {
      method: 'POST'
    });

    revalidatePath('/admin');
    return createSuccessResult(undefined, 'Notification marked as read');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to mark notification as read:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to mark notification as read');
  }
}

/**
 * Broadcast notification to all users
 * @param notification
 * @param notification.title
 * @param notification.message
 * @param notification.type
 * @param notification.priority
 */
export async function broadcastNotification(notification: {
  title: string;
  message: string;
  type: 'info' | 'warning' | 'error' | 'success';
  priority: 'low' | 'medium' | 'high';
}): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/notifications/broadcast', {
      method: 'POST',
      body: JSON.stringify(notification)
    });

    return createSuccessResult(undefined, 'Notification broadcasted successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to broadcast notification:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to broadcast notification');
  }
}

// ============================================================================
// SYSTEM CONFIGURATION OPERATIONS
// ============================================================================

/**
 * Get system configuration
 */
export async function getSystemConfig(): Promise<ActionResult<SystemConfig>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/config');
    return createSuccessResult(response.config || {});
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch system config:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch system configuration');
  }
}

/**
 * Update system configuration
 * @param config
 */
export async function updateSystemConfig(config: Partial<SystemConfig>): Promise<ActionResult<SystemConfig>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/config', {
      method: 'PUT',
      body: JSON.stringify(config)
    });

    revalidatePath('/admin/settings');
    return createSuccessResult(response, 'System configuration updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update system config:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to update system configuration');
  }
}

/**
 * Reset system configuration to defaults
 */
export async function resetSystemConfig(): Promise<ActionResult<SystemConfig>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/config/reset', {
      method: 'POST'
    });

    revalidatePath('/admin/settings');
    return createSuccessResult(response, 'System configuration reset to defaults');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to reset system config:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to reset system configuration');
  }
}

// ============================================================================
// AUDIT LOG OPERATIONS
// ============================================================================

/**
 * Get audit logs with filtering
 * @param filters
 * @param filters.userId
 * @param filters.action
 * @param filters.resource
 * @param filters.startDate
 * @param filters.endDate
 * @param filters.page
 * @param filters.limit
 */
export async function getAuditLogs(filters: {
  userId?: string;
  action?: string;
  resource?: string;
  startDate?: string;
  endDate?: string;
  page?: number;
  limit?: number;
}): Promise<ActionResult<{
  logs: AuditLogEntry[];
  totalCount: number;
  currentPage: number;
  totalPages: number;
}>> {
  try {
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined) {
        params.append(key, String(value));
      }
    });

    const response = await makeAuthenticatedRequest(`/admin/audit-logs?${params.toString()}`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch audit logs:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch audit logs');
  }
}

/**
 * Export audit logs
 * @param filters
 * @param filters.userId
 * @param filters.action
 * @param filters.resource
 * @param filters.startDate
 * @param filters.endDate
 * @param filters.format
 */
export async function exportAuditLogs(filters: {
  userId?: string;
  action?: string;
  resource?: string;
  startDate?: string;
  endDate?: string;
  format?: 'csv' | 'json';
}): Promise<ActionResult<{
  downloadUrl: string;
  filename: string;
}>> {
  try {
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined) {
        params.append(key, String(value));
      }
    });

    const response = await makeAuthenticatedRequest(`/admin/audit-logs/export?${params.toString()}`);
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to export audit logs:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to export audit logs');
  }
}

// ============================================================================
// SYSTEM HEALTH AND MONITORING
// ============================================================================

/**
 * Get system health status
 */
export async function getSystemHealth(): Promise<ActionResult<{
  status: 'healthy' | 'degraded' | 'unhealthy';
  services: Array<{
    name: string;
    status: 'up' | 'down' | 'degraded';
    responseTime?: number;
    lastCheck: string;
  }>;
  metrics: {
    cpu: number;
    memory: number;
    disk: number;
    activeUsers: number;
    totalRequests: number;
    errorRate: number;
  };
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/health');
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch system health:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch system health');
  }
}

/**
 * Get system statistics
 */
export async function getSystemStats(): Promise<ActionResult<{
  users: {
    total: number;
    active: number;
    newThisMonth: number;
  };
  permissions: {
    total: number;
    temporary: number;
    embedded: number;
  };
  activities: {
    logins: number;
    actions: number;
    errors: number;
  };
  performance: {
    avgResponseTime: number;
    uptime: number;
    cacheHitRate: number;
  };
}>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/stats');
    return createSuccessResult(response);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch system stats:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch system statistics');
  }
}

// ============================================================================
// CACHE AND SESSION MANAGEMENT
// ============================================================================

/**
 * Clear system cache
 * @param cacheType
 */
export async function clearSystemCache(cacheType?: 'all' | 'permissions' | 'users' | 'sessions'): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/cache/clear', {
      method: 'POST',
      body: JSON.stringify({ type: cacheType || 'all' })
    });

    return createSuccessResult(undefined, 'System cache cleared successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to clear system cache:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to clear system cache');
  }
}

/**
 * Get active sessions
 */
export async function getActiveSessions(): Promise<ActionResult<Array<{
  sessionId: string;
  userId: string;
  userEmail: string;
  ipAddress: string;
  userAgent: string;
  createdAt: string;
  lastActivity: string;
  expiresAt: string;
}>>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/sessions');
    return createSuccessResult(response.sessions || []);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch active sessions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch active sessions');
  }
}

/**
 * Revoke user session
 * @param sessionId
 */
export async function revokeUserSession(sessionId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/sessions/${sessionId}`, {
      method: 'DELETE'
    });

    return createSuccessResult(undefined, 'Session revoked successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to revoke session:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to revoke session');
  }
}

// ============================================================================
// ADMIN USER MANAGEMENT OPERATIONS
// ============================================================================

/**
 * Get users with admin authentication
 */
export async function getUsersAction(): Promise<ActionResult<any[]>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/users');
    return createSuccessResult(response.users || []);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to fetch users:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to fetch users');
  }
}

/**
 * Soft delete user
 * @param formData
 */
export async function softDeleteUserAction(formData: FormData): Promise<ActionResult<{ message: string }>> {
  const userId = formData.get('userId') as string;
  const reason = formData.get('reason') as string;

  try {
    const response = await makeAuthenticatedRequest(`/admin/users/${userId}/soft-delete`, {
      method: 'DELETE',
      body: JSON.stringify({
        reason: reason || 'Deleted via admin interface'
      })
    });

    revalidatePath('/users');
    return createSuccessResult(response, 'User deleted successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to soft delete user:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to delete user');
  }
}

// ============================================================================
// PERMISSION PROFILE ASSIGNMENT OPERATIONS
// ============================================================================

/**
 * Assign permission profile to user (form-based)
 * @param formData
 */
export async function assignPermissionProfileAction(formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const profileId = formData.get('profileId') as string;
  const userId = formData.get('userId') as string;
  const expiresAt = formData.get('expiresAt') as string;

  try {
    const response = await makeAuthenticatedRequest('/admin/permission-profiles/assign', {
      method: 'POST',
      body: JSON.stringify({
        profile_id: profileId,
        user_id: userId,
        expires_at: expiresAt || undefined
      })
    });

    revalidatePath('/admin/permission-profiles');
    revalidatePath('/users');
    return createSuccessResult(response, 'Permission profile assigned successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to assign permission profile:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Assignment failed');
  }
}

// ============================================================================
// STOCK RANKING MANAGEMENT OPERATIONS
// ============================================================================

/**
 * Bulk assign stock ranking packages
 * @param formData
 */
export async function assignBulkStockRankingAction(formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const userIds = JSON.parse(formData.get('userIds') as string);
  const packageTier = formData.get('packageTier') as string;
  const expiresAt = formData.get('expiresAt') as string;

  try {
    const response = await makeAuthenticatedRequest('/admin/stock-ranking/bulk-assign', {
      method: 'POST',
      body: JSON.stringify({
        user_ids: userIds,
        package_tier: packageTier,
        expires_at: expiresAt || undefined
      })
    });

    revalidatePath('/admin/stock-ranking');
    revalidatePath('/users');
    return createSuccessResult(response, 'Stock ranking packages assigned successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to assign bulk stock ranking:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Assignment failed');
  }
}

/**
 * Revoke stock ranking assignment
 * @param assignmentId
 */
export async function revokeStockRankingAssignmentAction(assignmentId: string): Promise<ActionResult<AssignmentResult>> {
  try {
    const response = await makeAuthenticatedRequest(`/admin/stock-ranking/assignments/${assignmentId}`, {
      method: 'DELETE'
    });

    revalidatePath('/admin/stock-ranking');
    return createSuccessResult(response, 'Stock ranking assignment revoked successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to revoke stock ranking assignment:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Revocation failed');
  }
}

/**
 * Extend stock ranking assignment
 * @param assignmentId
 * @param formData
 */
export async function extendStockRankingAssignmentAction(assignmentId: string, formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const newExpiresAt = formData.get('newExpiresAt') as string;

  try {
    const response = await makeAuthenticatedRequest(`/admin/stock-ranking/assignments/${assignmentId}/extend`, {
      method: 'PUT',
      body: JSON.stringify({
        new_expires_at: newExpiresAt
      })
    });

    revalidatePath('/admin/stock-ranking');
    return createSuccessResult(response, 'Stock ranking assignment extended successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to extend stock ranking assignment:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Extension failed');
  }
}

/**
 * Update stock ranking assignment
 * @param assignmentId
 * @param formData
 */
export async function updateStockRankingAssignmentAction(assignmentId: string, formData: FormData): Promise<ActionResult<AssignmentResult>> {
  const updateData: StockRankingAssignmentUpdateRequest = {
    status: formData.get('status') as string,
    expires_at: formData.get('expiresAt') as string,
    package_tier: formData.get('packageTier') as string
  };

  try {
    const response = await makeAuthenticatedRequest(`/admin/stock-ranking/assignments/${assignmentId}`, {
      method: 'PUT',
      body: JSON.stringify(updateData)
    });

    revalidatePath('/admin/stock-ranking');
    return createSuccessResult(response, 'Stock ranking assignment updated successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to update stock ranking assignment:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Update failed');
  }
}

// ============================================================================
// AUTHENTICATION OPERATIONS
// ============================================================================

/**
 * Handle admin sign out with OIDC token revocation
 */
export async function handleSignOut(): Promise<void> {
  // Use OIDC logout endpoint to properly revoke tokens
  const backendUrl = env.BACKEND_URL;
  
  // OIDC Migration: Get current OIDC access token for revocation
  const cookieStore = await cookies();
  const jwt = cookieStore.get('access_token')?.value;
  
  if (jwt) {
    try {
      // Call OIDC logout endpoint to revoke token
      await fetch(`${backendUrl}/oauth/logout`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${jwt}`
        }
      });
    } catch (logoutError) {
      // eslint-disable-next-line no-console
      console.error('❌ Backend logout failed:', logoutError);
      // Continue with local logout even if backend fails
    }
  }
  
  // OIDC Migration: Clear OIDC tokens instead of legacy JWT
  cookieStore.delete('access_token');
  cookieStore.delete('id_token'); 
  cookieStore.delete('refresh_token');
  
  // Redirect to login page with proper PKCE flow
  redirect('/login');
}

/**
 * Cleanup expired permissions - alias for clearing permissions cache
 */
export async function cleanupExpiredPermissionsAction(): Promise<ActionResult<void>> {
  try {
    // Call permissions-specific cleanup endpoint
    await makeAuthenticatedRequest('/admin/permissions/cleanup-expired', {
      method: 'POST'
    });

    return createSuccessResult(undefined, 'Expired permissions cleaned up successfully');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Failed to cleanup expired permissions:', _error);
    return createErrorResult(_error instanceof Error ? _error.message : 'Failed to cleanup expired permissions');
  }
}

/**
 * Send notification - alias for createAdminNotification
 */
export const sendNotification = createAdminNotification;

/**
 * Send broadcast notification - alias for broadcastNotification
 */
export const sendBroadcastNotification = broadcastNotification;