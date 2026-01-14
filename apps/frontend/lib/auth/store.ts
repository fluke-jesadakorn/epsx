'use client';

/**
 * FRONTEND AUTH STORE
 * Minimal Zustand store for Web3/SIWE state management
 * Uses cookies for persistence (no localStorage)
 */

import { create } from 'zustand';

interface Web3AuthState {
    // Connection state
    isConnected: boolean;
    walletAddress: string | undefined;

    // Auth state
    isAuthenticated: boolean;
    isAuthenticating: boolean;
    isLoading: boolean;
    hasInitialized: boolean;

    // User data
    permissions: string[];
    enterpriseTier: string;
    hasApiAccess: boolean;
    error: string | undefined;

    // Actions
    setConnected: (connected: boolean) => void;
    setWalletAddress: (address: string | undefined) => void;
    setAuthenticated: (authenticated: boolean) => void;
    setAuthenticating: (authenticating: boolean) => void;
    setLoading: (loading: boolean) => void;
    setInitialized: (initialized: boolean) => void;
    setPermissions: (permissions: string[]) => void;
    setEnterpriseTier: (tier: string) => void;
    setApiAccess: (hasAccess: boolean) => void;
    setError: (error: string | undefined) => void;

    // Complex actions (stubs - actual implementation uses SharedAuthProvider)
    authenticate: () => Promise<boolean>;
    logout: () => Promise<void>;
    getUser: () => Promise<any>;
    clearError: () => void;
    resetAuthState: () => void;
}

// Create store without localStorage persistence (cookies are used via SharedAuthProvider)
export const useWeb3AuthStore = create<Web3AuthState>()((set, get) => ({
    // Initial state
    isConnected: false,
    walletAddress: undefined,
    isAuthenticated: false,
    isAuthenticating: false,
    isLoading: false,
    hasInitialized: false,
    permissions: [],
    enterpriseTier: 'Starter',
    hasApiAccess: false,
    error: undefined,

    // Setters
    setConnected: (connected) => set({ isConnected: connected }),
    setWalletAddress: (address) => set({ walletAddress: address }),
    setAuthenticated: (authenticated) => set({ isAuthenticated: authenticated }),
    setAuthenticating: (authenticating) => set({ isAuthenticating: authenticating }),
    setLoading: (loading) => set({ isLoading: loading }),
    setInitialized: (initialized) => set({ hasInitialized: initialized }),
    setPermissions: (permissions) => set({ permissions }),
    setEnterpriseTier: (tier) => set({ enterpriseTier: tier }),
    setApiAccess: (hasAccess) => set({ hasApiAccess: hasAccess }),
    setError: (error) => set({ error }),

    // Complex actions - these are stubs, actual auth uses SharedAuthProvider
    authenticate: async () => {
        console.warn('⚠️ authenticate() called on legacy store - use SharedAuthProvider instead');
        return false;
    },

    logout: async () => {
        set({
            isAuthenticated: false,
            isConnected: false,
            walletAddress: undefined,
            permissions: [],
            enterpriseTier: 'Starter',
            hasApiAccess: false,
        });
    },

    getUser: async () => {
        console.warn('⚠️ getUser() called on legacy store - use SharedAuthProvider instead');
        return null;
    },

    clearError: () => set({ error: undefined }),

    resetAuthState: () => set({
        isAuthenticated: false,
        isAuthenticating: false,
        permissions: [],
        enterpriseTier: 'Starter',
        hasApiAccess: false,
        error: undefined,
    }),
}));

export default useWeb3AuthStore;