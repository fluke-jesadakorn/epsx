/**
 * UNIFIED SETTINGS API CLIENT
 *
 * System settings management endpoints.
 * Migrated from admin-frontend/lib/api/settings-client.ts
 *
 * Features:
 * - Global system settings management
 * - Category-based settings
 * - Settings update and reset
 */

import type { UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

import { createAdminApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface GeneralSettings {
    systemName: string;
    adminEmail: string;
    maintenanceMode: boolean;
}

export interface NotificationSettings {
    emailNotifications: boolean;
    pushNotifications: boolean;
    smsNotifications: boolean;
    securityAlerts: boolean;
}

export interface SecuritySettings {
    sessionTimeout: number;
}

export interface AppearanceSettings {
    theme: 'light' | 'dark' | 'auto';
    primaryColor: string;
}

export interface SystemSettings {
    general: GeneralSettings;
    notifications: NotificationSettings;
    security: SecuritySettings;
    appearance: AppearanceSettings;
}

export interface SettingUpdate {
    category: string;
    key: string;
    value: unknown;
}

export interface UpdateSettingsResponse {
    success: boolean;
    message: string;
    updated_count: number;
    errors?: string[];
}

// Default settings values
export const DEFAULT_SETTINGS: SystemSettings = {
    general: {
        systemName: 'EPSX Admin',
        adminEmail: 'admin@epsx.io',
        maintenanceMode: false,
    },
    notifications: {
        emailNotifications: true,
        pushNotifications: false,
        smsNotifications: true,
        securityAlerts: true,
    },
    security: {
        sessionTimeout: 30,
    },
    appearance: {
        theme: 'auto',
        primaryColor: '#3b82f6',
    },
};

// ============================================================================
// HELPER FUNCTIONS
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
// SETTINGS API CLASS
// ============================================================================

export class SettingsApi {
    private client: UnifiedApiClient;

    constructor(client: UnifiedApiClient) {
        this.client = client;
    }

    /**
     * Fetch all system settings
     */
    async getAll(): Promise<SystemSettings> {
        try {
            const res = await this.client.get<{ data?: unknown } | unknown>('/api/admin/settings');
            const responseData = res.data as { data?: unknown } | undefined;
            const data = responseData && typeof responseData === 'object' && 'data' in responseData
                ? responseData.data
                : res.data;
            return parseSettingsData(data);
        } catch (error) {
            console.error('Failed to fetch settings:', error);
            return DEFAULT_SETTINGS;
        }
    }

    /**
     * Fetch settings for a specific category
     */
    async getByCategory(category: string): Promise<Record<string, unknown>> {
        try {
            const res = await this.client.get<{ data?: { settings?: Record<string, unknown> } }>(
                `/api/admin/settings/${category}`
            );
            return res.data?.data?.settings || {};
        } catch (error) {
            console.error(`Failed to fetch ${category} settings:`, error);
            return {};
        }
    }

    /**
     * Update multiple settings at once
     */
    async update(settings: SystemSettings): Promise<UpdateSettingsResponse> {
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
            const res = await this.client.put<UpdateSettingsResponse>('/api/admin/settings', {
                settings: updates,
            });
            return res.data || { success: false, message: 'Unknown error', updated_count: 0 };
        } catch (error) {
            console.error('Failed to update settings:', error);
            return { success: false, message: 'Failed to update settings', updated_count: 0 };
        }
    }

    /**
     * Reset all settings to defaults
     */
    async reset(): Promise<SystemSettings> {
        try {
            const res = await this.client.post<{ data?: unknown }>('/api/admin/settings/reset', {});
            const data = res.data?.data || {};
            return parseSettingsData(data);
        } catch (error) {
            console.error('Failed to reset settings:', error);
            return DEFAULT_SETTINGS;
        }
    }
}

/**
 * Create a settings API client
 */
export function createSettingsClient(options?: {
    baseURL?: string;
    token?: string;
    serverSide?: boolean;
}): SettingsApi {
    const client = createAdminApiClient(options);
    return new SettingsApi(client);
}
