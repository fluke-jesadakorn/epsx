// User Level Types
export type UserLevelType =
  | 'BASIC'
  | 'SILVER'
  | 'GOLD'
  | 'PLATINUM'
  | 'API_PERSONAL'
  | 'API_COMPANY'
  | 'API_PARTNER';

// Level Configuration - Centralized level definitions
export interface LevelConfig {
  name: string;
  level: UserLevelType;
  numericLevel: number;
  rankingLimit: number;
  minPayments: number;
  color: string;
  features: string[];
}

// Core Level Definitions - Easy to extend
export const LEVEL_CONFIGS: Record<UserLevelType, LevelConfig> = {
  BASIC: {
    name: 'Basic',
    level: 'BASIC',
    numericLevel: 0,
    rankingLimit: 5,
    minPayments: 0,
    color: 'gray-500',
    features: ['Limited access', 'Basic features', 'Community support'],
  },
  SILVER: {
    name: 'Silver',
    level: 'SILVER',
    numericLevel: 1,
    rankingLimit: 25,
    minPayments: 1,
    color: 'slate-400',
    features: ['Full access for 1 month', 'Priority support', 'Advanced features'],
  },
  GOLD: {
    name: 'Gold',
    level: 'GOLD',
    numericLevel: 2,
    rankingLimit: 50,
    minPayments: 3,
    color: 'yellow-500',
    features: ['Extended access', 'Premium features', 'Priority support', 'Early access to new features'],
  },
  PLATINUM: {
    name: 'Platinum',
    level: 'PLATINUM',
    numericLevel: 3,
    rankingLimit: 100,
    minPayments: 6,
    color: 'purple-500',
    features: ['Unlimited access', 'All premium features', 'VIP support', 'Early access to new features', 'Custom analytics'],
  },
  API_PERSONAL: {
    name: 'API Personal',
    level: 'API_PERSONAL',
    numericLevel: 4,
    rankingLimit: 25,
    minPayments: 1,
    color: 'indigo-500',
    features: ['25 Data sets', 'Country Selection', 'Unlimited Accounts'],
  },
  API_COMPANY: {
    name: 'API Company',
    level: 'API_COMPANY',
    numericLevel: 5,
    rankingLimit: 100,
    minPayments: 1,
    color: 'blue-600',
    features: ['100 Data sets', 'Country Selection', 'Unlimited Accounts', 'Priority Support'],
  },
  API_PARTNER: {
    name: 'API Partner',
    level: 'API_PARTNER',
    numericLevel: 6,
    rankingLimit: 100,
    minPayments: 0,
    color: 'purple-600',
    features: ['100 Data sets', 'Country Selection', 'Industry Selection', '15% Revenue Share', 'Unlimited Accounts', 'Custom Integration'],
  },
} as const;
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

// Helper Functions for Level-based Logic
export const getLevelConfig = (level: UserLevelType): LevelConfig => {
  return LEVEL_CONFIGS[level];
};

export const getRankingLimitByLevel = (level: UserLevelType): number => {
  return LEVEL_CONFIGS[level].rankingLimit;
};

export const getNumericLevelByLevel = (level: UserLevelType): number => {
  return LEVEL_CONFIGS[level].numericLevel;
};

export const getLevelByNumeric = (numericLevel: number): UserLevelType => {
  const level = Object.values(LEVEL_CONFIGS).find(config => config.numericLevel === numericLevel);
  return level?.level || 'BASIC';
};

export const canAccessLevel = (currentLevel: UserLevelType, requiredLevel: UserLevelType): boolean => {
  return LEVEL_CONFIGS[currentLevel].numericLevel >= LEVEL_CONFIGS[requiredLevel].numericLevel;
};

export const getLevelColor = (level: UserLevelType): string => {
  const colorMap: Record<UserLevelType, string> = {
    BASIC: 'text-gray-600',
    SILVER: 'text-blue-600',
    GOLD: 'text-yellow-600',
    PLATINUM: 'text-purple-600',
    API_PERSONAL: 'text-indigo-600',
    API_COMPANY: 'text-blue-700',
    API_PARTNER: 'text-purple-700',
  };
  return colorMap[level] || 'text-gray-600';
};

export const getNextLevelLimit = (currentLevel: UserLevelType): number => {
  const currentNumeric = LEVEL_CONFIGS[currentLevel].numericLevel;
  const nextLevel = Object.values(LEVEL_CONFIGS).find(
    config => config.numericLevel === currentNumeric + 1
  );
  return nextLevel?.rankingLimit || LEVEL_CONFIGS.PLATINUM.rankingLimit;
};

export const getLockedRankings = (userLevel: UserLevelType): number => {
  // Calculate how many rankings should appear locked/blurred
  const maxRankings = getRankingLimitByLevel(userLevel);
  const nextLevelLimit = getNextLevelLimit(userLevel);
  return Math.min(nextLevelLimit - maxRankings, 50); // Cap at 50 locked rankings
};

// Level Requirements - Now derived from LEVEL_CONFIGS
export const LEVEL_REQUIREMENTS = Object.fromEntries(
  Object.entries(LEVEL_CONFIGS).map(([key, config]) => [
    key,
    { minPayments: config.minPayments, color: config.color.split('-')[0] }
  ])
) as Record<UserLevelType, { minPayments: number; color: string }>;

// Available Packages
export const PACKAGES: Package[] = [
  // Personal Plans
  {
    id: 'basic',
    name: 'Basic Plan',
    level: 'BASIC',
    numericLevel: LEVEL_CONFIGS.BASIC.numericLevel,
    rankingLimit: LEVEL_CONFIGS.BASIC.rankingLimit,
    price: 0,
    currency: 'USDT',
    features: LEVEL_CONFIGS.BASIC.features,
    minPayments: LEVEL_CONFIGS.BASIC.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.BASIC.color,
    icon: '/icons/basic.svg',
  },
  {
    id: 'silver',
    name: 'Silver Plan',
    level: 'SILVER',
    numericLevel: LEVEL_CONFIGS.SILVER.numericLevel,
    rankingLimit: LEVEL_CONFIGS.SILVER.rankingLimit,
    price: 1,
    currency: 'USDT',
    features: LEVEL_CONFIGS.SILVER.features,
    minPayments: LEVEL_CONFIGS.SILVER.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.SILVER.color,
    icon: '/icons/silver.svg',
  },
  {
    id: 'gold',
    name: 'Gold Plan',
    level: 'GOLD',
    numericLevel: LEVEL_CONFIGS.GOLD.numericLevel,
    rankingLimit: LEVEL_CONFIGS.GOLD.rankingLimit,
    price: 9.9,
    currency: 'USDT',
    features: LEVEL_CONFIGS.GOLD.features,
    minPayments: LEVEL_CONFIGS.GOLD.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.GOLD.color,
    icon: '/icons/gold.svg',
  },
  {
    id: 'platinum',
    name: 'Platinum Plan',
    level: 'PLATINUM',
    numericLevel: LEVEL_CONFIGS.PLATINUM.numericLevel,
    rankingLimit: LEVEL_CONFIGS.PLATINUM.rankingLimit,
    price: 9.9,
    currency: 'USDT',
    features: LEVEL_CONFIGS.PLATINUM.features,
    minPayments: LEVEL_CONFIGS.PLATINUM.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.PLATINUM.color,
    icon: '/icons/platinum.svg',
  },
  // API Plans
  {
    id: 'api_personal',
    name: 'API Personal',
    level: 'API_PERSONAL',
    numericLevel: LEVEL_CONFIGS.API_PERSONAL.numericLevel,
    rankingLimit: LEVEL_CONFIGS.API_PERSONAL.rankingLimit,
    price: 999,
    currency: 'USDT',
    features: LEVEL_CONFIGS.API_PERSONAL.features,
    minPayments: LEVEL_CONFIGS.API_PERSONAL.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.API_PERSONAL.color,
    icon: '/icons/api.svg',
  },
  {
    id: 'api_company',
    name: 'API Company',
    level: 'API_COMPANY',
    numericLevel: LEVEL_CONFIGS.API_COMPANY.numericLevel,
    rankingLimit: LEVEL_CONFIGS.API_COMPANY.rankingLimit,
    price: 2999,
    currency: 'USDT',
    features: LEVEL_CONFIGS.API_COMPANY.features,
    minPayments: LEVEL_CONFIGS.API_COMPANY.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.API_COMPANY.color,
    icon: '/icons/enterprise.svg',
  },
  {
    id: 'api_partner',
    name: 'API Partner',
    level: 'API_PARTNER',
    numericLevel: LEVEL_CONFIGS.API_PARTNER.numericLevel,
    rankingLimit: LEVEL_CONFIGS.API_PARTNER.rankingLimit,
    price: 0,
    currency: 'USDT',
    features: LEVEL_CONFIGS.API_PARTNER.features,
    minPayments: LEVEL_CONFIGS.API_PARTNER.minPayments,
    duration: 1,
    color: LEVEL_CONFIGS.API_PARTNER.color,
    icon: '/icons/partner.svg',
  },
] as const;

// Helper Functions
export const getUserLevel = (paymentCount: number): UserLevelType => {
  // Sort levels by minPayments descending to check highest levels first
  const sortedLevels = Object.entries(LEVEL_CONFIGS)
    .sort(([, a], [, b]) => b.minPayments - a.minPayments);
  
  for (const [, config] of sortedLevels) {
    if (paymentCount >= config.minPayments) {
      return config.level;
    }
  }
  
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

// User Level Benefits - Now derived from LEVEL_CONFIGS
export const LEVEL_BENEFITS: Record<UserLevelType, readonly string[]> = {
  BASIC: LEVEL_CONFIGS.BASIC.features,
  SILVER: LEVEL_CONFIGS.SILVER.features,
  GOLD: LEVEL_CONFIGS.GOLD.features,
  PLATINUM: LEVEL_CONFIGS.PLATINUM.features,
  API_PERSONAL: LEVEL_CONFIGS.API_PERSONAL.features,
  API_COMPANY: LEVEL_CONFIGS.API_COMPANY.features,
  API_PARTNER: LEVEL_CONFIGS.API_PARTNER.features,
} as const;
