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

// Permission-based Helper Functions
export function getDisplayTierFromPermissions(permissions: string[]): string {
  for (const permission of permissions) {
    if (permission.startsWith('epsx:rankings:view:')) {
      const limitStr = permission.split(':')[3];
      if (limitStr === 'unlimited') return 'ENTERPRISE';
      const limit = parseInt(limitStr, 10);
      if (limit >= 100) return 'PLATINUM';
      if (limit >= 50) return 'GOLD'; 
      if (limit >= 25) return 'SILVER';
      if (limit >= 5) return 'BRONZE';
    }
  }
  if (permissions.some(p => p.includes('admin:'))) return 'ENTERPRISE';
  return 'FREE';
}

export function getUserRoleFromPermissions(permissions: string[]): 'admin' | 'user' | 'guest' {
  if (permissions.some(p => p.startsWith('admin:') || p === 'admin:*:*')) {
    return 'admin';
  }
  if (permissions.some(p => p.startsWith('epsx:') && !p.includes(':view:3'))) {
    return 'user';
  }
  return 'guest';
}

// Additional missing exports that were referenced
export interface PermissionTemplate {
  id: string;
  name: PermissionTemplateName;
  permissions: string[];
  displayTier: string;
  price: number;
}

export function getPermissionTemplateByName(name: PermissionTemplateName): PermissionTemplate | null {
  // Implementation placeholder
  return null;
}

export const PERMISSION_TEMPLATES: PermissionTemplate[] = [];
