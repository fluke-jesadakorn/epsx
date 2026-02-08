'use client';

import { useTheme } from 'next-themes';
import React, { createContext, useCallback, useContext, useEffect, useState } from 'react';

import { settingsApi } from '@/lib/api/settings-client';
import type { SystemSettings } from '@/types/settings';

interface SettingsContextType {
    settings: SystemSettings | null;
    isLoading: boolean;
    applySettings: (settings: SystemSettings) => void;
    refreshSettings: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

/**
 * Apply accent color as CSS custom property on document root
 * @param color
 */
function applyAccentColor(color: string) {
    if (typeof document !== 'undefined') {
        document.documentElement.style.setProperty('--accent-color', color);
        // Also set RGB values for transparency support
        const hex = color.replace('#', '');
        const r = parseInt(hex.substring(0, 2), 16);
        const g = parseInt(hex.substring(2, 4), 16);
        const b = parseInt(hex.substring(4, 6), 16);
        document.documentElement.style.setProperty('--accent-color-rgb', `${r}, ${g}, ${b}`);
    }
}

/**
 * SettingsProvider - Fetches and applies system settings across the app
 * 
 * Features:
 * - Fetches settings on mount
 * - Applies theme mode via next-themes
 * - Applies accent color as CSS variable
 * - Provides context for components to access settings
 * @param root0
 * @param root0.children
 */
export function SettingsProvider({ children }: { children: React.ReactNode }) {
    const [settings, setSettings] = useState<SystemSettings | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const { setTheme } = useTheme();

    const applySettings = useCallback((newSettings: SystemSettings) => {
        // Apply theme mode
        // NOTE: Commented out to prevent conflict with next-themes user selection
        // const themeMode = newSettings.appearance?.theme || 'dark';
        // setTheme(themeMode === 'auto' ? 'system' : themeMode);

        // Apply accent color
        const accentColor = newSettings.appearance?.primaryColor || '#00ff33';
        applyAccentColor(accentColor);

        setSettings(newSettings);
    }, [setTheme]);

    const refreshSettings = useCallback(async () => {
        try {
            setIsLoading(true);
            const fetchedSettings = await settingsApi.getAll();
            applySettings(fetchedSettings);
        } catch (error) {
            console.error('Failed to fetch settings:', error);
            // Apply defaults on error
            applyAccentColor('#00ff33');
        } finally {
            setIsLoading(false);
        }
    }, [applySettings]);

    // Fetch and apply settings on mount
    useEffect(() => {
        refreshSettings();
    }, [refreshSettings]);

    return (
        <SettingsContext.Provider value={{ settings, isLoading, applySettings, refreshSettings }}>
            {children}
        </SettingsContext.Provider>
    );
}

/**
 * Hook to access settings context
 */
export function useSettings() {
    const context = useContext(SettingsContext);
    if (context === undefined) {
        throw new Error('useSettings must be used within a settings-provider');
    }
    return context;
}

export default SettingsProvider;
