import { PaymentTier, UserSubscription } from './index';

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
