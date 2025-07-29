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

export interface CreatePaymentRequest {
  planId: string;
  paymentMethod: string;
  billingAddress?: BillingAddress;
  couponCode?: string;
}

export interface CreatePaymentResponse {
  paymentIntentId: string;
  clientSecret: string;
  subscriptionId: string;
  status: PaymentStatus;
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