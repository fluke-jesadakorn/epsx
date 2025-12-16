export interface PureWeb3AuthState {
    // Connection state
    isConnected: boolean;
    isAuthenticating: boolean;
    isLoading: boolean;
    hasInitialized: boolean;

    // Wallet data
    walletAddress?: string;
    chainId: number;

    // Permission data (from backend)
    permissions: string[];
    groups: Array<{
        group_id: string;
        name: string;
        permissions: string[];
    }>;

    // Bearer token data (from Web3 authentication)
    bearerToken?: string;
    tokenExpiresAt?: string;

    // Nonce management
    currentNonce?: string;
    nonceExpiry?: number;

    // Error state
    error?: string;
}

export interface PureWeb3AuthActions {
    // State management
    setConnected: (connected: boolean, address?: string, chainId?: number) => void;
    setAuthenticating: (authenticating: boolean) => void;
    setLoading: (loading: boolean) => void;
    setInitialized: (initialized: boolean) => void;
    setPermissions: (permissions: string[]) => void;
    setGroups: (groups: PureWeb3AuthState['groups']) => void;
    setBearerToken: (token: string, expiresAt: string) => void;
    clearBearerToken: () => void;
    setNonce: (nonce: string, expiry: number) => void;
    clearNonce: () => void;
    setError: (error?: string) => void;

    // Core authentication
    generateChallenge: (endpoint?: string) => Promise<{ nonce: string; message: string; chainId: number }>;
    verifyConnection: () => Promise<boolean>;
    refreshPermissions: () => Promise<void>;
    signOut: () => Promise<void>;

    // Request signing
    signRequest: (endpoint: string, method: string, body?: any) => Promise<SignedRequestHeaders>;

    // Utility
    resetState: () => void;
}

export interface SignedRequestHeaders {
    'X-Wallet-Address': string;
    'X-Chain-Id': string;
    'X-Web3-Signature': string;
    'X-Signed-Message': string;
    'X-Timestamp': string;
    'X-Nonce': string;
    [key: string]: string;
}

export type PureWeb3AuthStore = PureWeb3AuthState & PureWeb3AuthActions;

// Global state for wallet signature function (injected by Web3 provider)
declare global {
    interface Window {
        __pureWeb3_signMessage?: (message: string) => Promise<string>;
        __pureWeb3_getAccount?: () => { address: string; chainId: number } | null;
    }
}
