/**
 * Pure Web3 Authentication Service
 * Signs every API request with wallet signature - no sessions or cookies
 * Implements request-level authentication using EIP-712 typed data signing
 */

'use client';

import { PureWeb3ApiClient as BaseApiClient } from '../../../../shared/auth/web3/client';
import { createPureWeb3AuthStore } from '../../../../shared/auth/web3/store';

export type { PureWeb3AuthActions, PureWeb3AuthState, PureWeb3AuthStore, SignedRequestHeaders } from '../../../../shared/auth/web3/types';

// Pure Web3 Auth Store
export const usePureWeb3AuthStore = createPureWeb3AuthStore({
  persistenceName: 'pure-web3-auth-storage',
  requireAdmin: false,
  verifyMethod: 'POST'
});

// API Client for Pure Web3 Verification
export class PureWeb3ApiClient extends BaseApiClient {
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
// We need to adhere to the constructor signature of BaseApiClient: constructor(authStore: AuthStore)
// usePureWeb3AuthStore is a hook, but it also has .getState() method attached to it in Zustand?
// Yes, create() returns a hook that is also the store API.
export const pureWeb3ApiClient = new PureWeb3ApiClient(usePureWeb3AuthStore);

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