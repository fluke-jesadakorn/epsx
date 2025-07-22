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
  apiLimits: {
    requestsPerMinute: number;
    requestsPerDay: number;
    maxRankings: number;
    maxFileSize: number;
  };
  duration: number;
  numericLevel: number;
  color: string;
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

export interface PaymentLimits {
  requestsPerMinute: number;
  requestsPerDay: number;
  maxRankings: number;
  maxFileSize: number;
}
