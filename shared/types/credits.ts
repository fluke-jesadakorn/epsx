/**
 * Credit Wallet Types
 * TypeScript interfaces for credit balance and transactions
 */

export interface CreditBalance {
  wallet_address: string;
  balance: number | string;
  pending_balance: number | string;
  available_balance: number | string;
  lifetime_earned: number | string;
  lifetime_spent: number | string;
  last_transaction_at: string | null;
}

export interface CreditTransaction {
  id: string;
  wallet_address: string;
  amount: number | string;
  balance_after: number | string;
  tx_type: CreditTransactionType;
  reference_id: string | null;
  reference_type: string | null;
  reason: string | null;
  granted_by: string | null;
  expires_at: string | null;
  created_at: string;
}

export type CreditTransactionType =
  | 'grant'
  | 'revoke'
  | 'payment_debit'
  | 'proration_credit'
  | 'refund'
  | 'expiry'
  | 'adjustment';

export interface CreditTransactionFilters {
  tx_type?: CreditTransactionType;
  from_date?: string;
  to_date?: string;
  limit?: number;
  offset?: number;
}

export interface CreditStats {
  total_credits_outstanding: number | string;
  total_credits_granted_today: number | string;
  total_credits_used_today: number | string;
  active_users_with_credits: number;
  total_transactions_today: number;
  average_balance: number | string;
}

export interface GrantCreditsRequest {
  wallet_address: string;
  amount: number;
  reason?: string;
  expires_at?: string;
  granted_by: string;
}

export interface RevokeCreditsRequest {
  wallet_address: string;
  amount: number;
  reason?: string;
  granted_by: string;
}

export interface CreditHistoryResponse {
  success: boolean;
  data: CreditTransaction[];
  count: number;
}

// Plan switch response data
export interface PlanSwitchData {
  proration_credit: string;
  new_wallet_balance: string;
  new_plan_name: string;
  new_plan_expires_at: string | null;
  switch_type: 'downgrade' | 'upgrade_credit_applied';
}

// Upgrade preview with credits
export interface UpgradePreviewWithCredits {
  current_plan: {
    name: string;
    price: string;
    expires_at: string | null;
    started_at: string | null;
    days_remaining: number;
  } | null;
  new_plan: {
    id: number;
    name: string;
    price: string;
  };
  credit_from_current_plan: string;
  wallet_credit_balance: string;
  total_credits_available: string;
  amount_to_pay: string;
  new_duration_days: number;
  new_expiry_date: string;
  is_upgrade_allowed: boolean;
}
