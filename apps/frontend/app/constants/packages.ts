// Permission Template System - Simplified Configuration
export type PermissionTemplateName =
  | 'Free Template'
  | 'Bronze Template'
  | 'Silver Template'
  | 'Gold Template'
  | 'Platinum Template'
  | 'Enterprise Template';

// Permission Template Configuration
export interface PermissionTemplate {
  name: PermissionTemplateName;
  displayTier: string;
  color: string;
  features: string[];
  permissions: string[];
}

// Permission Templates - Matches backend templates
export const PERMISSION_TEMPLATES: Record<PermissionTemplateName, PermissionTemplate> = {
  'Free Template': {
    name: 'Free Template',
    displayTier: 'FREE',
    color: 'gray-500',
    features: ['Basic access', 'View 3 rankings', 'Community support'],
    permissions: ['epsx:rankings:view:3', 'epsx:analytics:basic', 'epsx:portfolio:view']
  },
  'Bronze Template': {
    name: 'Bronze Template',
    displayTier: 'BRONZE',
    color: 'amber-600',
    features: ['Enhanced access', 'View 5 rankings', 'Basic features'],
    permissions: ['epsx:rankings:view:5', 'epsx:analytics:basic', 'epsx:portfolio:view', 'epsx:portfolio:history']
  },
  'Silver Template': {
    name: 'Silver Template',
    displayTier: 'SILVER',
    color: 'slate-400',
    features: ['Premium access', 'View 25 rankings', 'Advanced analytics'],
    permissions: ['epsx:rankings:view:25', 'epsx:analytics:basic', 'epsx:analytics:advanced', 'epsx:portfolio:view', 'epsx:analytics:basic']
  },
  'Gold Template': {
    name: 'Gold Template',
    displayTier: 'GOLD',
    color: 'yellow-500',
    features: ['Professional access', 'View 50 rankings', 'Premium tools'],
    permissions: ['epsx:rankings:view:50', 'epsx:analytics:premium', 'epsx:portfolio:tools', 'epsx:analytics:advanced']
  },
  'Platinum Template': {
    name: 'Platinum Template',
    displayTier: 'PLATINUM',
    color: 'purple-500',
    features: ['VIP access', 'View 100 rankings', 'Advanced features'],
    permissions: ['epsx:rankings:view:100', 'epsx:analytics:premium', 'epsx:research:reports', 'epsx:dashboards:custom']
  },
  'Enterprise Template': {
    name: 'Enterprise Template',
    displayTier: 'ENTERPRISE',
    color: 'red-500',
    features: ['Unlimited access', 'All platform features', 'Priority support'],
    permissions: ['epsx:rankings:view:unlimited', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*']
  }
};
// Payment System Types (Essential only)
export type CurrencyType = 'USDT' | 'USDT_TRC20' | 'USDT_BSC' | 'USDT_ERC20' | 'USDT_ARB';

export const MIN_AMOUNT = {
  USDT: 1,
  USDT_TRC20: 1,
  USDT_BSC: 1,
  USDT_ERC20: 1,
  USDT_ARB: 1,
} as const;

export type PaymentError =
  | { type: 'INSUFFICIENT_AMOUNT'; minAmount: number; currency: CurrencyType }
  | { type: 'INVALID_CURRENCY' }
  | { type: 'NETWORK_ERROR' }
  | { type: 'TRANSACTION_FAILED'; reason: string };

// Permission Template Helper Functions
export const getTemplateByName = (name: PermissionTemplateName): PermissionTemplate => {
  return PERMISSION_TEMPLATES[name];
};

export const getDisplayTierColor = (displayTier: string): string => {
  const template = Object.values(PERMISSION_TEMPLATES).find(t => t.displayTier === displayTier);
  return template ? `text-${template.color}` : 'text-gray-600';
};

export const getRankingLimitFromPermissions = (permissions: string[]): number => {
  for (const permission of permissions) {
    if (permission.startsWith('epsx:rankings:view:')) {
      const limitStr = permission.split(':')[3];
      if (limitStr === 'unlimited') return -1;
      const limit = parseInt(limitStr, 10);
      if (!isNaN(limit)) return limit;
    }
  }
  return 3; // Default free limit
};

export const getDisplayTierFromPermissions = (permissions: string[]): string => {
  // Find the highest tier based on permissions
  for (const [_templateName, template] of Object.entries(PERMISSION_TEMPLATES)) {
    const hasRequiredPermissions = template.permissions.some(perm =>
      permissions.some(userPerm => {
        // Simple match for now - could be more sophisticated
        return userPerm === perm || userPerm.startsWith(perm.split(':').slice(0, 2).join(':'));
      })
    );

    if (hasRequiredPermissions) {
      return template.displayTier;
    }
  }

  return 'FREE'; // Default tier
};

// Payment Transaction Constants
export const TRANSACTION_STATUSES = {
  PENDING: 'pending',
  COMPLETED: 'completed',
  FAILED: 'failed',
} as const;

// Get network from environment variable
const isMainnet = process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK === 'mainnet';

export const BLOCKCHAIN_CONFIG = {
  BSC: {
    name: isMainnet ? 'BSC Mainnet' : 'BSC Testnet',
    currency: 'USDT_BSC',
    explorerUrl: isMainnet ? 'https://bscscan.com/tx/' : 'https://testnet.bscscan.com/tx/',
    networkId: isMainnet ? '56' : '97',
  },
} as const;

export const validatePayment = (amount: number, currency: CurrencyType): PaymentError | null => {
  if (!Object.keys(MIN_AMOUNT).includes(currency)) {
    return { type: 'INVALID_CURRENCY' };
  }
  if (amount < MIN_AMOUNT[currency as keyof typeof MIN_AMOUNT]) {
    return { type: 'INSUFFICIENT_AMOUNT', minAmount: MIN_AMOUNT[currency as keyof typeof MIN_AMOUNT], currency };
  }
  return null;
};

export type PaymentLoadingState =
  | { state: 'idle' }
  | { state: 'loading' }
  | { state: 'success' }
  | { state: 'error'; error: PaymentError };

// Legacy compatibility exports to maintain existing functionality
export const PACKAGES = Object.values(PERMISSION_TEMPLATES).map(template => ({
  id: template.name.toLowerCase().replace(' ', '_'),
  name: template.name,
  displayTier: template.displayTier,
  color: template.color,
  features: template.features,
  price: 0, // Templates don't have pricing, would need separate pricing logic
  currency: 'USD',
  permissions: template.permissions
}));
