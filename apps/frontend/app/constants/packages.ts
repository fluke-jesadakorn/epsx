// User Level Types
export type UserLevelType =
  | 'BASIC'
  | 'SILVER'
  | 'GOLD'
  | 'PLATINUM'
  | 'API_PERSONAL'
  | 'API_COMPANY'
  | 'API_PARTNER';
export type CurrencyType =
  | 'USDT'
  | 'USDT_TRC20'
  | 'USDT_BSC'
  | 'USDT_ERC20'
  | 'USDT_ARB';

// Price validation
export const MIN_AMOUNT = {
  USDT: 1,
  USDT_TRC20: 1,
  USDT_BSC: 1,
  USDT_ERC20: 1,
  USDT_ARB: 1,
} as const;

// Payment error types
export type PaymentError =
  | { type: 'INSUFFICIENT_AMOUNT'; minAmount: number; currency: CurrencyType }
  | { type: 'INVALID_CURRENCY' }
  | { type: 'NETWORK_ERROR' }
  | { type: 'TRANSACTION_FAILED'; reason: string };

// Package Configuration
export interface Package {
  id: string;
  name: string;
  level: UserLevelType;
  numericLevel: number; // numeric user level for access control
  rankingLimit: number; // max ranking stocks user can see
  price: number;
  currency: string;
  features: string[];
  minPayments: number;
  duration: number; // in months
  color: string; // for UI theming
  icon: string; // path to icon
}

// Level Requirements
export const LEVEL_REQUIREMENTS = {
  BASIC: { minPayments: 0, color: 'gray' },
  SILVER: { minPayments: 1, color: 'silver' },
  GOLD: { minPayments: 3, color: 'gold' },
  PLATINUM: { minPayments: 6, color: 'purple' },
} as const;

// Available Packages
export const PACKAGES: Package[] = [
  // Personal Plans
  {
    id: 'basic',
    name: 'Basic Plan',
    level: 'BASIC',
    numericLevel: 0,
    rankingLimit: 5,
    price: 0,
    currency: 'USDT',
    features: ['Limited access', 'Basic features', 'Community support'],
    minPayments: 0,
    duration: 1,
    color: 'gray-500',
    icon: '/icons/basic.svg',
  },
  {
    id: 'silver',
    name: 'Silver Plan',
    level: 'SILVER',
    numericLevel: 1,
    rankingLimit: 25,
    price: 1,
    currency: 'USDT',
    features: [
      'Full access for 1 month',
      'Priority support',
      'Advanced features',
    ],
    minPayments: 1,
    duration: 1,
    color: 'slate-400',
    icon: '/icons/silver.svg',
  },
  {
    id: 'gold',
    name: 'Gold Plan',
    level: 'GOLD',
    numericLevel: 2,
    rankingLimit: 50,
    price: 9.9,
    currency: 'USDT',
    features: [
      'Extended access',
      'Premium features',
      'Priority support',
      'Early access to new features',
    ],
    minPayments: 3,
    duration: 1,
    color: 'yellow-500',
    icon: '/icons/gold.svg',
  },
  {
    id: 'platinum',
    name: 'Platinum Plan',
    level: 'PLATINUM',
    numericLevel: 3,
    rankingLimit: 100,
    price: 9.9,
    currency: 'USDT',
    features: [
      'Unlimited access',
      'All premium features',
      'VIP support',
      'Early access to new features',
      'Custom analytics',
    ],
    minPayments: 6,
    duration: 1,
    color: 'purple-500',
    icon: '/icons/platinum.svg',
  },
  // API Plans
  {
    id: 'api_personal',
    name: 'API Personal',
    level: 'API_PERSONAL',
    numericLevel: 4,
    rankingLimit: 25,
    price: 999,
    currency: 'USDT',
    features: ['25 Data sets', 'Country Selection', 'Unlimited Accounts'],
    minPayments: 1,
    duration: 1,
    color: 'indigo-500',
    icon: '/icons/api.svg',
  },
  {
    id: 'api_company',
    name: 'API Company',
    level: 'API_COMPANY',
    numericLevel: 5,
    rankingLimit: 100,
    price: 2999,
    currency: 'USDT',
    features: [
      '100 Data sets',
      'Country Selection',
      'Unlimited Accounts',
      'Priority Support',
    ],
    minPayments: 1,
    duration: 1,
    color: 'blue-600',
    icon: '/icons/enterprise.svg',
  },
  {
    id: 'api_partner',
    name: 'API Partner',
    level: 'API_PARTNER',
    numericLevel: 6,
    rankingLimit: 100,
    price: 0,
    currency: 'USDT',
    features: [
      '100 Data sets',
      'Country Selection',
      'Industry Selection',
      '15% Revenue Share',
      'Unlimited Accounts',
      'Custom Integration',
    ],
    minPayments: 0,
    duration: 1,
    color: 'purple-600',
    icon: '/icons/partner.svg',
  },
] as const;

// Helper Functions
export const getUserLevel = (paymentCount: number): UserLevelType => {
  if (paymentCount >= LEVEL_REQUIREMENTS.PLATINUM.minPayments)
    return 'PLATINUM';
  if (paymentCount >= LEVEL_REQUIREMENTS.GOLD.minPayments) return 'GOLD';
  if (paymentCount >= LEVEL_REQUIREMENTS.SILVER.minPayments) return 'SILVER';
  return 'BASIC';
};

export const getPackageById = (id: string): Package | undefined => {
  return PACKAGES.find((pkg) => pkg.id === id);
};

export const getPackageByLevel = (
  level: UserLevelType,
): Package | undefined => {
  return PACKAGES.find((pkg) => pkg.level === level);
};

// Transaction Related Constants
export const TRANSACTION_STATUSES = {
  PENDING: 'pending',
  COMPLETED: 'completed',
  FAILED: 'failed',
} as const;

export const BSC_EXPLORER_URL = 'https://bscscan.com/tx/';

// Payment Duration Constants
export const PAYMENT_DURATION = {
  MONTHS: 1,
  MILLISECONDS: 30 * 24 * 60 * 60 * 1000, // 30 days in milliseconds
} as const;

// Blockchain Network Configuration
export const BLOCKCHAIN_CONFIG = {
  BSC: {
    name: 'BSC',
    currency: 'USDT_BSC',
    explorerUrl: 'https://bscscan.com/tx/',
    networkId: '56',
  },
} as const;

// Validation function for payment amount
export const validatePayment = (
  amount: number,
  currency: CurrencyType,
): PaymentError | null => {
  if (!Object.keys(MIN_AMOUNT).includes(currency)) {
    return { type: 'INVALID_CURRENCY' };
  }

  if (amount < MIN_AMOUNT[currency as keyof typeof MIN_AMOUNT]) {
    return {
      type: 'INSUFFICIENT_AMOUNT',
      minAmount: MIN_AMOUNT[currency as keyof typeof MIN_AMOUNT],
      currency,
    };
  }

  return null;
};

// Loading states for components
export type PaymentLoadingState =
  | { state: 'idle' }
  | { state: 'loading' }
  | { state: 'success' }
  | { state: 'error'; error: PaymentError };

// User Level Benefits
export const LEVEL_BENEFITS: Record<UserLevelType, readonly string[]> = {
  BASIC: ['Limited access to features', 'Community support', 'Basic analytics'],
  SILVER: [
    'Full access for 1 month',
    'Priority support',
    'Advanced features',
    'Standard analytics',
  ],
  GOLD: [
    'Extended access',
    'Premium features',
    'Priority support',
    'Early access to new features',
    'Advanced analytics',
  ],
  PLATINUM: [
    'Unlimited access',
    'All premium features',
    'VIP support',
    'Early access to new features',
    'Custom analytics',
    'Dedicated account manager',
  ],
  API_PERSONAL: [
    '25 Data sets',
    'Country Selection',
    'Unlimited Accounts',
    'Developer Support',
    'Testing Environment',
  ],
  API_COMPANY: [
    '100 Data sets',
    'Country Selection',
    'Unlimited Accounts',
    'Priority Support',
    'Custom Features',
    'Enterprise SLA',
  ],
  API_PARTNER: [
    '100 Data sets',
    'Country Selection',
    'Industry Selection',
    '15% Revenue Share',
    'Custom Integration',
    'Dedicated Support',
    'White Label Option',
  ],
} as const;
