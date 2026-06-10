'use server';

import type { NotificationsResponse } from '@/shared/api/notifications';
import { createNotificationsClient } from '@/shared/api/notifications';
import { logger } from '@/lib/logger';
import { getAdminServerActionClient } from '@/shared/utils/server-fetch';
import type { ApiResponse } from '@/shared/utils/api-client';

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
    logger.action.error('getAdminNotifications', error);
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

export async function markAsUnreadAction(notificationId: string): Promise<{ success: boolean; message: string }> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.markAsUnread(notificationId);
  } catch (error) {
    logger.action.error('markAsUnread', error);
    return { success: false, message: 'Failed to mark as unread' };
  }
}

export async function markAllAsReadAction(): Promise<{ success: boolean; updated_count: number }> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.markAllAsRead();
  } catch (error) {
    logger.action.error('markAllAsRead', error);
    return { success: false, updated_count: 0 };
  }
}

export async function deleteAdminNotificationAction(notificationId: string): Promise<{ success: boolean; message: string }> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.deleteAdminNotification(notificationId);
  } catch (error) {
    logger.action.error('deleteAdminNotification', error);
    return { success: false, message: 'Failed to delete notification' };
  }
}

export async function markAsReadAction(notificationId: string): Promise<{ success: boolean }> {
  const client = getAdminServerActionClient();
  const notifications = createNotificationsClient(client);

  try {
    return await notifications.markAsRead(notificationId);
  } catch (error) {
    logger.action.error('markAsRead', error);
    return { success: false };
  }
}

export async function uploadNotificationImageAction(
  formData: FormData
): Promise<ApiResponse<{ url: string; filename: string }>> {
  const client = getAdminServerActionClient();
  try {
    return await client.post('/api/admin/notifications/upload-image', formData);
  } catch (error) {
    logger.action.error('uploadNotificationImage', error);
    return { success: false, data: null as unknown as { url: string; filename: string }, error: { code: 'UPLOAD_FAILED', message: error instanceof Error ? error.message : 'Upload failed' } };
  }
}
