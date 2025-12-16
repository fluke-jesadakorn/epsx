/**
 * Pure Web3 Authentication Service for Admin Frontend
 * Signs every API request with wallet signature - no sessions or cookies
 * Implements request-level authentication using EIP-712 typed data signing
 * Optimized for admin operations with enhanced permission validation
 */

'use client';

import { PureWeb3ApiClient as BaseApiClient } from '../../../../shared/auth/web3/client';
import { createPureWeb3AuthStore } from '../../../../shared/auth/web3/store';
import { PureWeb3AuthActions, PureWeb3AuthState, PureWeb3AuthStore, SignedRequestHeaders } from '../../../../shared/auth/web3/types';

export type { PureWeb3AuthActions, PureWeb3AuthState, PureWeb3AuthStore, SignedRequestHeaders };

// Pure Web3 Auth Store for Admin
export const usePureWeb3AuthStore = createPureWeb3AuthStore({
  persistenceName: 'pure-web3-admin-auth-storage',
  requireAdmin: true,
  verifyMethod: 'GET'
});

// Admin API Client
class AdminWeb3ApiClient extends BaseApiClient {
  async getAdminStatus() {
    return this.get('/api/admin/status');
  }

  async getSystemStats() {
    return this.get('/api/admin/system/stats');
  }

  async listWallets(params: { limit?: number; offset?: number } = {}) {
    const q = new URLSearchParams();
    if (params.limit) q.set('limit', params.limit.toString());
    if (params.offset) q.set('offset', params.offset.toString());
    return this.get(`/api/admin/wallets?${q}`);
  }

  async grantWalletPermission(walletAddress: string, permission: string, expiresAt?: string) {
    return this.post(`/api/admin/wallets/${walletAddress}/permissions`, {
      permission,
      expires_at: expiresAt
    });
  }

  async revokeWalletPermission(walletAddress: string, permission: string) {
    return this.delete(`/api/admin/wallets/${walletAddress}/permissions/${permission}`);
  }

  async getWalletDetails(walletAddress: string) {
    return this.get(`/api/admin/wallets/${walletAddress}`);
  }
}

// Admin API methods with Web3 signing
export const web3AdminApi = new AdminWeb3ApiClient(usePureWeb3AuthStore);

// Hook for using Pure Web3 admin auth
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
export const usePureWeb3ConnectedState = () => usePureWeb3AuthStore(state => ({
  isConnected: state.isConnected,
  walletAddress: state.walletAddress,
  chainId: state.chainId,
}));

export const usePureWeb3AuthState = () => usePureWeb3AuthStore(state => ({
  isAuthenticated: state.isConnected && state.permissions.length > 0 && state.permissions.some(p => p.startsWith('admin:')),
  isAuthenticating: state.isAuthenticating,
  permissions: state.permissions,
  groups: state.groups,
}));

export const usePureWeb3LoadingState = () => usePureWeb3AuthStore(state => ({
  isLoading: state.isLoading,
  hasInitialized: state.hasInitialized,
  error: state.error,
}));