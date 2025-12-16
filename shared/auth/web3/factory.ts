/**
 * Pure Web3 Service Factory
 * Creates platform-specific Pure Web3 Auth services with shared hook implementations
 */

import { PureWeb3ApiClient } from './client';
import { createPureWeb3AuthStore, PureWeb3StoreConfig } from './store';
import { PureWeb3AuthStore } from './types';

export interface PureWeb3ServiceConfig extends PureWeb3StoreConfig {
    platform: 'frontend' | 'admin';
}

export type PureWeb3StoreHook = ReturnType<typeof createPureWeb3AuthStore>;

/**
 * Create common selector hooks for Pure Web3 auth
 */
export function createPureWeb3Hooks(useStore: PureWeb3StoreHook, platform: 'frontend' | 'admin') {
    const isAdmin = platform === 'admin';

    /**
     * Hook for connected state (wallet address, chain)
     */
    const usePureWeb3ConnectedState = () => useStore((state: PureWeb3AuthStore) => ({
        isConnected: state.isConnected,
        walletAddress: state.walletAddress,
        chainId: state.chainId,
    }));

    /**
     * Hook for authentication state (permissions, groups)
     */
    const usePureWeb3AuthState = () => useStore((state: PureWeb3AuthStore) => ({
        isAuthenticated: isAdmin
            ? state.isConnected && state.permissions.length > 0 && state.permissions.some((p: string) => p.startsWith('admin:'))
            : state.isConnected && state.permissions.length > 0,
        isAuthenticating: state.isAuthenticating,
        permissions: state.permissions,
        groups: state.groups,
    }));

    /**
     * Hook for loading state  
     */
    const usePureWeb3LoadingState = () => useStore((state: PureWeb3AuthStore) => ({
        isLoading: state.isLoading,
        hasInitialized: state.hasInitialized,
        error: state.error,
    }));

    return {
        usePureWeb3ConnectedState,
        usePureWeb3AuthState,
        usePureWeb3LoadingState,
    };
}

/**
 * Create the main usePureWeb3Auth hook
 */
export function createUsePureWeb3Auth<TApiClient extends PureWeb3ApiClient>(
    useStore: PureWeb3StoreHook,
    apiClient: TApiClient,
    platform: 'frontend' | 'admin'
) {
    const isAdmin = platform === 'admin';

    return function usePureWeb3Auth() {
        const store = useStore();

        return {
            // State
            ...store,

            // Computed values
            isReady: store.hasInitialized && !store.isLoading,
            isAuthorized: isAdmin
                ? store.isConnected && store.permissions.length > 0 && (store.permissions as string[]).some(p => p.startsWith('admin:'))
                : store.isConnected && store.permissions.length > 0,

            // Actions
            connect: async (address: string, chainId: number) => {
                store.setConnected(true, address, chainId);
                store.setInitialized(true);
                await store.verifyConnection();
            },

            disconnect: async () => {
                await store.signOut();
            },

            // Permission helpers
            hasPermission: (permission: string) => (store.permissions as string[]).includes(permission),
            hasAnyPermission: (permissions: string[]) =>
                permissions.some(p => (store.permissions as string[]).includes(p)),
            hasAllPermissions: (permissions: string[]) =>
                permissions.every(p => (store.permissions as string[]).includes(p)),

            isAdmin: () => (store.permissions as string[]).some(p => p.startsWith('admin:')),

            // API client
            api: apiClient
        };
    };
}

/**
 * Create a complete Pure Web3 service with store, hooks, and base API client
 */
export function createPureWeb3Service(config: PureWeb3ServiceConfig) {
    const useStore = createPureWeb3AuthStore(config);
    const baseApiClient = new PureWeb3ApiClient(useStore);
    const hooks = createPureWeb3Hooks(useStore, config.platform);
    const usePureWeb3Auth = createUsePureWeb3Auth(useStore, baseApiClient, config.platform);

    return {
        useStore,
        baseApiClient,
        usePureWeb3Auth,
        ...hooks,
    };
}

export { createPureWeb3AuthStore, PureWeb3ApiClient };
export type { PureWeb3StoreConfig };

