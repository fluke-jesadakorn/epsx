'use server';

import type { NotificationsResponse } from '@/shared/api/notifications';
import { createNotificationsClient } from '@/shared/api/notifications';
import { logger } from '@/shared/utils/logger';
import { getAdminServerActionClient } from '@/shared/utils/server-fetch';

export async function getAdminNotificationsAction(filters: {
  page: number;
  limit?: number;
  status?: string;
}): Promise<NotificationsResponse> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.getAllNotifications({
      page: filters.page,
      limit: filters.limit ?? 20,
      status: filters.status as 'read' | 'unread' | 'all' | undefined,
    });
  } catch (error) {
    logger.warn('Failed to fetch admin notifications:', error);
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

export async function markAllAsReadAction(): Promise<{ success: boolean; updated_count: number }> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.markAllAsRead();
  } catch (error) {
    logger.error('Failed to mark all notifications as read:', error);
    return { success: false, updated_count: 0 };
  }
}

export async function deleteAdminNotificationAction(notificationId: string): Promise<{ success: boolean; message: string }> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.deleteAdminNotification(notificationId);
  } catch (error) {
    logger.error('Failed to delete admin notification:', error);
    return { success: false, message: 'Failed to delete notification' };
  }
}
