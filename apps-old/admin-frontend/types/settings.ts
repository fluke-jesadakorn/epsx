/**
 * System Settings Types
 * Type definitions for admin console settings
 */

export interface SystemSettings {
    general: GeneralSettings;
    notifications: NotificationSettings;
    security: SecuritySettings;
    appearance: AppearanceSettings;
}

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

export interface SettingUpdate {
    category: string;
    key: string;
    value: unknown;
}

// Default settings values
export const DEFAULT_SETTINGS: SystemSettings = {
    general: {
        systemName: 'EPSX Admin Console',
        adminEmail: 'admin@epsx.com',
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
        theme: 'light',
        primaryColor: '#FF8C00',
    },
};
