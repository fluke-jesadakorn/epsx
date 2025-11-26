/**
 * FRONTEND - CONSTANTS COMPATIBILITY LAYER
 * Migrated to use consolidated shared/config/constants.ts
 * Provides user-specific constants and backward compatibility
 */

import {
  // Permission templates
  PERMISSION_TEMPLATES,
  
  // Asset definitions (using correct export names)
  SUPPORTED_ASSETS,
  
  // Payment configurations
  MIN_AMOUNT,
  TRANSACTION_STATUSES,
  
  // Blockchain configurations
  BLOCKCHAIN_NETWORKS,
  
  // UI constants
  Z_INDEX_LAYERS,
  
  // Utility functions
  getPermissionTemplate,
  getAssetInfo,
  validatePayment,
  isFeatureEnabled
} from '../../../shared/config/constants';

import type { PermissionTemplateName } from '../../../shared/types/payment';

// Re-export types
export type { PermissionTemplateName };

// Re-export from shared constants for direct use
export {
  PERMISSION_TEMPLATES,
  SUPPORTED_ASSETS,
  MIN_AMOUNT,
  TRANSACTION_STATUSES,
  BLOCKCHAIN_NETWORKS,
  Z_INDEX_LAYERS,
  getPermissionTemplate,
  getAssetInfo,
  validatePayment,
  isFeatureEnabled
};

/**
 * Frontend-specific constants
 * Additional constants used only in frontend application
 */
export const FRONTEND_CONSTANTS = {
  // Frontend-specific UI constants
  USER_UI: {
    // Navigation
    HEADER_HEIGHT: '60px',
    MOBILE_HEADER_HEIGHT: '56px',
    
    // Content
    CONTENT_MAX_WIDTH: '1200px',
    SIDEBAR_WIDTH: '300px',
    
    // Cards and layouts
    CARD_PADDING: '1.5rem',
    SECTION_SPACING: '2rem',
    
    // Animations (keeping as static values per zero-animation policy)
    TRANSITIONS: {
      NONE: 'transition-none',
      INSTANT: 'transition-none',
    },
  },
  
  // User operation limits
  LIMITS: {
    MAX_WATCHLIST_ITEMS: 100,
    MAX_PORTFOLIO_ITEMS: 50,
    MAX_ALERTS: 25,
    MAX_EXPORT_RECORDS: 10000,
    MAX_SEARCH_RESULTS: 100,
  },
  
  // User notification types
  NOTIFICATION_TYPES: [
    'price_alert',
    'portfolio_update',
    'system_maintenance',
    'feature_announcement',
    'payment_confirmation',
    'subscription_update',
    'security_notification',
    'welcome_message'
  ] as const,
  
  // User dashboard metrics
  DASHBOARD_METRICS: {
    PORTFOLIO_STATS: [
      'total_value',
      'daily_change',
      'weekly_change',
      'monthly_change',
      'ytd_change',
      'all_time_high',
      'all_time_low'
    ],
    
    ANALYTICS_STATS: [
      'views_today',
      'exports_month',
      'alerts_active',
      'watchlist_size'
    ],
    
    USAGE_STATS: [
      'sessions_this_week',
      'time_spent_today',
      'features_used',
      'last_login'
    ]
  } as const,
  
  // Progressive authentication levels
  PROGRESSIVE_AUTH: {
    PUBLIC: 'public',
    CONNECTED: 'connected', 
    AUTHENTICATED: 'authenticated'
  } as const,
  
  // Web3 specific constants
  WEB3: {
    SUPPORTED_CHAINS: [56, 97], // BSC Mainnet, BSC Testnet
    DEFAULT_CHAIN: 56, // BSC Mainnet
    
    WALLET_TYPES: [
      'metamask',
      'walletconnect',
      'coinbase',
      'trust',
      'binance'
    ],
    
    CONNECTION_TIMEOUT: 30000, // 30 seconds
    TRANSACTION_TIMEOUT: 120000, // 2 minutes
  } as const,
  
  // User subscription tiers
  SUBSCRIPTION_TIERS: [
    'free',
    'trial', 
    'basic',
    'premium',
    'enterprise'
  ] as const,
  
  // Payment methods for users
  USER_PAYMENT_METHODS: [
    'crypto',
    'credit_card',
    'bank_transfer',
    'paypal'
  ] as const,
} as const;

/**
 * Frontend-specific utility functions
 */

/**
 * Get subscription tier display name
 */
export function getSubscriptionTierDisplayName(tier: typeof FRONTEND_CONSTANTS.SUBSCRIPTION_TIERS[number]): string {
  const tierDisplayNames: Record<string, string> = {
    free: 'Free',
    trial: 'Trial',
    basic: 'Basic',
    premium: 'Premium',
    enterprise: 'Enterprise',
  };
  
  return tierDisplayNames[tier] || tier;
}

/**
 * Get notification type display name
 */
export function getNotificationTypeDisplayName(type: typeof FRONTEND_CONSTANTS.NOTIFICATION_TYPES[number]): string {
  const typeDisplayNames: Record<string, string> = {
    price_alert: 'Price Alert',
    portfolio_update: 'Portfolio Update',
    system_maintenance: 'System Maintenance',
    feature_announcement: 'Feature Announcement', 
    payment_confirmation: 'Payment Confirmation',
    subscription_update: 'Subscription Update',
    security_notification: 'Security Notification',
    welcome_message: 'Welcome Message',
  };
  
  return typeDisplayNames[type] || type;
}

/**
 * Validate subscription tier
 */
export function isValidSubscriptionTier(tier: string): tier is typeof FRONTEND_CONSTANTS.SUBSCRIPTION_TIERS[number] {
  return FRONTEND_CONSTANTS.SUBSCRIPTION_TIERS.includes(tier as any);
}

/**
 * Get user operation limits
 */
export function getUserOperationLimit(operation: string): number {
  const limits: Record<string, number> = {
    watchlist: FRONTEND_CONSTANTS.LIMITS.MAX_WATCHLIST_ITEMS,
    portfolio: FRONTEND_CONSTANTS.LIMITS.MAX_PORTFOLIO_ITEMS,
    alerts: FRONTEND_CONSTANTS.LIMITS.MAX_ALERTS,
    export: FRONTEND_CONSTANTS.LIMITS.MAX_EXPORT_RECORDS,
    search: FRONTEND_CONSTANTS.LIMITS.MAX_SEARCH_RESULTS,
  };
  
  return limits[operation] || 10; // Default limit
}

/**
 * Check if wallet is supported
 */
export function isSupportedWallet(walletType: string): boolean {
  return FRONTEND_CONSTANTS.WEB3.WALLET_TYPES.includes(walletType as any);
}

/**
 * Check if chain is supported
 */
export function isSupportedChain(chainId: number): boolean {
  return FRONTEND_CONSTANTS.WEB3.SUPPORTED_CHAINS.includes(chainId as 56 | 97);
}

/**
 * Get chain name from chain ID
 */
export function getChainName(chainId: number): string {
  const chainNames: Record<number, string> = {
    56: 'BSC Mainnet',
    97: 'BSC Testnet',
  };
  
  return chainNames[chainId] || `Chain ${chainId}`;
}

/**
 * Progressive auth level validation
 */
export function isValidAuthLevel(level: string): level is keyof typeof FRONTEND_CONSTANTS.PROGRESSIVE_AUTH {
  return Object.values(FRONTEND_CONSTANTS.PROGRESSIVE_AUTH).includes(level as any);
}

/**
 * Get auth level priority (higher number = more privileged)
 */
export function getAuthLevelPriority(level: string): number {
  const priorities: Record<string, number> = {
    public: 0,
    connected: 1,
    authenticated: 2,
  };
  
  return priorities[level] || 0;
}

/**
 * Frontend-specific error messages
 */
export const FRONTEND_ERROR_MESSAGES = {
  WALLET_CONNECTION_FAILED: 'Failed to connect to wallet',
  UNSUPPORTED_WALLET: 'Wallet type not supported',
  UNSUPPORTED_CHAIN: 'Blockchain network not supported',
  TRANSACTION_FAILED: 'Transaction failed to complete',
  INSUFFICIENT_BALANCE: 'Insufficient balance for transaction',
  USER_OPERATION_LIMIT_EXCEEDED: 'Operation limit exceeded',
  SUBSCRIPTION_REQUIRED: 'Subscription upgrade required for this feature',
  NETWORK_ERROR: 'Network error occurred',
  AUTH_REQUIRED: 'Authentication required',
  PERMISSION_DENIED: 'Permission denied for this action',
} as const;

// Export combined constants for backward compatibility
export const CONSTANTS = {
  // Shared constants
  ...Z_INDEX_LAYERS,
  PERMISSION_TEMPLATES,
  SUPPORTED_ASSETS,
  MIN_AMOUNT,
  TRANSACTION_STATUSES,
  BLOCKCHAIN_NETWORKS,
  
  // Frontend-specific constants
  ...FRONTEND_CONSTANTS,
  FRONTEND_ERROR_MESSAGES,
} as const;

// Legacy compatibility - export constants as default
export default CONSTANTS;