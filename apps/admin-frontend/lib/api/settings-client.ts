/**
 * ADMIN SETTINGS CLIENT
 *
 * Re-exports from shared/api/settings for backward compatibility.
 * This file is deprecated - import directly from @/shared/api instead.
 */

'use client';

// Re-export everything from shared settings module
export {
    createSettingsClient,
    DEFAULT_SETTINGS, SettingsApi, type AppearanceSettings, type GeneralSettings,
    type NotificationSettings,
    type SecuritySettings, type SettingUpdate, type SystemSettings, type UpdateSettingsResponse
} from '@/shared/api/settings';

// Create singleton instance for backward compatibility
import { createSettingsClient, type SettingsApi as SettingsApiType } from '@/shared/api/settings';

let settingsApiInstance: SettingsApiType | null = null;

function getSettingsApiInstance(): SettingsApiType {
    if (!settingsApiInstance) {
        settingsApiInstance = createSettingsClient({
            serverSide: typeof window === 'undefined',
        });
    }
    return settingsApiInstance;
}

/**
 * Settings API singleton for backward compatibility
 * @deprecated Use createSettingsClient() from @/shared/api instead
 */
export const settingsApi = {
    async getAll() {
        return getSettingsApiInstance().getAll();
    },
    async getByCategory(category: string) {
        return getSettingsApiInstance().getByCategory(category);
    },
    async update(settings: Parameters<SettingsApiType['update']>[0]) {
        return getSettingsApiInstance().update(settings);
    },
    async reset() {
        return getSettingsApiInstance().reset();
    },
};

export default settingsApi;
