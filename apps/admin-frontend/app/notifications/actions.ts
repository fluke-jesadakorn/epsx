'use server';

import { createNotificationsClient } from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';

export async function getNotificationsAction(page = 1, limit = 20) {
    try {
        const client = createNotificationsClient(createAdminApiClient({ serverSide: true }));
        const response = await client.getAllNotifications({ page, limit });
        return { success: true, data: response.data };
    } catch (error) {
        console.error('Failed to fetch notifications:', error);
        return { success: false, error: 'Failed to fetch notifications' };
    }
}

export async function getNotificationStatsAction() {
    try {
        const client = createNotificationsClient(createAdminApiClient({ serverSide: true }));
        const response = await client.getNotificationStats();
        return { success: true, data: response.data };
    } catch (error) {
        console.error('Failed to fetch notification stats:', error);
        return { success: false, error: 'Failed to fetch notification stats' };
    }
}

export async function deleteNotificationAction(id: string) {
    try {
        const client = createNotificationsClient(createAdminApiClient({ serverSide: true }));
        const response = await client.deleteAdminNotification(id);
        return { success: response.success, message: response.message };
    } catch (error) {
        console.error('Failed to delete notification:', error);
        return { success: false, error: 'Failed to delete notification' };
    }
}
