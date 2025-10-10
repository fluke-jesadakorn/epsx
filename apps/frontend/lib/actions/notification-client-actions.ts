'use client';

import { createNotificationsClient } from '@/shared/api/notifications';
import { createFrontendApiClient } from '@/shared/utils/api-client';

/**
 * Client-side notification actions
 * These are used by client components and use the shared API client
 */

/**
 * Mark a notification as read
 */
export async function markNotificationRead(notificationId: string): Promise<void> {
  const client = createNotificationsClient(createFrontendApiClient());
  await client.markAsRead(notificationId);
}

/**
 * Mark all notifications as read
 */
export async function markAllNotificationsRead(): Promise<void> {
  const client = createNotificationsClient(createFrontendApiClient());
  await client.markAllAsRead();
}

/**
 * Delete a notification
 */
export async function deleteNotification(notificationId: string): Promise<void> {
  const client = createNotificationsClient(createFrontendApiClient());
  await client.deleteNotification(notificationId);
}

/**
 * Clear all notifications
 */
export async function clearAllNotifications(): Promise<void> {
  const client = createNotificationsClient(createFrontendApiClient());
  await client.clearAllNotifications();
}
