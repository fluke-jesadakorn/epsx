import { PaymentTier } from '@/types/payment-types';

export interface UserSubscription {
  id: string;
  userId: string;
  tier: PaymentTier;
  status: 'active' | 'expired' | 'cancelled';
  startDate: string;
  endDate: string;
  autoRenew: boolean;
}

// Legacy enum - keep for backward compatibility
export enum UserLevel {
  Basic = 'Basic',
  Premium = 'Premium',
  VIP = 'VIP'
}

export interface PaymentStatus {
  lastPaymentDate: Date
  expirationDate: Date
  paymentMethod: 'USDT'
  transactionId: string
  amount: number
}

export interface USDTDetails {
  network: 'ERC20' | 'TRC20' | 'BEP20' | 'Arbitrum' | 'TON'
  walletAddress: string
  qrCodePath?: string
  tag?: string
  paymentStatus: PaymentStatus
  userLevel: UserLevel // Legacy field
  paymentTier?: PaymentTier // New payment tier field
  subscription?: UserSubscription // New subscription field
}

// Helper function to convert legacy UserLevel to PaymentTier
export function convertUserLevelToPaymentTier(userLevel: UserLevel): PaymentTier {
  const mapping = {
    [UserLevel.Basic]: PaymentTier.BRONZE,
    [UserLevel.Premium]: PaymentTier.SILVER,
    [UserLevel.VIP]: PaymentTier.GOLD
  };
  return mapping[userLevel] || PaymentTier.BRONZE;
}

// Helper function to convert PaymentTier to legacy UserLevel
export function convertPaymentTierToUserLevel(paymentTier: PaymentTier): UserLevel {
  const mapping = {
    [PaymentTier.BRONZE]: UserLevel.Basic,
    [PaymentTier.SILVER]: UserLevel.Premium,
    [PaymentTier.GOLD]: UserLevel.VIP,
    [PaymentTier.PLATINUM]: UserLevel.VIP
  };
  return mapping[paymentTier] || UserLevel.Basic;
}

// Simple role conversion functions for new system
export function convertUserLevelToRole(userLevel: UserLevel): 'admin' | 'user' | 'guest' {
  switch (userLevel) {
    case UserLevel.VIP:
      return 'user'; // VIP users get premium features (user role)
    case UserLevel.Premium:
      return 'user'; // Premium users get premium features (user role)
    case UserLevel.Basic:
      return 'guest'; // Basic users get limited access (guest role)
    default:
      return 'guest';
  }
}

export function convertPaymentTierToRole(paymentTier: PaymentTier): 'admin' | 'user' | 'guest' {
  switch (paymentTier) {
    case PaymentTier.PLATINUM:
    case PaymentTier.GOLD:
    case PaymentTier.SILVER:
    case PaymentTier.BRONZE:
      return 'user'; // All paid tiers get user role (premium features)
    default:
      return 'guest'; // Free tier gets guest role (basic features)
  }
}

export function convertRoleToPaymentTier(role: 'admin' | 'user' | 'guest'): PaymentTier {
  switch (role) {
    case 'admin':
      return PaymentTier.PLATINUM; // Admin gets highest tier
    case 'user':
      return PaymentTier.SILVER; // User gets premium tier
    case 'guest':
      return PaymentTier.BRONZE; // Guest gets basic tier
    default:
      return PaymentTier.BRONZE;
  }
}
