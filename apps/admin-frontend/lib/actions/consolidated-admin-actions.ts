/**
 * Consolidated Admin Actions
 * Combines: admin-actions.ts, admin-notification-actions.ts, page-actions.ts
 */

'use server';

import { revalidatePath } from 'next/cache';
import { makeAuthenticatedRequest, createSuccessResult, createErrorResult, type ActionResult } from './shared-utils';

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
  } catch (error) {
    console.error('Failed to fetch admin notifications:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch notifications');
  }
}

/**
 * Create admin notification
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
  } catch (error) {
    console.error('Failed to create admin notification:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to create notification');
  }
}

/**
 * Update admin notification
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
  } catch (error) {
    console.error('Failed to update admin notification:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update notification');
  }
}

/**
 * Delete admin notification
 */
export async function deleteAdminNotification(notificationId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/notifications/${notificationId}`, {
      method: 'DELETE'
    });

    revalidatePath('/admin');
    return createSuccessResult(undefined, 'Notification deleted successfully');
  } catch (error) {
    console.error('Failed to delete admin notification:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to delete notification');
  }
}

/**
 * Mark notification as read
 */
export async function markNotificationAsRead(notificationId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/notifications/${notificationId}/read`, {
      method: 'POST'
    });

    revalidatePath('/admin');
    return createSuccessResult(undefined, 'Notification marked as read');
  } catch (error) {
    console.error('Failed to mark notification as read:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to mark notification as read');
  }
}

/**
 * Broadcast notification to all users
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
  } catch (error) {
    console.error('Failed to broadcast notification:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to broadcast notification');
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
  } catch (error) {
    console.error('Failed to fetch system config:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch system configuration');
  }
}

/**
 * Update system configuration
 */
export async function updateSystemConfig(config: Partial<SystemConfig>): Promise<ActionResult<SystemConfig>> {
  try {
    const response = await makeAuthenticatedRequest('/admin/config', {
      method: 'PUT',
      body: JSON.stringify(config)
    });

    revalidatePath('/admin/settings');
    return createSuccessResult(response, 'System configuration updated successfully');
  } catch (error) {
    console.error('Failed to update system config:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to update system configuration');
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
  } catch (error) {
    console.error('Failed to reset system config:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to reset system configuration');
  }
}

// ============================================================================
// AUDIT LOG OPERATIONS
// ============================================================================

/**
 * Get audit logs with filtering
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
  } catch (error) {
    console.error('Failed to fetch audit logs:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch audit logs');
  }
}

/**
 * Export audit logs
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
  } catch (error) {
    console.error('Failed to export audit logs:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to export audit logs');
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
  } catch (error) {
    console.error('Failed to fetch system health:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch system health');
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
  } catch (error) {
    console.error('Failed to fetch system stats:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch system statistics');
  }
}

// ============================================================================
// CACHE AND SESSION MANAGEMENT
// ============================================================================

/**
 * Clear system cache
 */
export async function clearSystemCache(cacheType?: 'all' | 'permissions' | 'users' | 'sessions'): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest('/admin/cache/clear', {
      method: 'POST',
      body: JSON.stringify({ type: cacheType || 'all' })
    });

    return createSuccessResult(undefined, 'System cache cleared successfully');
  } catch (error) {
    console.error('Failed to clear system cache:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to clear system cache');
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
  } catch (error) {
    console.error('Failed to fetch active sessions:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to fetch active sessions');
  }
}

/**
 * Revoke user session
 */
export async function revokeUserSession(sessionId: string): Promise<ActionResult<void>> {
  try {
    await makeAuthenticatedRequest(`/admin/sessions/${sessionId}`, {
      method: 'DELETE'
    });

    return createSuccessResult(undefined, 'Session revoked successfully');
  } catch (error) {
    console.error('Failed to revoke session:', error);
    return createErrorResult(error instanceof Error ? error.message : 'Failed to revoke session');
  }
}