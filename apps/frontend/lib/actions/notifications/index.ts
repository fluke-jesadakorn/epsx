'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { requireAuth, requirePermission } from '../auth';
import { safeError } from '@/lib/utils/logging';

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
  customData?: Record<string, any>;
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
    const notifications = result.data.map(notification => ({
      id: notification.id,
      title: notification.title,
      body: notification.message,
      type: notification.type,
      priority: notification.priority,
      createdAt: notification.createdAt,
      readAt: notification.readAt,
      actionUrl: notification.actionUrl,
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
export async function getNotificationPreferences(): Promise<any> {
  try {
    await requireAuth();
    
    const client = getClient();
    const result = await client.get('/api/v1/user/notification-preferences');
    
    return result.data;
  } catch (error) {
    console.error('Get notification preferences error:', error);
    throw error;
  }
}

/**
 * Update notification preferences
 */
export async function updateNotificationPreferences(preferences: any): Promise<void> {
  try {
    await requireAuth();
    
    const client = getClient();
    await client.put('/api/v1/user/notification-preferences', preferences);
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
    await client.post('/api/v1/admin/notifications/send', notification);
  } catch (error) {
    console.error('Send notification error:', error);
    throw error;
  }
}

/**
 * Get notification stats (admin only)
 */
export async function getNotificationStats(): Promise<any> {
  try {
    await requirePermission('admin:notifications:read');
    
    const client = getClient();
    const result = await client.getNotificationStats();
    
    return result;
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