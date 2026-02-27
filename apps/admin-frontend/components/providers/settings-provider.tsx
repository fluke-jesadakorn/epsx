'use client';

import { usePathname } from 'next/navigation';
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
        const r = parseInt(hex.slice(0, 2), 16);
        const g = parseInt(hex.slice(2, 4), 16);
        const b = parseInt(hex.slice(4, 6), 16);
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

    const applySettings = useCallback((newSettings: SystemSettings) => {
        // Apply accent color
        const accentColor = newSettings.appearance.primaryColor;
        applyAccentColor(accentColor);

        setSettings(newSettings);
    }, []);

    const refreshSettings = useCallback(async () => {
        try {
            setIsLoading(true);
            const fetchedSettings = await settingsApi.getAll();
            applySettings(fetchedSettings);
        } catch {
            // Apply defaults on error
            applyAccentColor('#00ff33');
        } finally {
            setIsLoading(false);
        }
    }, [applySettings]);

    const pathname = usePathname();

    // Fetch and apply settings on mount, skip on auth/login pages
    useEffect(() => {
        if (pathname.startsWith('/auth') || pathname === '/login') {
            setIsLoading(false);
        } else {
            void refreshSettings();
        }
    }, [pathname, refreshSettings]);

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
