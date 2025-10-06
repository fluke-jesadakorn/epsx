import { PermissionTemplateName } from '@/app/constants/packages';

// Re-export the type for external use
export { PermissionTemplateName };

// Permission-based User Subscription
export interface UserSubscription {
  id: string;
  userId: string;
  permissionTemplate: PermissionTemplateName;
  permissions: string[];
  displayTier: string;
  status: 'active' | 'expired' | 'cancelled';
  startDate: string;
  endDate: string;
  autoRenew: boolean;
}

export interface PaymentStatus {
  lastPaymentDate: Date;
  expirationDate: Date;
  paymentMethod: 'USDT';
  transactionId: string;
  amount: number;
}

export interface USDTDetails {
  network: 'ERC20' | 'TRC20' | 'BEP20' | 'Arbitrum' | 'TON';
  walletAddress: string;
  qrCodePath?: string;
  tag?: string;
  paymentStatus: PaymentStatus;
  subscription?: UserSubscription;
}
