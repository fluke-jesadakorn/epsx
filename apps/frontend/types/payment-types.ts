// Payment and User Level Types

export enum PaymentTier {
  BRONZE = 'BRONZE',
  SILVER = 'SILVER', 
  GOLD = 'GOLD',
  PLATINUM = 'PLATINUM'
}

export interface ApiLimits {
  requestsPerMinute: number;
  requestsPerDay: number;
  maxRankings: number;
  maxFileSize: number;
}

export interface PaymentPlan {
  id: string;
  tier: PaymentTier;
  name: string;
  price: number;
  currency: string;
  features: string[];
  apiLimits: ApiLimits;
  duration: number;
  numericLevel: number;
  color: string;
}

export interface User {
  id: string;
  email: string;
  name?: string;
  level?: string;
  tier?: PaymentTier;
  createdAt?: string;
  updatedAt?: string;
}

export interface PaymentStatus {
  id: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled';
  amount: number;
  currency: string;
  createdAt: string;
  userId?: string;
  tier?: PaymentTier;
}

export interface PaymentTransaction {
  id: string;
  amount: number;
  currency: string;
  status: string;
  createdAt: string;
  description?: string;
  userId?: string;
  tier?: PaymentTier;
}