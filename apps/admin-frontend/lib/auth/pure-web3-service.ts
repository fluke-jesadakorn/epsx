/**
 * Pure Web3 Authentication Service for Admin Frontend
 * Signs every API request with wallet signature - no sessions or cookies
 * Implements request-level authentication using EIP-712 typed data signing
 * Optimized for admin operations with enhanced permission validation
 */

'use client';

import { PureWeb3ApiClient as BaseApiClient } from '@/shared/auth/web3/client';
import {
  createPureWeb3Hooks,
  createUsePureWeb3Auth
} from '@/shared/auth/web3/factory';
import { createPureWeb3AuthStore } from '@/shared/auth/web3/store';
import { PureWeb3AuthActions, PureWeb3AuthState, PureWeb3AuthStore, SignedRequestHeaders } from '@/shared/auth/web3/types';

export type { PureWeb3AuthActions, PureWeb3AuthState, PureWeb3AuthStore, SignedRequestHeaders };

// Pure Web3 Auth Store for Admin
export const usePureWeb3AuthStore = createPureWeb3AuthStore({
  persistenceName: 'pure-web3-admin-auth-storage',
  requireAdmin: true,
  verifyMethod: 'GET'
});

// Admin API Client (Admin-specific extensions)
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

// Admin API singleton
export const web3AdminApi = new AdminWeb3ApiClient(usePureWeb3AuthStore);

// Use shared hook factory for common hooks (configured for admin)
export const usePureWeb3Auth = createUsePureWeb3Auth(usePureWeb3AuthStore, web3AdminApi, 'admin');

// Use shared selector hooks (configured for admin)
const {
  usePureWeb3ConnectedState,
  usePureWeb3AuthState,
  usePureWeb3LoadingState
} = createPureWeb3Hooks(usePureWeb3AuthStore, 'admin');

export { usePureWeb3AuthState, usePureWeb3ConnectedState, usePureWeb3LoadingState };
