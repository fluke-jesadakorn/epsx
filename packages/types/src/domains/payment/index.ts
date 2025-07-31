export enum PaymentTier {
  BRONZE = 'BRONZE',
  SILVER = 'SILVER', 
  GOLD = 'GOLD',
  PLATINUM = 'PLATINUM'
}

export interface PaymentPlan {
  id: string;
  tier: PaymentTier;
  name: string;
  price: number;
  currency: string;
  features: string[];
  apiLimits: PaymentLimits;
  duration: number;
  numericLevel: number;
  color: string;
}

export interface PaymentLimits {
  requestsPerMinute: number;
  requestsPerDay: number;
  maxRankings: number;
  maxFileSize: number;
}

export interface UserSubscription {
  userId: string;
  tier: PaymentTier;
  subscriptionId?: string;
  validUntil?: Date;
  isActive: boolean;
  features: string[];
  lastPaymentDate?: Date;
  paymentMethod?: string;
  transactionId?: string;
  amount?: number;
}

// Traditional payment request (Stripe-style)
export interface CreatePaymentRequest {
  planId: string;
  paymentMethod: string;
  billingAddress?: BillingAddress;
  couponCode?: string;
}

// Crypto payment request (USDT/crypto-style)
export interface CreateCryptoPaymentRequest {
  currency: string;
  amount: string;
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  notify_url?: string;
}

// Traditional payment response
export interface CreatePaymentResponse {
  paymentIntentId: string;
  clientSecret: string;
  subscriptionId: string;
  status: PaymentStatus;
}

// Crypto payment response  
export interface CreateCryptoPaymentResponse extends PaymentResponse {
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  order_no: string;
  order_amount: number;
  receive_address?: string;
  checkout_url?: string;
}

// Unified payment response interface
export interface PaymentResponse {
  id: string;
  amount: number;
  currency: string;
  status: "Pending" | "Processing" | "Succeeded" | "Failed" | "Cancelled" | "Expired" | "RequiresAction";
  created_at: string;
  updated_at: string;
  expiration_date: string;
  payment_tier: PaymentTier;
  qr_code?: string;
  checkout_url?: string;
  payment_method: string;
  retry_count: number;
  error_message?: string;
}

export interface PaymentStatusResponse {
  paymentIntentId: string;
  subscriptionId: string;
  status: PaymentStatus;
  amount: number;
  currency: string;
  nextBillingDate?: Date;
}

export enum PaymentStatus {
  PENDING = 'PENDING',
  SUCCEEDED = 'SUCCEEDED',
  FAILED = 'FAILED',
  CANCELLED = 'CANCELLED',
  REFUNDED = 'REFUNDED'
}

export interface BillingAddress {
  line1: string;
  line2?: string;
  city: string;
  state: string;
  postalCode: string;
  country: string;
}

// Crypto asset information
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