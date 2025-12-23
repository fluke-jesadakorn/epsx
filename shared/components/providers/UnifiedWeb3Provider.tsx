'use client';

// @ts-ignore
import {
    darkTheme,
    getDefaultConfig,
    lightTheme,
    RainbowKitProvider
} from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
// @ts-ignore
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTheme } from 'next-themes';
import React, { createContext, useContext, useEffect, useState } from 'react';
// @ts-ignore
import { bsc, bscTestnet } from 'viem/chains';
// @ts-ignore
import { WagmiProvider } from 'wagmi';

// Singleton Query Client with common settings
const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            retry: 1,
            staleTime: 60000,
            refetchOnWindowFocus: false,
        },
    },
});

export interface UnifiedWeb3ProviderProps {
    children: React.ReactNode;
    appName?: string;
    projectId?: string;
    chains?: any[];
    learnMoreUrl?: string;
    isAdminMode?: boolean;
}

const Web3Context = createContext({
    isInitialized: false,
    isAdminMode: false,
});

export const useUnifiedWeb3 = () => useContext(Web3Context);

export function UnifiedWeb3Provider({
    children,
    appName = 'EPSX',
    projectId = process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || '04e0a500abfa1e095bf8f64b15fa2812',
    chains = [bsc, bscTestnet],
    learnMoreUrl = 'https://epsx.io',
    isAdminMode = false,
}: UnifiedWeb3ProviderProps) {
    const [isHydrated, setIsHydrated] = useState(false);
    const { resolvedTheme } = useTheme();

    useEffect(() => {
        setIsHydrated(true);
    }, []);

    const config = getDefaultConfig({
        appName,
        projectId,
        chains: chains as any,
        ssr: true,
    });

    const customLightTheme = lightTheme({
        accentColor: '#f97316',
        accentColorForeground: 'white',
        borderRadius: 'medium',
    });

    const customDarkTheme = darkTheme({
        accentColor: isAdminMode ? '#fbbf24' : '#f97316',
        accentColorForeground: isAdminMode ? '#1f2937' : 'white',
        borderRadius: 'medium',
    });

    const theme = resolvedTheme === 'dark' ? customDarkTheme : customLightTheme;

    if (!isHydrated) {
        return (
            <Web3Context.Provider value={{ isInitialized: false, isAdminMode }}>
                <WagmiProvider config={config}>
                    <QueryClientProvider client={queryClient}>
                        {children}
                    </QueryClientProvider>
                </WagmiProvider>
            </Web3Context.Provider>
        );
    }

    return (
        <Web3Context.Provider value={{ isInitialized: true, isAdminMode }}>
            <WagmiProvider config={config}>
                <QueryClientProvider client={queryClient}>
                    <RainbowKitProvider
                        theme={theme}
                        modalSize="compact"
                        appInfo={{
                            appName,
                            learnMoreUrl,
                        }}
                    >
                        {children}
                    </RainbowKitProvider>
                </QueryClientProvider>
            </WagmiProvider>
        </Web3Context.Provider>
    );
}
