/**
 * System Settings API Client
 * API functions for managing global admin console settings
 */

'use client';

import type { SettingUpdate, SystemSettings } from '@/types/settings';
import { DEFAULT_SETTINGS } from '@/types/settings';
import { adminApiClient } from '../api-client';

// ============================================================================
// TYPES
// ============================================================================

interface RawSettingsData {
    general?: {
        systemName?: unknown;
        adminEmail?: unknown;
        maintenanceMode?: unknown;
    };
    notifications?: {
        emailNotifications?: unknown;
        pushNotifications?: unknown;
        smsNotifications?: unknown;
        securityAlerts?: unknown;
    };
    security?: {
        sessionTimeout?: unknown;
    };
    appearance?: {
        theme?: unknown;
        primaryColor?: unknown;
    };
}

export interface UpdateSettingsResponse {
    success: boolean;
    message: string;
    updated_count: number;
    errors?: string[];
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function parseSettingsData(data: unknown): SystemSettings {
    const raw = (data || {}) as RawSettingsData;

    return {
        general: {
            systemName: String(raw.general?.systemName || DEFAULT_SETTINGS.general.systemName),
            adminEmail: String(raw.general?.adminEmail || DEFAULT_SETTINGS.general.adminEmail),
            maintenanceMode: Boolean(raw.general?.maintenanceMode),
        },
        notifications: {
            emailNotifications: raw.notifications?.emailNotifications !== false,
            pushNotifications: Boolean(raw.notifications?.pushNotifications),
            smsNotifications: raw.notifications?.smsNotifications !== false,
            securityAlerts: raw.notifications?.securityAlerts !== false,
        },
        security: {
            sessionTimeout: Number(raw.security?.sessionTimeout) || DEFAULT_SETTINGS.security.sessionTimeout,
        },
        appearance: {
            theme: (['light', 'dark', 'auto'].includes(String(raw.appearance?.theme))
                ? String(raw.appearance?.theme)
                : DEFAULT_SETTINGS.appearance.theme) as 'light' | 'dark' | 'auto',
            primaryColor: String(raw.appearance?.primaryColor || DEFAULT_SETTINGS.appearance.primaryColor),
        },
    };
}

// ============================================================================
// SETTINGS API
// ============================================================================

export const settingsApi = {
    /**
     * Fetch all system settings
     */
    async getAll(): Promise<SystemSettings> {
        try {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const res = await adminApiClient.get<any>('/api/admin/settings');
            const data = res.data?.data || res.data;
            return parseSettingsData(data);
        } catch (error) {
            console.error('Failed to fetch settings:', error);
            return DEFAULT_SETTINGS;
        }
    },

    /**
     * Fetch settings for a specific category
     */
    async getByCategory(category: string): Promise<Record<string, unknown>> {
        try {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const res = await adminApiClient.get<any>(`/api/admin/settings/${category}`);
            return res.data?.data?.settings || {};
        } catch (error) {
            console.error(`Failed to fetch ${category} settings:`, error);
            return {};
        }
    },

    /**
     * Update multiple settings at once
     */
    async update(settings: SystemSettings): Promise<UpdateSettingsResponse> {
        // Convert frontend format to API format
        const updates: SettingUpdate[] = [];

        // General settings
        updates.push(
            { category: 'general', key: 'systemName', value: settings.general.systemName },
            { category: 'general', key: 'adminEmail', value: settings.general.adminEmail },
            { category: 'general', key: 'maintenanceMode', value: settings.general.maintenanceMode }
        );

        // Notification settings
        Object.entries(settings.notifications).forEach(([key, value]) => {
            updates.push({ category: 'notifications', key, value });
        });

        // Security settings
        updates.push({ category: 'security', key: 'sessionTimeout', value: settings.security.sessionTimeout });

        // Appearance settings
        updates.push(
            { category: 'appearance', key: 'theme', value: settings.appearance.theme },
            { category: 'appearance', key: 'primaryColor', value: settings.appearance.primaryColor }
        );

        try {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const res = await adminApiClient.put<any>('/api/admin/settings', {
                settings: updates,
            });

            return res.data || { success: false, message: 'Unknown error', updated_count: 0 };
        } catch (error) {
            console.error('Failed to update settings:', error);
            return { success: false, message: 'Failed to update settings', updated_count: 0 };
        }
    },

    /**
     * Reset all settings to defaults
     */
    async reset(): Promise<SystemSettings> {
        try {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const res = await adminApiClient.post<any>('/api/admin/settings/reset', {});
            const data = res.data?.data || {};
            return parseSettingsData(data);
        } catch (error) {
            console.error('Failed to reset settings:', error);
            return DEFAULT_SETTINGS;
        }
    },
};

export default settingsApi;
