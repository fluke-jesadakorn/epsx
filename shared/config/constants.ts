/**
 * CONSOLIDATED CONSTANTS
 * Unified constants and configuration shared across admin-frontend and frontend
 * Includes permission templates, asset definitions, payment configurations, and UI constants
 */

import type { AssetInfo, PermissionTemplateName } from '../types/payment';

// ============================================================================
// PERMISSION TEMPLATE SYSTEM
// ============================================================================

export interface PermissionTemplate {
  name: PermissionTemplateName;
  displayTier: string;
  color: string;
  features: string[];
  permissions: string[];
  description: string;
  monthlyPrice?: number;
  yearlyPrice?: number;
  popular?: boolean;
}

export const PERMISSION_TEMPLATES: Record<PermissionTemplateName, PermissionTemplate> = {
  'FREE': {
    name: 'FREE',
    displayTier: 'FREE',
    color: 'gray-500',
    description: 'Basic access for beginners',
    features: [
      'Basic access', 
      'View 3 rankings', 
      'Community support',
      'Basic analytics',
      'Public data access'
    ],
    permissions: [
      'epsx:analytics:view', 
      'epsx:profile:view',
      'epsx:notifications:receive'
    ],
    monthlyPrice: 0,
    yearlyPrice: 0
  },
  
  'BASIC': {
    name: 'BASIC',
    displayTier: 'BASIC',
    color: 'blue-500',
    description: 'Enhanced features for casual users',
    features: [
      'Enhanced access', 
      'View 10 rankings', 
      'Email support',
      'Basic portfolio tracking',
      'Export capabilities'
    ],
    permissions: [
      'epsx:analytics:view', 
      'epsx:analytics:export',
      'epsx:profile:manage', 
      'epsx:notifications:receive',
      'epsx:billing:view'
    ],
    monthlyPrice: 9.99,
    yearlyPrice: 99.99
  },
  
  'PRO': {
    name: 'PRO',
    displayTier: 'PRO', 
    color: 'purple-500',
    description: 'Advanced tools for serious traders',
    features: [
      'Professional access', 
      'View 50 rankings', 
      'Priority support',
      'Advanced analytics',
      'Real-time data',
      'Custom alerts'
    ],
    permissions: [
      'epsx:analytics:view',
      'epsx:analytics:advanced',
      'epsx:analytics:export', 
      'epsx:realtime:access',
      'epsx:profile:manage',
      'epsx:notifications:manage',
      'epsx:billing:manage',
      'epsx:payment:create'
    ],
    monthlyPrice: 29.99,
    yearlyPrice: 299.99,
    popular: true
  },
  
  'ENTERPRISE': {
    name: 'ENTERPRISE',
    displayTier: 'ENTERPRISE',
    color: 'red-500',
    description: 'Complete access for organizations',
    features: [
      'Unlimited access', 
      'All platform features', 
      'Dedicated support',
      'Custom integrations',
      'Team management',
      'API access',
      'White-label options'
    ],
    permissions: [
      'epsx:*:*',
      'epsx-pay:transactions:read',
      'epsx-pay:payments:process'
    ],
    monthlyPrice: 99.99,
    yearlyPrice: 999.99
  },
  
  'WHALE': {
    name: 'WHALE',
    displayTier: 'WHALE',
    color: 'gradient-to-r from-purple-500 to-pink-500',
    description: 'Premium tier for high-volume users',
    features: [
      'Whale-tier access',
      'Unlimited everything',
      'VIP support',
      'Early feature access',
      'Custom development',
      'Direct line to team'
    ],
    permissions: [
      'epsx:*:*',
      'epsx-pay:*:*',
      'epsx-token:*:*'
    ],
    monthlyPrice: 299.99,
    yearlyPrice: 2999.99
  },
  
  'CUSTOM': {
    name: 'CUSTOM',
    displayTier: 'CUSTOM',
    color: 'indigo-500',
    description: 'Tailored solution for specific needs',
    features: [
      'Custom features',
      'Negotiated pricing',
      'Bespoke development',
      'Enterprise SLA',
      'Dedicated account manager'
    ],
    permissions: [], // Defined per customer
  }
} as const;

// ============================================================================
// SUPPORTED CRYPTO ASSETS
// ============================================================================

export const SUPPORTED_ASSETS: AssetInfo[] = [
  {
    currency: 'USDT_TRC20',
    name: 'Tether (TRC20)',
    symbol: 'USDT',
    chain: 'TRX',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: 'Typically 34 characters long with the capital letter "T"',
    contract_address: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'
  },
  {
    currency: 'USDT_ERC20',
    name: 'Tether (ERC20)',
    symbol: 'USDT',
    chain: 'Ethereum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\'',
    contract_address: '0xdAC17F958D2ee523a2206206994597C13D831ec7'
  },
  {
    currency: 'USDT_BSC',
    name: 'Tether (BSC)',
    symbol: 'USDT',
    chain: 'Binance Smart Chain',
    decimals: 18,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\'',
    contract_address: '0x55d398326f99059fF775485246999027B3197955'
  },
  {
    currency: 'USDT_ARB',
    name: 'Tether (Arbitrum)',
    symbol: 'USDT',
    chain: 'Arbitrum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\'',
    contract_address: '0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9'
  },
  {
    currency: 'USDC_ERC20',
    name: 'USD Coin (ERC20)',
    symbol: 'USDC',
    chain: 'Ethereum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\'',
    contract_address: '0xA0b86a33E6441E95C5a7D2fAF8c8d11e8c38CE8D'
  },
  {
    currency: 'USDC_ARB',
    name: 'USD Coin (Arbitrum)',
    symbol: 'USDC',
    chain: 'Arbitrum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\'',
    contract_address: '0xaf88d065e77c8cC2239327C5EDb3A432268e5831'
  }
] as const;

// ============================================================================
// PAYMENT CONFIGURATION
// ============================================================================

export type CurrencyType = 'USDT' | 'USDT_TRC20' | 'USDT_BSC' | 'USDT_ERC20' | 'USDT_ARB' | 'USDC_ERC20' | 'USDC_ARB';

export const MIN_AMOUNT: Record<CurrencyType, number> = {
  USDT: 1,
  USDT_TRC20: 1,
  USDT_BSC: 1,
  USDT_ERC20: 1,
  USDT_ARB: 1,
  USDC_ERC20: 1,
  USDC_ARB: 1,
} as const;

export const TRANSACTION_FEES: Record<CurrencyType, number> = {
  USDT: 0,
  USDT_TRC20: 1, // TRON network fee
  USDT_BSC: 0.01, // BSC network fee
  USDT_ERC20: 5, // Ethereum network fee (higher)
  USDT_ARB: 0.1, // Arbitrum network fee
  USDC_ERC20: 5, // Ethereum network fee
  USDC_ARB: 0.1, // Arbitrum network fee
} as const;

export const TRANSACTION_STATUSES = {
  PENDING: 'pending',
  PROCESSING: 'processing',
  COMPLETED: 'completed',
  SUCCEEDED: 'succeeded',
  FAILED: 'failed',
  CANCELLED: 'cancelled',
  EXPIRED: 'expired',
  REQUIRES_ACTION: 'RequiresAction',
} as const;

export type PaymentStatus = typeof TRANSACTION_STATUSES[keyof typeof TRANSACTION_STATUSES];

// ============================================================================
// BLOCKCHAIN CONFIGURATION
// ============================================================================

export interface BlockchainNetwork {
  name: string;
  networkId: string;
  chainId: number;
  currency: CurrencyType;
  explorerUrl: string;
  rpcUrl?: string;
  testnet?: boolean;
}

export const BLOCKCHAIN_NETWORKS: Record<string, BlockchainNetwork> = {
  BSC_MAINNET: {
    name: 'BSC Mainnet',
    networkId: '56',
    chainId: 56,
    currency: 'USDT_BSC',
    explorerUrl: 'https://bscscan.com/tx/',
    rpcUrl: 'https://bsc-dataseed1.binance.org/',
  },
  BSC_TESTNET: {
    name: 'BSC Testnet',
    networkId: '97',
    chainId: 97,
    currency: 'USDT_BSC',
    explorerUrl: 'https://testnet.bscscan.com/tx/',
    rpcUrl: 'https://data-seed-prebsc-1-s1.binance.org:8545/',
    testnet: true,
  },
  ETHEREUM_MAINNET: {
    name: 'Ethereum Mainnet',
    networkId: '1',
    chainId: 1,
    currency: 'USDT_ERC20',
    explorerUrl: 'https://etherscan.io/tx/',
    rpcUrl: 'https://eth-mainnet.alchemyapi.io/v2/',
  },
  ARBITRUM_ONE: {
    name: 'Arbitrum One',
    networkId: '42161',
    chainId: 42161,
    currency: 'USDT_ARB',
    explorerUrl: 'https://arbiscan.io/tx/',
    rpcUrl: 'https://arb1.arbitrum.io/rpc',
  },
  TRON_MAINNET: {
    name: 'TRON Mainnet',
    networkId: '728126428',
    chainId: 728126428,
    currency: 'USDT_TRC20',
    explorerUrl: 'https://tronscan.org/#/transaction/',
  },
} as const;

// Get network configuration based on environment
export function getBlockchainConfig(): BlockchainNetwork {
  const isMainnet = process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK === 'mainnet';
  return isMainnet ? BLOCKCHAIN_NETWORKS.BSC_MAINNET : BLOCKCHAIN_NETWORKS.BSC_TESTNET;
}

// ============================================================================
// UI CONSTANTS
// ============================================================================

export const Z_INDEX_LAYERS = {
  // Base content layer: z-0 to z-10
  BASE: {
    CONTENT: 'z-0',
    ELEVATED_CONTENT: 'z-10',
  },
  
  // Dropdowns and tooltips: z-[9999] to z-[10000]
  DROPDOWNS: {
    DROPDOWN: 'z-[10000]',
    TOOLTIP: 'z-[9999]',
  },
  
  // Sidebar and navigation overlays: z-40 to z-50
  NAVIGATION: {
    OVERLAY_BACKDROP: 'z-40',  // Mobile sidebar backdrop
    SIDEBAR: 'z-50',           // Sidebar panels and navigation bars
  },
  
  // Modal dialogs and overlays: z-60 to z-70
  MODALS: {
    MODAL_BACKDROP: 'z-60',    // Modal overlay backgrounds
    MODAL_CONTENT: 'z-70',     // Modal content (if needed to stack above backdrop)
  },
  
  // Toast notifications: z-80 to z-90
  NOTIFICATIONS: {
    TOAST: 'z-80',             // Toast notification containers
    CRITICAL_TOAST: 'z-90',    // Critical/urgent notifications
  },
} as const;

export const ANIMATION_DURATIONS = {
  INSTANT: '0ms',
  FAST: '150ms',
  NORMAL: '250ms',
  SLOW: '350ms',
  SLOWER: '500ms',
} as const;

export const BREAKPOINTS = {
  SM: '640px',
  MD: '768px',
  LG: '1024px',
  XL: '1280px',
  '2XL': '1536px',
} as const;

// ============================================================================
// FEATURE FLAGS
// ============================================================================

export interface FeatureFlagConfig {
  name: string;
  description: string;
  defaultValue: boolean;
  environments?: ('development' | 'staging' | 'production')[];
  rolloutPercentage?: number;
}

export const FEATURE_FLAGS: Record<string, FeatureFlagConfig> = {
  UNIFIED_USER_MANAGEMENT: {
    name: 'Unified User Management',
    description: 'Enable the new unified user management system',
    defaultValue: true,
    environments: ['development', 'staging'],
  },
  
  SERVER_COMPONENTS: {
    name: 'Server Components',
    description: 'Enable Next.js Server Components where applicable',
    defaultValue: true,
  },
  
  NEW_NAVIGATION: {
    name: 'New Navigation',
    description: 'Enable the redesigned navigation system',
    defaultValue: false,
    environments: ['development'],
  },
  
  PROGRESSIVE_AUTH: {
    name: 'Progressive Authentication',
    description: 'Enable progressive authentication (PUBLIC → CONNECTED → AUTHENTICATED)',
    defaultValue: true,
  },
  
  WEB3_INTEGRATION: {
    name: 'Web3 Integration',
    description: 'Enable Web3 wallet connection and blockchain features',
    defaultValue: true,
  },
  
  ADMIN_ENHANCED_PERMISSIONS: {
    name: 'Admin Enhanced Permissions',
    description: 'Enable enhanced permission management for admins',
    defaultValue: true,
    environments: ['development', 'staging'],
  },
  
  REAL_TIME_UPDATES: {
    name: 'Real-time Updates',
    description: 'Enable real-time data updates via WebSocket',
    defaultValue: false,
    rolloutPercentage: 50,
  },
} as const;

// ============================================================================
// API ENDPOINTS CONFIGURATION
// ============================================================================

export const API_ENDPOINTS = {
  // Authentication endpoints
  AUTH: {
    CHALLENGE: '/api/v1/auth/challenge',
    VERIFY: '/api/v1/auth/verify',
    LOGOUT: '/api/v1/auth/logout',
    REFRESH: '/api/v1/auth/refresh',
    PERMISSIONS: '/api/v1/auth/permissions',
  },
  
  // User endpoints
  USER: {
    PROFILE: '/api/v1/user/profile',
    PREFERENCES: '/api/v1/user/preferences',
    SUBSCRIPTION: '/api/v1/user/subscription',
    NOTIFICATIONS: '/api/v1/user/notifications',
  },
  
  // Payment endpoints
  PAYMENT: {
    CREATE: '/api/v1/payment/create',
    STATUS: '/api/v1/payment/status',
    HISTORY: '/api/v1/payment/history',
    ASSETS: '/api/v1/payment/assets',
  },
  
  // Analytics endpoints
  ANALYTICS: {
    RANKINGS: '/api/v1/analytics/rankings',
    EXPORT: '/api/v1/analytics/export',
    ADVANCED: '/api/v1/analytics/advanced',
    REAL_TIME: '/api/v1/analytics/realtime',
  },
  
  // Admin endpoints
  ADMIN: {
    USERS: '/api/admin/users',
    PERMISSIONS: '/api/admin/permissions',
    SYSTEM: '/api/admin/system',
    ANALYTICS: '/api/admin/analytics',
    NOTIFICATIONS: '/api/admin/notifications',
    AUDIT: '/api/admin/audit',
  },
} as const;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get permission template by name
 */
export function getPermissionTemplate(name: PermissionTemplateName): PermissionTemplate {
  return PERMISSION_TEMPLATES[name];
}

/**
 * Get template by display tier
 */
export function getTemplateByTier(displayTier: string): PermissionTemplate | undefined {
  return Object.values(PERMISSION_TEMPLATES).find(t => t.displayTier === displayTier);
}

/**
 * Get color class for display tier
 */
export function getDisplayTierColor(displayTier: string): string {
  const template = getTemplateByTier(displayTier);
  return template ? `text-${template.color}` : 'text-gray-600';
}

/**
 * Validate payment amount and currency
 */
export function validatePayment(amount: number, currency: CurrencyType): { valid: boolean; error?: string } {
  if (!Object.keys(MIN_AMOUNT).includes(currency)) {
    return { valid: false, error: 'Invalid currency' };
  }
  
  const minAmount = MIN_AMOUNT[currency];
  if (amount < minAmount) {
    return { valid: false, error: `Minimum amount is ${minAmount} ${currency}` };
  }
  
  return { valid: true };
}

/**
 * Get asset info by currency
 */
export function getAssetInfo(currency: CurrencyType): AssetInfo | undefined {
  return SUPPORTED_ASSETS.find(asset => asset.currency === currency);
}

/**
 * Check if feature flag is enabled
 */
export function isFeatureEnabled(flag: keyof typeof FEATURE_FLAGS): boolean {
  const config = FEATURE_FLAGS[flag];
  
  // Check environment restriction
  if (config.environments) {
    const currentEnv = process.env.NODE_ENV as 'development' | 'staging' | 'production';
    if (!config.environments.includes(currentEnv)) {
      return false;
    }
  }
  
  // Check rollout percentage (simple implementation)
  if (config.rolloutPercentage !== undefined) {
    // In a real implementation, this would use user ID or session ID for consistent rollout
    return Math.random() * 100 < config.rolloutPercentage;
  }
  
  // Check environment variable override
  const envVar = `NEXT_PUBLIC_ENABLE_${flag}`;
  const envValue = process.env[envVar];
  if (envValue !== undefined) {
    return envValue.toLowerCase() === 'true';
  }
  
  return config.defaultValue;
}

/**
 * Get blockchain explorer URL for transaction
 */
export function getExplorerUrl(txHash: string, currency: CurrencyType): string {
  const asset = getAssetInfo(currency);
  const network = getBlockchainConfig();
  
  if (currency.includes('TRC20')) {
    return `${BLOCKCHAIN_NETWORKS.TRON_MAINNET.explorerUrl}${txHash}`;
  }
  
  return `${network.explorerUrl}${txHash}`;
}