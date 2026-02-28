'use server';

import { rethrowRedirect } from '@/lib/api-error';
import { createNotificationsClient } from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';

export async function getNotificationsAction(page = 1, limit = 20) {
    try {
        const client = createNotificationsClient(createAdminApiClient({ serverSide: true }));
        const response = await client.getAllNotifications({ page, limit });
        return { success: true, data: response.data };
    } catch (e) {
        rethrowRedirect(e);
        return { success: false, error: 'Failed to fetch notifications' };
    }
}

export async function getNotificationStatsAction() {
    try {
        const client = createNotificationsClient(createAdminApiClient({ serverSide: true }));
        const response = await client.getNotificationStats();
        return { success: true, data: response.data };
    } catch (e) {
        rethrowRedirect(e);
        return { success: false, error: 'Failed to fetch notification stats' };
    }
}

export async function getNotificationOverviewAction(limit = 20) {
    try {
        const apiClient = createAdminApiClient({ serverSide: true });
        const res = await apiClient.get<{ notifications: unknown[]; stats: unknown }>(`/api/admin/notifications/overview?limit=${String(limit)}`);
        return { success: true, data: res.data ?? { notifications: [], stats: {} } };
    } catch (e) {
        rethrowRedirect(e);
        return { success: false, data: { notifications: [], stats: {} } };
    }
}

export async function deleteNotificationAction(id: string) {
    try {
        const client = createNotificationsClient(createAdminApiClient({ serverSide: true }));
        const response = await client.deleteAdminNotification(id);
        return { success: response.success, message: response.message };
    } catch (e) {
        rethrowRedirect(e);
        return { success: false, error: 'Failed to delete notification' };
    }
}
