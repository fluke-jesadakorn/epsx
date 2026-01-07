// @ts-nocheck
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
import type { Chain } from 'wagmi/chains';

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

/**
 * Get the dynamic Anvil RPC URL based on the current browser hostname.
 * Supports Tailscale IPs (100.x.x.x) and other local network access.
 */
function getAnvilRpcUrl(): string {
    if (typeof window === 'undefined') {
        return 'http://127.0.0.1:8545';
    }
    const hostname = window.location.hostname;
    if (hostname === 'localhost' || hostname === '127.0.0.1') {
        return 'http://127.0.0.1:8545';
    }
    // For Tailscale or other network access, use the same hostname
    return `http://${hostname}:8545`;
}

// Define Anvil Localhost chain for development (configured as BSC-like)
// Note: rpcUrls are evaluated at runtime via getter for dynamic hostname support
const createAnvilLocalhost = (): Chain => {
    const rpcUrl = getAnvilRpcUrl();
    return {
        id: 31337,
        name: 'Anvil Local (BSC)',
        nativeCurrency: {
            decimals: 18,
            name: 'BNB',
            symbol: 'BNB',
        },
        rpcUrls: {
            default: { http: [rpcUrl] },
            public: { http: [rpcUrl] },
        },
        blockExplorers: {
            default: { name: 'Anvil', url: rpcUrl },
        },
        testnet: true,
    } as Chain;
};

// Lazy-init to ensure window is available
let anvilLocalhost: Chain | null = null;
const getAnvilLocalhost = (): Chain => {
    if (!anvilLocalhost) {
        anvilLocalhost = createAnvilLocalhost();
    }
    return anvilLocalhost;
};

// Get the blockchain network from environment
const isMainnet = process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK === 'mainnet';
const isProduction = process.env.NODE_ENV === 'production';
const defaultChainId = Number(process.env.NEXT_PUBLIC_CHAIN_ID);

// Determine default chains with environment awareness
// In production mainnet: only BSC Mainnet
// In production non-mainnet: BSC Testnet + BSC Mainnet
// In development: Include Anvil Local for local testing
// Note: Function-based to ensure Anvil chain is created with correct hostname at runtime
function getDefaultChains(): Chain[] {
    if (isMainnet && isProduction) {
        return [bsc];
    }
    if (isProduction) {
        return [bscTestnet, bsc];
    }
    // Development: include Anvil with dynamic RPC URL
    const anvil = getAnvilLocalhost();
    return defaultChainId === 31337
        ? [anvil, bscTestnet, bsc]  // Anvil first when it's the default
        : [bscTestnet, anvil, bsc]; // Include Anvil in dev for switching
}

const DEFAULT_APP_NAME = 'EPSX';
const DEFAULT_PROJECT_ID = process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || '04e0a500abfa1e095bf8f64b15fa2812';
const DEFAULT_LEARN_MORE_URL = 'https://epsx.io';

interface Web3ContextType {
    isInitialized: boolean;
    isAdminMode: boolean;
    forceReset: () => void;
    forceRecreateConnectors: () => void;
}

const Web3Context = createContext<Web3ContextType>({
    isInitialized: false,
    isAdminMode: false,
    forceReset: () => { if (typeof window !== 'undefined') window.location.reload(); },
    forceRecreateConnectors: () => { if (typeof window !== 'undefined') window.location.reload(); },
});

export const useUnifiedWeb3 = () => useContext(Web3Context);

export interface UnifiedWeb3ProviderProps {
    children: React.ReactNode;
    appName?: string;
    projectId?: string;
    chains?: any[];
    learnMoreUrl?: string;
    isAdminMode?: boolean;
}

export function UnifiedWeb3Provider({
    children,
    appName = DEFAULT_APP_NAME,
    projectId = DEFAULT_PROJECT_ID,
    chains,
    learnMoreUrl = DEFAULT_LEARN_MORE_URL,
    isAdminMode = false,
}: UnifiedWeb3ProviderProps) {
    // Use provided chains or get default chains (with dynamic Anvil RPC)
    const resolvedChains = chains ?? getDefaultChains();
    const [isHydrated, setIsHydrated] = useState(false);
    const { resolvedTheme } = useTheme();

    useEffect(() => {
        setIsHydrated(true);

        // Handle wallet library and database cleanup errors that are safe to ignore
        const handleWalletErrors = (event: PromiseRejectionEvent) => {
            const error = event.reason;
            if (error instanceof TypeError) {
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

    const config = React.useMemo(() => getDefaultConfig({
        appName,
        projectId,
        chains: resolvedChains as any,
        ssr: true,
    }), [appName, projectId, resolvedChains]);

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
        resolvedTheme === 'dark' ? customDarkTheme : customLightTheme,
        [resolvedTheme, customDarkTheme, customLightTheme]);

    const forceReset = React.useCallback(() => {
        if (typeof window !== 'undefined') window.location.reload();
    }, []);

    const forceRecreateConnectors = React.useCallback(() => {
        if (typeof window !== 'undefined') window.location.reload();
    }, []);

    const contextValue = React.useMemo(() => ({
        isInitialized: isHydrated,
        isAdminMode,
        forceReset,
        forceRecreateConnectors
    }), [isHydrated, isAdminMode, forceReset, forceRecreateConnectors]);

    if (!isHydrated) {
        return (
            <Web3Context.Provider value={contextValue}>
                <WagmiProvider config={config}>
                    <QueryClientProvider client={queryClient}>
                        {children}
                    </QueryClientProvider>
                </WagmiProvider>
            </Web3Context.Provider>
        );
    }

    return (
        <Web3Context.Provider value={contextValue}>
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
