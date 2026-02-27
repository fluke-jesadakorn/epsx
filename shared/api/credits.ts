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
  PlanSwitchData,
} from '../types/credits';
import type { UnifiedApiClient } from '../utils/api-client';

function buildCreditParams(filters: CreditTransactionFilters): URLSearchParams {
  const params = new URLSearchParams();
  const { tx_type, from_date, to_date, limit, offset } = filters;
  if (tx_type !== undefined && tx_type !== '') { params.append('tx_type', tx_type); }
  if (from_date !== undefined && from_date !== '') { params.append('from_date', from_date); }
  if (to_date !== undefined && to_date !== '') { params.append('to_date', to_date); }
  if (limit !== undefined && limit !== 0) { params.append('limit', limit.toString()); }
  if (offset !== undefined && offset !== 0) { params.append('offset', offset.toString()); }
  return params;
}

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
    const query = filters !== undefined ? buildCreditParams(filters).toString() : '';
    const endpoint = query !== ''
      ? `/api/payments/credits/history?${query}`
      : '/api/payments/credits/history';
    return this.client.get<CreditHistoryResponse>(endpoint);
  }

  /**
   * Switch plan (downgrade with pro-rata credit or apply upgrade credit)
   */
  async switchPlan(newPlanId: string): Promise<ApiResponse<PlanSwitchData>> {
    return this.client.post<PlanSwitchData>('/api/payments/plans/switch', { new_plan_id: newPlanId });
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
    const query = filters !== undefined ? buildCreditParams(filters).toString() : '';
    const endpoint = query !== ''
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
