// @ts-nocheck
'use client';

// @ts-ignore
import {
    darkTheme,
    lightTheme,
    RainbowKitProvider
} from '@rainbow-me/rainbowkit';
import '@rainbow-me/rainbowkit/styles.css';
// @ts-ignore
import { QueryClientProvider } from '@tanstack/react-query';
import { useTheme } from 'next-themes';
import React, { createContext, useContext, useEffect, useState } from 'react';
// @ts-ignore
import { useRouter } from 'next/navigation';
import type { Chain, State } from 'wagmi';
import { useAccount, useReconnect, WagmiProvider } from 'wagmi';
import { DEFAULT_APP_NAME, DEFAULT_LEARN_MORE_URL, DEFAULT_PROJECT_ID, getConfig, getDefaultChains } from '../../config/wagmi';
import { createQueryClient } from '../../state';

/**
 * Component that triggers wagmi reconnection on mount
 * This ensures wallet state is restored from cookie storage after page refresh
 * Reconnection happens silently in background without blocking UI
 */
function WagmiReconnectProvider({ children }: { children: React.ReactNode }) {
    const { reconnect } = useReconnect();
    const { isConnected } = useAccount();
    const [hasAttemptedReconnect, setHasAttemptedReconnect] = React.useState(false);

    React.useEffect(() => {
        if (!hasAttemptedReconnect && !isConnected) {
            setHasAttemptedReconnect(true);
            // Let reconnect happen silently in background
            reconnect();
        }
    }, [reconnect, isConnected, hasAttemptedReconnect]);

    return <>{children}</>;
}

interface Web3ContextType {
    isInitialized: boolean;
    isAdminMode: boolean;
    forceReset: () => void;
    forceRecreateConnectors: () => void;
}

const Web3Context = createContext<Web3ContextType>({
    isInitialized: false,
    isAdminMode: false,
    forceReset: () => { },
    forceRecreateConnectors: () => { },
});

export const useUnifiedWeb3 = () => useContext(Web3Context);

export interface UnifiedWeb3ProviderProps {
    children: React.ReactNode;
    appName?: string;
    projectId?: string;
    chains?: Chain[];
    learnMoreUrl?: string;
    isAdminMode?: boolean;
    initialState?: State;
}

/**
 * Inner provider component that renders after hydration
 * This separation ensures RainbowKit is only mounted on the client after hydration
 */
function RainbowKitWrapper({
    children,
    appName,
    learnMoreUrl,
    isAdminMode,
}: {
    children: React.ReactNode;
    appName: string;
    learnMoreUrl: string;
    isAdminMode: boolean;
}) {
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

    // Don't render RainbowKit until after client-side mount
    // This prevents the ConnectModal setState during Hydrate error
    if (!isMounted) {
        return <>{children}</>;
    }

    return (
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
    );
}

export function UnifiedWeb3Provider({
    children,
    appName = DEFAULT_APP_NAME,
    projectId = DEFAULT_PROJECT_ID,
    chains,
    learnMoreUrl = DEFAULT_LEARN_MORE_URL,
    isAdminMode = false,
    initialState,
}: UnifiedWeb3ProviderProps) {
    // Use provided chains or get default chains (with dynamic Anvil RPC)
    const resolvedChains = chains ?? getDefaultChains();
    const [isHydrated, setIsHydrated] = useState(false);
    const router = useRouter();
    const [queryClient] = useState(() => createQueryClient(isAdminMode ? 'admin' : 'frontend'));

    useEffect(() => {
        setIsHydrated(true);

        // Handle wallet library and database cleanup errors that are safe to ignore
        const handleWalletErrors = (event: PromiseRejectionEvent) => {
            const error = event.reason as unknown;
            if (error instanceof Error) {
                const msg = error.message;
                // These specific errors are from wallet library and database cleanup and are harmless
                // Also includes localStorage SSR errors from @walletconnect/keyvaluestorage
                if (
                    msg.includes("Cannot set properties of null") ||
                    msg.includes("Cannot read properties of null") ||
                    msg.includes("Cannot mix BigInt") ||
                    msg.includes("IndexedDB") ||
                    msg.includes("WebSocket") ||
                    msg.includes("transaction") ||
                    msg.includes("onclose") ||
                    msg.includes("localStorage") ||
                    msg.includes("is not a function")
                ) {
                    event.preventDefault(); // Prevent console spam
                    return;
                }
            }
        };

        if (typeof window !== 'undefined') {
            window.addEventListener('unhandledrejection', handleWalletErrors);
            return () => window.removeEventListener('unhandledrejection', handleWalletErrors);
        }
    }, []);

    const config = React.useMemo(() => getConfig({
        appName,
        projectId,
        chains: resolvedChains,
        ssr: true,
    }), [appName, projectId, resolvedChains]);

    const forceReset = React.useCallback(() => {
        router.refresh();
    }, [router]);

    const forceRecreateConnectors = React.useCallback(() => {
        router.refresh();
    }, [router]);

    const contextValue = React.useMemo(() => ({
        isInitialized: isHydrated,
        isAdminMode,
        forceReset,
        forceRecreateConnectors
    }), [isHydrated, isAdminMode, forceReset, forceRecreateConnectors]);

    return (
        <Web3Context.Provider value={contextValue}>
            <WagmiProvider config={config} initialState={initialState}>
                <QueryClientProvider client={queryClient}>
                    <WagmiReconnectProvider>
                        <RainbowKitWrapper
                            appName={appName}
                            learnMoreUrl={learnMoreUrl}
                            isAdminMode={isAdminMode}
                        >
                            {children}
                        </RainbowKitWrapper>
                    </WagmiReconnectProvider>
                </QueryClientProvider>
            </WagmiProvider>
        </Web3Context.Provider>
    );
}

