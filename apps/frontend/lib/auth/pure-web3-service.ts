/**
 * Pure Web3 Authentication Service
 * Signs every API request with wallet signature - no sessions or cookies
 * Implements request-level authentication using EIP-712 typed data signing
 */

'use client';

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// Types for Pure Web3 Authentication
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
  
  // Bearer token data (from Web3 auth verification)
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
  setBearerToken: (token?: string, expiresAt?: string) => void;
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
  'X-Web3-Signature': string;  // Standardized header name
  'X-Signed-Message': string;   // Standardized header name
  'X-Timestamp': string;
  'X-Nonce': string;
}

export type PureWeb3AuthStore = PureWeb3AuthState & PureWeb3AuthActions;

// Global state for wallet signature function (injected by Web3 provider)
declare global {
  interface Window {
    __pureWeb3_signMessage?: (message: string) => Promise<string>;
    __pureWeb3_getAccount?: () => { address: string; chainId: number } | null;
  }
}

// Backend URLs
const getBackendUrl = () => process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';

// Pure Web3 Auth Store
export const usePureWeb3AuthStore = create<PureWeb3AuthStore>()(
  persist(
    (set, get) => ({
      // Initial state
      isConnected: false,
      isAuthenticating: false,
      isLoading: true,
      hasInitialized: false,
      chainId: 1,
      permissions: [],
      groups: [],
      bearerToken: undefined,
      tokenExpiresAt: undefined,

      // State setters
      setConnected: (connected, address, chainId) => set({ 
        isConnected: connected, 
        walletAddress: address,
        chainId: chainId || 1
      }),
      setAuthenticating: (authenticating) => set({ isAuthenticating: authenticating }),
      setLoading: (loading) => set({ isLoading: loading }),
      setInitialized: (initialized) => set({ hasInitialized: initialized }),
      setPermissions: (permissions) => set({ permissions }),
      setGroups: (groups) => set({ groups }),
      setBearerToken: (token, expiresAt) => set({ bearerToken: token, tokenExpiresAt: expiresAt }),
      setNonce: (nonce, expiry) => set({ currentNonce: nonce, nonceExpiry: expiry }),
      clearNonce: () => set({ currentNonce: undefined, nonceExpiry: undefined }),
      setError: (error) => set({ error }),

      // Generate authentication challenge from backend
      generateChallenge: async (endpoint = '/auth/verify') => {
        const state = get();
        
        if (!state.walletAddress) {
          throw new Error('Wallet not connected');
        }

        try {
          const response = await fetch(`${getBackendUrl()}/api/auth/web3/challenge`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              wallet_address: state.walletAddress,
              chain_id: state.chainId,
              endpoint: endpoint
            }),
          });

          if (!response.ok) {
            throw new Error(`Failed to get challenge: ${response.status}`);
          }

          const challenge = await response.json();
          
          // Store nonce with expiry
          const expiryTime = new Date(challenge.expires_at).getTime();
          set({ 
            currentNonce: challenge.nonce, 
            nonceExpiry: expiryTime 
          });
          
          return {
            nonce: challenge.nonce,
            message: challenge.message,
            chainId: challenge.chain_id
          };
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : 'Challenge generation failed';
          set({ error: errorMsg });
          throw new Error(errorMsg);
        }
      },

      // Verify wallet connection by testing signature
      verifyConnection: async () => {
        const state = get();
        
        if (!state.walletAddress || !window.__pureWeb3_signMessage) {
          return false;
        }

        try {
          set({ isAuthenticating: true, error: undefined });
          
          // Generate challenge
          const challenge = await get().generateChallenge('/api/auth/web3/verify');
          
          if (!challenge.message) {
            throw new Error('Invalid challenge received from backend');
          }
          
          // Sign the message (ensure it's a string)
          const messageToSign = typeof challenge.message === 'string' ? challenge.message : JSON.stringify(challenge.message);
          
          if (!messageToSign.trim()) {
            throw new Error('Empty message cannot be signed');
          }
          
          const signature = await window.__pureWeb3_signMessage(messageToSign);
          
          // Verify with backend using POST request with JSON body
          const response = await fetch(`${getBackendUrl()}/api/auth/web3/verify`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              wallet_address: state.walletAddress,
              message: challenge.message,
              signature: signature,
              nonce: challenge.nonce,
            }),
          });

          if (response.ok) {
            const verifyData = await response.json();
            set({
              isAuthenticating: false,
              permissions: verifyData.permissions || [],
              groups: verifyData.groups || [],
              bearerToken: verifyData.bearer_token,
              tokenExpiresAt: verifyData.token_expires_at
            });
            return true;
          } else {
            set({ isAuthenticating: false, error: 'Signature verification failed' });
            return false;
          }
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : 'Connection verification failed';
          set({ isAuthenticating: false, error: errorMsg });
          return false;
        }
      },

      // Refresh permissions from backend
      refreshPermissions: async () => {
        const state = get();
        
        if (!state.walletAddress) {
          throw new Error('Wallet not connected');
        }

        try {
          // Sign request for permissions endpoint
          const signedHeaders = await get().signRequest('/api/v1/users/permissions', 'GET');
          
          const response = await fetch(`${getBackendUrl()}/api/v1/users/permissions`, {
            method: 'GET',
            headers: signedHeaders as any,
          });

          if (response.ok) {
            const data = await response.json();
            set({
              permissions: data.unique_permissions || [],
              groups: data.group_permissions?.map((g: any) => ({
                group_id: g.group_name,
                name: g.group_name,
                permissions: [g.permission]
              })) || []
            });
          }
        } catch (error) {
          console.warn('Failed to refresh permissions:', error);
        }
      },

      // Sign out (clear nonces on backend)
      signOut: async () => {
        const state = get();
        
        try {
          if (state.walletAddress && state.currentNonce) {
            // Clear nonces on backend
            const signedHeaders = await get().signRequest('/api/auth/web3/logout', 'DELETE');
            
            await fetch(`${getBackendUrl()}/api/auth/web3/logout`, {
              method: 'DELETE',
              headers: signedHeaders as any,
              body: JSON.stringify({ clear_all_sessions: true }),
            });
          }
        } catch (error) {
          console.warn('Logout request failed:', error);
        } finally {
          // Always clear local state
          get().resetState();
        }
      },

      // Sign API request with wallet signature
      signRequest: async (endpoint: string, method: string, body?: any): Promise<SignedRequestHeaders> => {
        const state = get();
        
        if (!state.walletAddress) {
          throw new Error('Wallet not connected');
        }

        if (!window.__pureWeb3_signMessage) {
          throw new Error('Wallet signing not available');
        }

        try {
          // Check if we need a new nonce
          const now = Date.now();
          let nonce = state.currentNonce;
          let needNewNonce = !nonce || !state.nonceExpiry || state.nonceExpiry <= now;
          
          if (needNewNonce) {
            const challenge = await get().generateChallenge(endpoint);
            nonce = challenge.nonce;
          }

          if (!nonce) {
            throw new Error('Failed to get nonce for request signing');
          }

          // Create signing message
          const timestamp = Math.floor(Date.now() / 1000);
          const bodyHash = body ? JSON.stringify(body) : '';
          
          const message = [
            `EPSX API Request`,
            `Wallet: ${state.walletAddress}`,
            `Method: ${method}`,
            `Endpoint: ${endpoint}`,
            `Chain ID: ${state.chainId}`,
            `Timestamp: ${timestamp}`,
            `Nonce: ${nonce}`,
            `Body: ${bodyHash}`
          ].join('\n');

          // Sign the message (ensure it's a string)
          const messageToSign = typeof message === 'string' ? message : JSON.stringify(message);
          const signature = await window.__pureWeb3_signMessage(messageToSign);

          return {
            'X-Wallet-Address': state.walletAddress,
            'X-Chain-Id': state.chainId.toString(),
            'X-Web3-Signature': signature,
            'X-Signed-Message': message,
            'X-Timestamp': timestamp.toString(),
            'X-Nonce': nonce
          };
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : 'Request signing failed';
          set({ error: errorMsg });
          throw new Error(errorMsg);
        }
      },

      // Reset all state
      resetState: () => set({
        isConnected: false,
        isAuthenticating: false,
        isLoading: false,
        hasInitialized: false,
        walletAddress: undefined,
        chainId: 1,
        permissions: [],
        groups: [],
        bearerToken: undefined,
        tokenExpiresAt: undefined,
        currentNonce: undefined,
        nonceExpiry: undefined,
        error: undefined,
      }),
    }),
    {
      name: 'pure-web3-auth-storage',
      partialize: (state) => ({
        // Only persist non-sensitive state
        chainId: state.chainId,
        // Don't persist: walletAddress, nonces, permissions, etc.
      }),
    }
  )
);

// API Client for Pure Web3 Requests
export class PureWeb3ApiClient {
  private baseUrl: string;
  private authStore: any;

  constructor() {
    this.baseUrl = getBackendUrl();
    this.authStore = usePureWeb3AuthStore;
  }

  // Generic request method with automatic signing
  async request<T>(
    endpoint: string, 
    options: {
      method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
      body?: any;
      headers?: Record<string, string>;
    } = {}
  ): Promise<T> {
    const { method = 'GET', body, headers = {} } = options;
    
    try {
      // Get signed headers
      const signedHeaders = await this.authStore.getState().signRequest(
        endpoint, 
        method, 
        body
      );

      // Prepare request
      const requestOptions: RequestInit = {
        method,
        headers: {
          'Content-Type': 'application/json',
          ...headers,
          ...signedHeaders,
        },
      };

      if (body && (method === 'POST' || method === 'PUT')) {
        requestOptions.body = JSON.stringify(body);
      }

      // Make request
      const response = await fetch(`${this.baseUrl}${endpoint}`, requestOptions);

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || `Request failed: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error(`Pure Web3 API request failed [${method} ${endpoint}]:`, error);
      throw error;
    }
  }

  // Convenience methods
  async get<T>(endpoint: string, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET', headers });
  }

  async post<T>(endpoint: string, body?: any, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'POST', body, headers });
  }

  async put<T>(endpoint: string, body?: any, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'PUT', body, headers });
  }

  async delete<T>(endpoint: string, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE', headers });
  }

  // Specific API methods
  async getWalletStatus() {
    return this.get('/auth/status');
  }

  async getWalletPermissions(includeExpired = false) {
    return this.get(`/auth/permissions?include_expired=${includeExpired}`);
  }

  async getWalletProfile() {
    return this.get('/auth/profile');
  }

  // Admin API methods (when user has admin permissions)
  async listWallets(params: { limit?: number; offset?: number } = {}) {
    const query = new URLSearchParams();
    if (params.limit) query.set('limit', params.limit.toString());
    if (params.offset) query.set('offset', params.offset.toString());
    return this.get(`/admin/wallets?${query}`);
  }

  async grantWalletPermission(walletAddress: string, permission: string, expiresAt?: string) {
    return this.post(`/admin/wallets/${walletAddress}/permissions`, {
      permission,
      expires_at: expiresAt
    });
  }

  async revokeWalletPermission(walletAddress: string, permission: string) {
    return this.delete(`/admin/wallets/${walletAddress}/permissions/${permission}`);
  }

  // User API methods
  async updateWalletProfile(data: { display_name?: string; preferred_chain_id?: number }) {
    return this.put('/user/profile', data);
  }

  async checkWalletPermission(permission: string) {
    return this.post('/user/permissions/check', { permission });
  }

  async getWalletActivity(params: { limit?: number; offset?: number } = {}) {
    const query = new URLSearchParams();
    if (params.limit) query.set('limit', params.limit.toString());
    if (params.offset) query.set('offset', params.offset.toString());
    return this.get(`/user/activity?${query}`);
  }
}

// Singleton API client
export const pureWeb3ApiClient = new PureWeb3ApiClient();

// Hook for using Pure Web3 auth
export function usePureWeb3Auth() {
  const store = usePureWeb3AuthStore();
  
  return {
    // State
    ...store,
    
    // Computed values
    isReady: store.hasInitialized && !store.isLoading,
    isAuthorized: store.isConnected && store.permissions.length > 0,
    
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
    hasPermission: (permission: string) => store.permissions.includes(permission),
    hasAnyPermission: (permissions: string[]) => 
      permissions.some(p => store.permissions.includes(p)),
    hasAllPermissions: (permissions: string[]) => 
      permissions.every(p => store.permissions.includes(p)),
    
    isAdmin: () => store.permissions.some(p => p.startsWith('admin:')),
    
    // API client
    api: pureWeb3ApiClient
  };
}

// Selector hooks for performance
export const usePureWeb3ConnectedState = () => usePureWeb3AuthStore(state => ({
  isConnected: state.isConnected,
  walletAddress: state.walletAddress,
  chainId: state.chainId,
}));

export const usePureWeb3AuthState = () => usePureWeb3AuthStore(state => ({
  isAuthenticated: state.isConnected && state.permissions.length > 0,
  isAuthenticating: state.isAuthenticating,
  permissions: state.permissions,
  groups: state.groups,
}));

export const usePureWeb3LoadingState = () => usePureWeb3AuthStore(state => ({
  isLoading: state.isLoading,
  hasInitialized: state.hasInitialized,
  error: state.error,
}));