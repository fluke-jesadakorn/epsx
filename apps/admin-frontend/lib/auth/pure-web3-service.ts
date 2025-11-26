/**
 * Pure Web3 Authentication Service for Admin Frontend
 * Signs every API request with wallet signature - no sessions or cookies
 * Implements request-level authentication using EIP-712 typed data signing
 * Optimized for admin operations with enhanced permission validation
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

import { apiFetch } from '../api-fetch'
import { env } from '@/config/env'
import { ROUTES } from '../route-compatibility'

const BACKEND_URL = env.BACKEND_URL

// Pure Web3 Auth Store for Admin
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
      clearBearerToken: () => set({ bearerToken: undefined, tokenExpiresAt: undefined }),
      setNonce: (nonce, expiry) => set({ currentNonce: nonce, nonceExpiry: expiry }),
      clearNonce: () => set({ currentNonce: undefined, nonceExpiry: undefined }),
      setError: (error) => set({ error }),

      // Generate authentication challenge from backend
      generateChallenge: async (endpoint = ROUTES.AUTH.WEB3_VERIFY) => {
        const state = get();

        if (!state.walletAddress) {
          throw new Error('Admin wallet not connected');
        }

        try {
          const challenge = await apiFetch(ROUTES.AUTH.WEB3_CHALLENGE, {
            method: 'POST',
            body: JSON.stringify({
              wallet_address: state.walletAddress,
              chain_id: state.chainId,
              endpoint: endpoint
            })
          });

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
        } catch (_error) {
          const errorMsg = _error instanceof Error ? _error.message : 'Admin challenge generation failed';
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
          const challenge = await get().generateChallenge(ROUTES.AUTH.WEB3_VERIFY);
          
          // Sign the message
          const signature = await window.__pureWeb3_signMessage(challenge.message);
          
          // Verify with backend admin endpoint
          const headers: SignedRequestHeaders = {
            'X-Wallet-Address': state.walletAddress,
            'X-Chain-Id': state.chainId.toString(),
            'X-Web3-Signature': signature,      // Standardized header name
            'X-Signed-Message': challenge.message, // Standardized header name
            'X-Timestamp': Math.floor(Date.now() / 1000).toString(),
            'X-Nonce': challenge.nonce
          };

          const response = await fetch(`${BACKEND_URL}${ROUTES.AUTH.WEB3_VERIFY}`, {
            method: 'GET',
            headers: headers as any,
          });

          if (response.ok) {
            const verifyData = await response.json();
            
            // Ensure user has admin permissions
            const adminPermissions = (verifyData.permissions || []).filter((p: string) => p.startsWith('admin:'));
            if (adminPermissions.length === 0) {
              set({ 
                isAuthenticating: false, 
                error: 'No admin permissions found for this wallet' 
              });
              return false;
            }
            
            // Store Bearer token if provided by backend
            if (verifyData.bearer_token && verifyData.token_expires_at) {
              get().setBearerToken(verifyData.bearer_token, verifyData.token_expires_at);
            }
            
            set({
              isAuthenticating: false,
              permissions: verifyData.permissions || [],
              groups: verifyData.groups || []
            });
            return true;
          } else {
            set({ isAuthenticating: false, error: 'Admin signature verification failed' });
            return false;
          }
        } catch (_error) {
          const errorMsg = _error instanceof Error ? _error.message : 'Admin connection verification failed';
          set({ isAuthenticating: false, error: errorMsg });
          return false;
        }
      },

      // Refresh permissions from backend
      refreshPermissions: async () => {
        const state = get();
        
        if (!state.walletAddress) {
          throw new Error('Admin wallet not connected');
        }

        try {
          // Sign request for admin permissions endpoint
          const signedHeaders = await get().signRequest('/user/permissions', 'GET');
          
          const response = await fetch(`${BACKEND_URL}/user/permissions`, {
            method: 'GET',
            headers: signedHeaders as any,
          });

          if (response.ok) {
            const data = await response.json();
            const adminPermissions = (data.unique_permissions || []).filter((p: string) => p.startsWith('admin:'));
            
            set({
              permissions: data.unique_permissions || [],
              groups: data.group_permissions?.map((g: any) => ({
                group_id: g.group_name,
                name: g.group_name,
                permissions: [g.permission]
              })) || []
            });
          }
        } catch (_error) {
          // eslint-disable-next-line no-console
          console.warn('Failed to refresh admin permissions:', _error);
        }
      },

      // Sign out (clear nonces on backend)
      signOut: async () => {
        const state = get();
        
        try {
          if (state.walletAddress && state.currentNonce) {
            // Clear nonces on backend
            const signedHeaders = await get().signRequest(ROUTES.AUTH.WEB3_LOGOUT, 'DELETE');

            await fetch(`${BACKEND_URL}${ROUTES.AUTH.WEB3_LOGOUT}`, {
              method: 'DELETE',
              headers: signedHeaders as any,
              body: JSON.stringify({ clear_all_sessions: true }),
            });
          }
        } catch (_error) {
          // eslint-disable-next-line no-console
          console.warn('Admin logout request failed:', _error);
        } finally {
          // Always clear local state
          get().resetState();
        }
      },

      // Sign API request with wallet signature
      signRequest: async (endpoint: string, method: string, body?: any): Promise<SignedRequestHeaders> => {
        const state = get();
        
        if (!state.walletAddress) {
          throw new Error('Admin wallet not connected');
        }

        if (!window.__pureWeb3_signMessage) {
          throw new Error('Wallet signing not available');
        }

        try {
          // Check if we need a new nonce
          const now = Date.now();
          let nonce = state.currentNonce;
          const needNewNonce = !nonce || !state.nonceExpiry || state.nonceExpiry <= now;
          
          if (needNewNonce) {
            const challenge = await get().generateChallenge(endpoint);
            nonce = challenge.nonce;
          }

          if (!nonce) {
            throw new Error('Failed to get nonce for admin request signing');
          }

          // Create signing message
          const timestamp = Math.floor(Date.now() / 1000);
          const bodyHash = body ? JSON.stringify(body) : '';
          
          const message = [
            `EPSX Admin API Request`,
            `Wallet: ${state.walletAddress}`,
            `Method: ${method}`,
            `Endpoint: ${endpoint}`,
            `Chain ID: ${state.chainId}`,
            `Timestamp: ${timestamp}`,
            `Nonce: ${nonce}`,
            `Body: ${bodyHash}`
          ].join('\n');

          // Sign the message
          const signature = await window.__pureWeb3_signMessage(message);

          return {
            'X-Wallet-Address': state.walletAddress,
            'X-Chain-Id': state.chainId.toString(),
            'X-Web3-Signature': signature,    // Standardized header name
            'X-Signed-Message': message,      // Standardized header name
            'X-Timestamp': timestamp.toString(),
            'X-Nonce': nonce
          };
        } catch (_error) {
          const errorMsg = _error instanceof Error ? _error.message : 'Admin request signing failed';
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
      name: 'pure-web3-admin-auth-storage',
      partialize: (state) => ({
        // Only persist non-sensitive state
        chainId: state.chainId,
        // Don't persist: walletAddress, nonces, permissions, etc.
      }),
    }
  )
);

// Admin API methods with Web3 signing (no abstraction layer)
export const web3AdminApi = {
  async request<T>(endpoint: string, options: {
    method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
    body?: any
    headers?: Record<string, string>
  } = {}): Promise<T> {
    const { method = 'GET', body, headers = {} } = options
    const signedHeaders = await usePureWeb3AuthStore.getState().signRequest(endpoint, method, body)

    const res = await fetch(`${BACKEND_URL}${endpoint}`, {
      method,
      headers: { 'Content-Type': 'application/json', ...headers, ...signedHeaders },
      body: body && (method === 'POST' || method === 'PUT') ? JSON.stringify(body) : undefined
    })

    if (!res.ok) {
      const err = await res.json().catch(() => ({}))
      throw new Error(err.message || `Request failed: ${res.status}`)
    }

    return res.json()
  },

  get<T>(endpoint: string, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET', headers })
  },

  post<T>(endpoint: string, body?: any, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'POST', body, headers })
  },

  put<T>(endpoint: string, body?: any, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'PUT', body, headers })
  },

  delete<T>(endpoint: string, headers?: Record<string, string>): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE', headers })
  },

  getAdminStatus() {
    return this.get('/api/admin/status')
  },

  getSystemStats() {
    return this.get('/api/admin/system/stats')
  },

  listWallets(params: { limit?: number; offset?: number } = {}) {
    const q = new URLSearchParams()
    if (params.limit) q.set('limit', params.limit.toString())
    if (params.offset) q.set('offset', params.offset.toString())
    return this.get(`/api/admin/wallets?${q}`)
  },

  grantWalletPermission(walletAddress: string, permission: string, expiresAt?: string) {
    return this.post(`/api/admin/wallets/${walletAddress}/permissions`, {
      permission,
      expires_at: expiresAt
    })
  },

  revokeWalletPermission(walletAddress: string, permission: string) {
    return this.delete(`/api/admin/wallets/${walletAddress}/permissions/${permission}`)
  },

  getWalletDetails(walletAddress: string) {
    return this.get(`/api/admin/wallets/${walletAddress}`)
  }
}

// Hook for using Pure Web3 admin auth
/**
 *
 */
export function usePureWeb3Auth() {
  const store = usePureWeb3AuthStore();
  
  return {
    // State
    ...store,
    
    // Computed values
    isReady: store.hasInitialized && !store.isLoading,
    isAuthorized: store.isConnected && store.permissions.length > 0 && store.permissions.some(p => p.startsWith('admin:')),
    
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
    api: web3AdminApi
  };
}

// Selector hooks for performance
/**
 *
 */
export const usePureWeb3ConnectedState = () => usePureWeb3AuthStore(state => ({
  isConnected: state.isConnected,
  walletAddress: state.walletAddress,
  chainId: state.chainId,
}));

/**
 *
 */
export const usePureWeb3AuthState = () => usePureWeb3AuthStore(state => ({
  isAuthenticated: state.isConnected && state.permissions.length > 0 && state.permissions.some(p => p.startsWith('admin:')),
  isAuthenticating: state.isAuthenticating,
  permissions: state.permissions,
  groups: state.groups,
}));

/**
 *
 */
export const usePureWeb3LoadingState = () => usePureWeb3AuthStore(state => ({
  isLoading: state.isLoading,
  hasInitialized: state.hasInitialized,
  error: state.error,
}));