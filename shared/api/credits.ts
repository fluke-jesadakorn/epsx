/**
 * Credits API Client
 * Handles credit balance, history, and admin operations
 */

import type { ApiResponse } from '../types/api';
import type {
  CreditBalance,
  CreditTransaction,
  CreditTransactionFilters,
  CreditStats,
  GrantCreditsRequest,
  RevokeCreditsRequest,
  CreditHistoryResponse,
} from '../types/credits';
import { UnifiedApiClient } from '../utils/api-client';

export class CreditsApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  /**
   * Get authenticated user's credit balance
   */
  async getBalance(): Promise<ApiResponse<CreditBalance>> {
    return this.client.get<CreditBalance>('/api/payments/credits/balance');
  }

  /**
   * Get authenticated user's credit transaction history
   */
  async getHistory(
    filters?: CreditTransactionFilters
  ): Promise<ApiResponse<CreditHistoryResponse>> {
    const params = new URLSearchParams();

    if (filters?.tx_type) {
      params.append('tx_type', filters.tx_type);
    }
    if (filters?.from_date) {
      params.append('from_date', filters.from_date);
    }
    if (filters?.to_date) {
      params.append('to_date', filters.to_date);
    }
    if (filters?.limit) {
      params.append('limit', filters.limit.toString());
    }
    if (filters?.offset) {
      params.append('offset', filters.offset.toString());
    }

    const query = params.toString();
    const endpoint = query
      ? `/api/payments/credits/history?${query}`
      : '/api/payments/credits/history';

    return this.client.get<CreditHistoryResponse>(endpoint);
  }

  // ==========================================================================
  // ADMIN METHODS
  // ==========================================================================

  /**
   * Get user's credit balance and history (admin)
   */
  async adminGetUserCredits(
    walletAddress: string,
    filters?: CreditTransactionFilters
  ): Promise<ApiResponse<{
    balance: CreditBalance;
    transactions: CreditTransaction[];
  }>> {
    const params = new URLSearchParams();

    if (filters?.tx_type) {
      params.append('tx_type', filters.tx_type);
    }
    if (filters?.from_date) {
      params.append('from_date', filters.from_date);
    }
    if (filters?.to_date) {
      params.append('to_date', filters.to_date);
    }
    if (filters?.limit) {
      params.append('limit', filters.limit.toString());
    }
    if (filters?.offset) {
      params.append('offset', filters.offset.toString());
    }

    const query = params.toString();
    const endpoint = query
      ? `/api/payments/admin/credits/${walletAddress}?${query}`
      : `/api/payments/admin/credits/${walletAddress}`;

    return this.client.get(endpoint);
  }

  /**
   * Grant credits to a user (admin)
   */
  async adminGrantCredits(
    request: GrantCreditsRequest
  ): Promise<ApiResponse<{
    transaction_id: string;
    new_balance: number;
  }>> {
    return this.client.post('/api/payments/admin/credits/grant', request);
  }

  /**
   * Revoke credits from a user (admin)
   */
  async adminRevokeCredits(
    request: RevokeCreditsRequest
  ): Promise<ApiResponse<{
    transaction_id: string;
    new_balance: number;
  }>> {
    return this.client.post('/api/payments/admin/credits/revoke', request);
  }

  /**
   * Get credit system statistics (admin)
   */
  async adminGetStats(): Promise<ApiResponse<CreditStats>> {
    return this.client.get<CreditStats>('/api/payments/admin/credits/stats');
  }
}

/**
 * Factory function to create CreditsApi instance
 */
export function createCreditsApi(client: UnifiedApiClient): CreditsApi {
  return new CreditsApi(client);
}
