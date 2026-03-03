'use client';

import { darkTheme, lightTheme, RainbowKitProvider } from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
import { useTheme } from 'next-themes';
import React, { useEffect, useState } from 'react';

export interface RainbowKitWrapperProps {
    children: React.ReactNode;
    appName: string;
    learnMoreUrl: string;
    isAdminMode: boolean;
}

export function RainbowKitWrapper({ children, appName, learnMoreUrl, isAdminMode }: RainbowKitWrapperProps) {
    const { resolvedTheme } = useTheme();
    const [isMounted, setIsMounted] = useState(false);

    useEffect(() => {
        setIsMounted(true);
    }, []);

    const customLightTheme = React.useMemo(() => lightTheme({
        accentColor: '#f97316',
        accentColorForeground: 'white',
        borderRadius: 'medium',
    }), []);

    const customDarkTheme = React.useMemo(() => darkTheme({
        accentColor: isAdminMode ? '#fbbf24' : '#f97316',
        accentColorForeground: isAdminMode ? '#1f2937' : 'white',
        borderRadius: 'medium',
    }), [isAdminMode]);

    const theme = React.useMemo(() =>
        isMounted && resolvedTheme === 'dark' ? customDarkTheme : customLightTheme,
        [isMounted, resolvedTheme, customDarkTheme, customLightTheme]);

    if (!isMounted) {
        return <>{children}</>;
    }

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
