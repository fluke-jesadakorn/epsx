'use client';

import { darkTheme, lightTheme, RainbowKitProvider } from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
import { useTheme } from 'next-themes';
import React from 'react';

export interface RainbowKitWrapperProps {
    children: React.ReactNode;
    appName: string;
    learnMoreUrl: string;
    isAdminMode: boolean;
}

export function RainbowKitWrapper({ children, appName, learnMoreUrl, isAdminMode }: RainbowKitWrapperProps) {
    const { resolvedTheme } = useTheme();

    const customLightTheme = React.useMemo(() => lightTheme({
        accentColor: '#f97316',
        accentColorForeground: 'white',
        borderRadius: 'medium',
    }), []);

    const customDarkTheme = React.useMemo(() => {
        const t = darkTheme({
            accentColor: isAdminMode ? '#fbbf24' : '#f97316',
            accentColorForeground: isAdminMode ? '#1f2937' : 'white',
            borderRadius: 'medium',
        });
        t.colors.modalBackground = '#1a1b23';
        t.colors.downloadTopCardBackground = '#1a1b23';
        t.colors.downloadBottomCardBackground = '#1a1b23';
        t.colors.profileForeground = '#1a1b23';
        return t;
    }, [isAdminMode]);

    const theme = React.useMemo(() =>
        resolvedTheme === 'dark' ? customDarkTheme : customLightTheme,
        [resolvedTheme, customDarkTheme, customLightTheme]);

    return (
        <RainbowKitProvider
            theme={theme}
            modalSize="compact"
            appInfo={{ appName, learnMoreUrl }}
        >
            {children}
        </RainbowKitProvider>
    );
}
