import { UserLevel } from "./userLevel.d.ts";

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
}

export interface AssetInfo {
  currency: string;
  name: string;
  symbol: string;
  decimals: number;
  contract_address?: string;
}

export interface PaymentResponse {
  id: string;
  amount: number;
  currency: string;
  status: "Pending" | "Processing" | "Succeeded" | "Failed" | "USDT";
  created_at: string;
  expiration_date: string;
  user_level: UserLevel;
  qr_code: string;
}
