'use server';

import type { NotificationsResponse } from '@/shared/api/notifications';
import { createNotificationsClient } from '@/shared/api/notifications';
import { logger } from '@/shared/utils/logger';
import { getServerActionClient } from '@/shared/utils/server-fetch';

/**
 * Get initial notifications for server-side rendering
 */
export async function getInitialNotificationsAction(filters: {
  page: number;
  limit?: number;
  status?: string;
  type?: string;
  priority?: string;
}): Promise<NotificationsResponse> {
  const client = getServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.getNotifications({
      page: filters.page,
      limit: filters.limit ?? 20,
      status: filters.status as 'read' | 'unread' | 'all' | undefined,
      type: filters.type as any,
      priority: filters.priority as any,
    });
  } catch (error) {
    logger.warn('Failed to fetch notifications:', error);
    return {
      success: false,
      data: {
        notifications: [],
        total_count: 0,
        unread_count: 0,
        page: 1,
        limit: filters.limit ?? 20,
        total_pages: 0,
      },
      error: {
        code: 'FETCH_FAILED',
        message: error instanceof Error ? error.message : 'Unknown error occurred',
      },
    };
  }
}

/**
 * Mark notification as read
 */
export async function markAsReadAction(notificationId: string): Promise<{ success: boolean; message: string }> {
  const client = getServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.markAsRead(notificationId);
  } catch (error) {
    logger.error('Failed to mark notification as read:', error);
    return { success: false, message: 'Failed to mark as read' };
  }
}

/**
 * Mark all notifications as read
 */
export async function markAllAsReadAction(): Promise<{ success: boolean; updated_count: number }> {
  const client = getServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.markAllAsRead();
  } catch (error) {
    logger.error('Failed to mark all notifications as read:', error);
    return { success: false, updated_count: 0 };
  }
}

/**
 * Delete notification
 */
export async function deleteNotificationAction(notificationId: string): Promise<{ success: boolean; message: string }> {
  const client = getServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.deleteNotification(notificationId);
  } catch (error) {
    logger.error('Failed to delete notification:', error);
    return { success: false, message: 'Failed to delete notification' };
  }
}

/**
 * Clear all notifications
 */
export async function clearAllNotificationsAction(): Promise<{ success: boolean; deleted_count: number }> {
  const client = getServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.clearAllNotifications();
  } catch (error) {
    logger.error('Failed to clear all notifications:', error);
    return { success: false, deleted_count: 0 };
  }
}
