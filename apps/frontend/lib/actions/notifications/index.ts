'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { requireAuth, requirePermission } from '../auth';

// ============================================================================
// Types
// ============================================================================

export interface Notification {
  id: string;
  title: string;
  body: string;
  type: 'system' | 'admin' | 'data' | 'feature' | 'security';
  priority: 'low' | 'normal' | 'high' | 'urgent';
  sender: 'system' | 'admin' | 'automated';
  imageUrl?: string;
  actionUrl?: string;
  customData?: Record<string, unknown>;
  createdAt: string;
  readAt?: string;
  clickedAt?: string;
  deliveredAt?: string;
  expiresAt?: string;
}

export interface NotificationData {
  notifications: Notification[];
  unreadCount: number;
  totalCount: number;
}

// ============================================================================
// Notification Server Actions
// ============================================================================

const getClient = () => createApiClient();

/**
 * Get user notifications
 */
export async function getUserNotifications(params?: {
  page?: number;
  per_page?: number;
  status?: 'read' | 'unread' | 'all';
}): Promise<NotificationData> {
  try {
    await requireAuth();
    
    const client = getClient();
    const result = await client.getNotifications(params);
    
    // Transform the response to match expected NotificationData interface
    const notifications = result.data.map((notification: Record<string, unknown>) => ({
      id: notification.id as string,
      title: notification.title as string,
      body: notification.message as string,
      type: notification.type as Notification['type'],
      priority: notification.priority as Notification['priority'],
      createdAt: notification.createdAt as string,
      readAt: notification.readAt as string | undefined,
      actionUrl: notification.actionUrl as string | undefined,
    }));

    const unreadCount = notifications.filter(n => !n.readAt).length;
    
    return {
      notifications,
      unreadCount,
      totalCount: result.pagination.total_items,
    };
  } catch (error) {
    console.error('Get user notifications error:', error);
    throw error;
  }
}

/**
 * Mark notification as read
 */
export async function markNotificationRead(notificationId: string): Promise<void> {
  try {
    await requireAuth();
    
    const client = getClient();
    await client.markNotificationRead(notificationId);
  } catch (error) {
    console.error('Mark notification read error:', error);
    throw error;
  }
}

/**
 * Mark all notifications as read
 */
export async function markAllNotificationsRead(): Promise<void> {
  try {
    await requireAuth();
    
    const client = getClient();
    await client.markAllNotificationsRead();
  } catch (error) {
    console.error('Mark all notifications read error:', error);
    throw error;
  }
}

/**
 * Delete notification
 */
export async function deleteNotification(notificationId: string): Promise<void> {
  try {
    await requireAuth();
    
    const client = getClient();
    await client.deleteNotification(notificationId);
  } catch (error) {
    console.error('Delete notification error:', error);
    throw error;
  }
}

/**
 * Get notification preferences
 */
export async function getNotificationPreferences(): Promise<Record<string, unknown>> {
  try {
    await requireAuth();

    const client = getClient();
    const result = await client.get('/api/user/notification-preferences');

    return result.data as Record<string, unknown>;
  } catch (error) {
    console.error('Get notification preferences error:', error);
    throw error;
  }
}

/**
 * Update notification preferences
 */
export async function updateNotificationPreferences(preferences: Record<string, unknown>): Promise<void> {
  try {
    await requireAuth();

    const client = getClient();
    await client.put('/api/user/notification-preferences', preferences);
  } catch (error) {
    console.error('Update notification preferences error:', error);
    throw error;
  }
}

/**
 * Send notification (admin only)
 */
export async function sendNotification(notification: {
  recipientId: string;
  title: string;
  body: string;
  type?: string;
  priority?: string;
}): Promise<void> {
  try {
    await requirePermission('admin:notifications:send');
    
    const client = getClient();
    await client.post('/api/admin/notifications/send', notification);
  } catch (error) {
    console.error('Send notification error:', error);
    throw error;
  }
}

/**
 * Get notification stats (admin only)
 */
export async function getNotificationStats(): Promise<Record<string, unknown> | undefined> {
  try {
    await requirePermission('admin:notifications:read');

    const client = getClient();
    const result = await client.getNotificationStats();

    return result as Record<string, unknown> | undefined;
  } catch (error) {
    console.error('Get notification stats error:', error);
    throw error;
  }
}

/**
 * Subscribe to push notifications
 */
export async function subscribeToPushNotifications(subscription: {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}): Promise<void> {
  try {
    await requireAuth();
    
    const client = getClient();
    await client.subscribeToPushNotifications(subscription);
  } catch (error) {
    console.error('Subscribe to push notifications error:', error);
    throw error;
  }
}

/**
 * Unsubscribe from push notifications
 */
export async function unsubscribeFromPushNotifications(): Promise<void> {
  try {
    await requireAuth();
    
    const client = getClient();
    await client.unsubscribeFromPushNotifications();
  } catch (error) {
    console.error('Unsubscribe from push notifications error:', error);
    throw error;
  }
}