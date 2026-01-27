import { getDefaultConfig } from '@rainbow-me/rainbowkit';
import { bsc, bscTestnet } from 'viem/chains';
import { cookieStorage, createConfig, createStorage, http, type Chain } from 'wagmi';

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
// Use a cached chain instance to maintain connector references across renders
let anvilLocalhost: Chain | null = null;
const getAnvilLocalhost = (): Chain => {
    if (!anvilLocalhost) {
        anvilLocalhost = createAnvilLocalhost();
    }
    return anvilLocalhost;
};

// Cache chains array to maintain connector references
let cachedChains: Chain[] | null = null;

// Get the blockchain network from environment
const isMainnet = process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK === 'mainnet';
const isProduction = process.env.NODE_ENV === 'production';
const defaultChainId = Number(process.env.NEXT_PUBLIC_CHAIN_ID);

// Determine default chains with environment awareness
// In production mainnet: only BSC Mainnet
// In production non-mainnet: BSC Testnet + BSC Mainnet
// In development: Include Anvil Local for local testing
// Note: Function-based to ensure Anvil chain is created with correct hostname at runtime
export function getDefaultChains(): Chain[] {
    // Return cached chains if available to maintain connector references
    if (cachedChains) return cachedChains;

    if (isMainnet && isProduction) {
        cachedChains = [bsc];
    } else if (isProduction) {
        cachedChains = [bscTestnet, bsc];
    } else {
        // Development: include Anvil with dynamic RPC URL
        const anvil = getAnvilLocalhost();
        cachedChains = defaultChainId === 31337
            ? [anvil, bscTestnet, bsc] // Anvil first when it's the default
            : [bscTestnet, anvil, bsc]; // Include Anvil in dev for switching
    }
    return cachedChains;
}

export const DEFAULT_APP_NAME = 'EPSX';
export const DEFAULT_PROJECT_ID = process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || '04e0a500abfa1e095bf8f64b15fa2812';
export const DEFAULT_LEARN_MORE_URL = 'https://epsx.io';

export interface WagmiConfigOptions {
    appName?: string;
    projectId?: string;
    chains?: Chain[];
    ssr?: boolean;
}

/**
 * Client-Side Configuration (Uses RainbowKit's getDefaultConfig)
 * WARNING: Do not call this from Server Components!
 */
export function getConfig({
    appName = DEFAULT_APP_NAME,
    projectId = DEFAULT_PROJECT_ID,
    chains,
    ssr = true,
}: WagmiConfigOptions = {}) {
    const resolvedChains = chains ?? getDefaultChains();

    return getDefaultConfig({
        appName,
        projectId,
        chains: resolvedChains as any,
        ssr,
        storage: createStorage({
            storage: cookieStorage,
        }),
    });
}

/**
 * Server-Side Configuration (Uses basic Wagmi createConfig)
 * Safe to use in Server Components and Layouts
 */
export function getServerConfig() {
    const chains = getDefaultChains();
    const transports = chains.reduce((acc, chain) => {
        acc[chain.id] = http();
        return acc;
    }, {} as Record<number, any>);

    return createConfig({
        chains: chains as [Chain, ...Chain[]],
        transports,
        ssr: true,
        storage: createStorage({
            storage: cookieStorage,
        }),
    });
}
