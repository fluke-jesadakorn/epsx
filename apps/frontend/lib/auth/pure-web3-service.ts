/**
 * Pure Web3 Authentication Service
 * Signs every API request with wallet signature - no sessions or cookies
 * Implements request-level authentication using EIP-712 typed data signing
 */

'use client';

import { PureWeb3ApiClient as BaseApiClient } from '@/shared/auth/web3/client';
import {
  createPureWeb3Hooks,
  createUsePureWeb3Auth
} from '@/shared/auth/web3/factory';
import { createPureWeb3AuthStore } from '@/shared/auth/web3/store';

export type { PureWeb3AuthActions, PureWeb3AuthState, PureWeb3AuthStore, SignedRequestHeaders } from '@/shared/auth/web3/types';

// Pure Web3 Auth Store
export const usePureWeb3AuthStore = createPureWeb3AuthStore({
  persistenceName: 'pure-web3-auth-storage',
  requireAdmin: false,
  verifyMethod: 'POST'
});

// API Client for Pure Web3 Verification (Frontend-specific extensions)
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
export const pureWeb3ApiClient = new PureWeb3ApiClient(usePureWeb3AuthStore);

// Use shared hook factory for common hooks
export const usePureWeb3Auth = createUsePureWeb3Auth(usePureWeb3AuthStore, pureWeb3ApiClient, 'frontend');

// Use shared selector hooks
const {
  usePureWeb3ConnectedState,
  usePureWeb3AuthState,
  usePureWeb3LoadingState
} = createPureWeb3Hooks(usePureWeb3AuthStore, 'frontend');

export { usePureWeb3AuthState, usePureWeb3ConnectedState, usePureWeb3LoadingState };
