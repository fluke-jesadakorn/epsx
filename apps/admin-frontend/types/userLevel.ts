import { UserSubscription } from './index';

// Permission Template Names for consistent typing
export type PermissionTemplateName = 
  | 'Free Template'
  | 'Bronze Template' 
  | 'Silver Template'
  | 'Gold Template'
  | 'Platinum Template'
  | 'Enterprise Template'
  | 'Admin Template';

export interface PermissionTemplate {
  name: PermissionTemplateName;
  displayTier: string;
  permissions: string[];
  features: string[];
  color?: string;
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
  permissionTemplate: PermissionTemplateName
  permissions: string[]
  subscription?: UserSubscription
}

// Helper function to get display tier from permission template
export function getDisplayTierFromTemplate(templateName: PermissionTemplateName): string {
  const mapping: Record<PermissionTemplateName, string> = {
    'Free Template': 'FREE',
    'Bronze Template': 'BRONZE',
    'Silver Template': 'SILVER', 
    'Gold Template': 'GOLD',
    'Platinum Template': 'PLATINUM',
    'Enterprise Template': 'ENTERPRISE',
    'Admin Template': 'ADMIN'
  };
  return mapping[templateName] || 'FREE';
}

// Helper function to get permissions from template name
export function getPermissionsFromTemplate(templateName: PermissionTemplateName): string[] {
  const templatePermissions: Record<PermissionTemplateName, string[]> = {
    'Free Template': ['epsx:rankings:view:3', 'epsx:trading:basic', 'epsx:portfolio:view'],
    'Bronze Template': ['epsx:rankings:view:5', 'epsx:trading:basic', 'epsx:portfolio:view', 'epsx:portfolio:history'],
    'Silver Template': ['epsx:rankings:view:25', 'epsx:trading:basic', 'epsx:trading:advanced', 'epsx:portfolio:view', 'epsx:analytics:basic'],
    'Gold Template': ['epsx:rankings:view:50', 'epsx:trading:premium', 'epsx:portfolio:tools', 'epsx:analytics:advanced'],
    'Platinum Template': ['epsx:rankings:view:100', 'epsx:trading:premium', 'epsx:analytics:premium', 'epsx:research:reports', 'epsx:dashboards:custom'],
    'Enterprise Template': ['epsx:rankings:view:unlimited', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*'],
    'Admin Template': ['admin:*:*', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*']
  };
  return templatePermissions[templateName] || [];
}
