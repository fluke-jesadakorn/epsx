import { PaymentTier, UserSubscription } from '@epsx/types';

export interface CreatePaymentRequest {
  currency: string;
  amount: string;
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  notify_url?: string;
}

export interface CreatePaymentResponse extends PaymentResponse {
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  order_no: string;
  order_amount: number;
  receive_address?: string;
  checkout_url?: string;
}

export interface AssetInfo {
  currency: string;
  name: string;
  symbol: string;
  decimals: number;
  contract_address?: string;
  chain?: string;
  depositThreshold?: number;
  addressFormat?: string;
}

export interface PaymentResponse {
  id: string;
  amount: number;
  currency: string;
  status: "Pending" | "Processing" | "Succeeded" | "Failed" | "Cancelled" | "Expired" | "RequiresAction";
  created_at: string;
  updated_at: string;
  expiration_date: string;
  payment_tier: PaymentTier; // Changed from user_level to payment_tier
  qr_code?: string;
  checkout_url?: string;
  payment_method: string;
  retry_count: number;
  error_message?: string;
}
