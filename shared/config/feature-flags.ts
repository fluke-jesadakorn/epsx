/**
 * CONSOLIDATED FEATURE FLAG SYSTEM
 * Unified feature flag management shared across admin-frontend and frontend
 * Supports environment-based rollouts, percentage-based rollouts, and user-based targeting
 */

// ============================================================================
// FEATURE FLAG CONFIGURATION TYPES
// ============================================================================

export interface FeatureFlagConfig {
  name: string;
  description: string;
  defaultValue: boolean;
  environments?: ('development' | 'staging' | 'production')[];
  rolloutPercentage?: number;
  enabledForAdmin?: boolean;
  enabledForUsers?: boolean;
  requiredPermissions?: string[];
  deprecatedAt?: string;
  removedAt?: string;
}

export interface FeatureFlagContext {
  userId?: string;
  userPermissions?: string[];
  isAdmin?: boolean;
  environment?: 'development' | 'staging' | 'production';
  userAgent?: string;
}

// ============================================================================
// CORE FEATURE FLAGS
// ============================================================================

export const FEATURE_FLAGS: Record<string, FeatureFlagConfig> = {
  // ============================================================================
  // UI/UX Features
  // ============================================================================

  NEW_NAVIGATION: {
    name: 'New Navigation',
    description: 'Enable the redesigned navigation system with improved UX',
    defaultValue: true,
    environments: ['development', 'staging'],
    rolloutPercentage: 50,
  },

  DARK_MODE: {
    name: 'Dark Mode',
    description: 'Enable dark mode theme toggle',
    defaultValue: true,
  },

  PROGRESSIVE_AUTH_UI: {
    name: 'Progressive Authentication UI',
    description: 'Show progressive auth states in UI (PUBLIC → CONNECTED → AUTHENTICATED)',
    defaultValue: true,
  },

  ENHANCED_TOOLTIPS: {
    name: 'Enhanced Tooltips',
    description: 'Enable enhanced tooltips with rich content and animations',
    defaultValue: false,
    environments: ['development'],
  },

  // ============================================================================
  // Authentication & Security Features  
  // ============================================================================

  WEB3_AUTHENTICATION: {
    name: 'Web3 Authentication',
    description: 'Enable Web3 wallet connection and SIWE authentication',
    defaultValue: true,
  },

  MULTI_FACTOR_AUTH: {
    name: 'Multi-Factor Authentication',
    description: 'Enable MFA for enhanced security',
    defaultValue: false,
    environments: ['staging', 'production'],
    enabledForAdmin: true,
    requiredPermissions: ['admin:security:manage'],
  },

  SESSION_MANAGEMENT: {
    name: 'Enhanced Session Management',
    description: 'Enable advanced session management with device tracking',
    defaultValue: true,
    environments: ['staging', 'production'],
  },

  BIOMETRIC_AUTH: {
    name: 'Biometric Authentication',
    description: 'Enable biometric authentication (Face ID, Touch ID, Windows Hello)',
    defaultValue: false,
    rolloutPercentage: 25,
  },

  // ============================================================================
  // Admin-Specific Features
  // ============================================================================

  UNIFIED_USER_MANAGEMENT: {
    name: 'Unified User Management',
    description: 'Enable the new unified user management system for admins',
    defaultValue: true,
    enabledForAdmin: true,
    requiredPermissions: ['admin:users:manage'],
  },

  ADMIN_ANALYTICS_DASHBOARD: {
    name: 'Admin Analytics Dashboard',
    description: 'Enable advanced analytics dashboard for administrators',
    defaultValue: true,
    enabledForAdmin: true,
    environments: ['development', 'staging', 'production'],
  },

  BULK_USER_OPERATIONS: {
    name: 'Bulk User Operations',
    description: 'Enable bulk operations for user management (create, update, delete)',
    defaultValue: true,
    enabledForAdmin: true,
    requiredPermissions: ['admin:users:manage'],
  },

  PERMISSION_TEMPLATES: {
    name: 'Permission Templates',
    description: 'Enable permission template system for easy role assignment',
    defaultValue: true,
    enabledForAdmin: true,
  },

  ADMIN_AUDIT_LOGS: {
    name: 'Admin Audit Logs',
    description: 'Enable comprehensive audit logging for admin actions',
    defaultValue: true,
    enabledForAdmin: true,
    environments: ['staging', 'production'],
  },

  // ============================================================================
  // User-Focused Features
  // ============================================================================

  REAL_TIME_ANALYTICS: {
    name: 'Real-time Analytics',
    description: 'Enable real-time data updates via WebSocket for analytics',
    defaultValue: false,
    rolloutPercentage: 30,
    enabledForUsers: true,
    requiredPermissions: ['epsx:realtime:access'],
  },

  ADVANCED_CHARTING: {
    name: 'Advanced Charting',
    description: 'Enable advanced charting tools and indicators',
    defaultValue: true,
    enabledForUsers: true,
    requiredPermissions: ['epsx:analytics:advanced'],
  },

  PORTFOLIO_TRACKING: {
    name: 'Portfolio Tracking',
    description: 'Enable portfolio tracking and management features',
    defaultValue: true,
    enabledForUsers: true,
  },

  CUSTOM_ALERTS: {
    name: 'Custom Alerts',
    description: 'Enable custom price and portfolio alerts',
    defaultValue: true,
    enabledForUsers: true,
    requiredPermissions: ['epsx:notifications:manage'],
  },

  // ============================================================================
  // Payment & Subscription Features
  // ============================================================================

  CRYPTO_PAYMENTS: {
    name: 'Crypto Payments',
    description: 'Enable cryptocurrency payment processing',
    defaultValue: true,
  },

  SUBSCRIPTION_MANAGEMENT: {
    name: 'Subscription Management',
    description: 'Enable subscription upgrade/downgrade functionality',
    defaultValue: true,
  },

  PAYMENT_HISTORY: {
    name: 'Payment History',
    description: 'Enable detailed payment history and transaction tracking',
    defaultValue: true,
  },

  ENTERPRISE_BILLING: {
    name: 'Enterprise Billing',
    description: 'Enable enterprise billing features (invoicing, custom terms)',
    defaultValue: false,
    environments: ['staging', 'production'],
    requiredPermissions: ['epsx:*:*'],
  },

  // ============================================================================
  // Technical Features
  // ============================================================================

  SERVER_COMPONENTS: {
    name: 'Next.js Server Components',
    description: 'Enable Next.js Server Components where applicable for better performance',
    defaultValue: true,
  },

  EDGE_RUNTIME: {
    name: 'Edge Runtime',
    description: 'Use Edge Runtime for API routes where possible',
    defaultValue: false,
    environments: ['staging', 'production'],
  },

  OPTIMISTIC_UPDATES: {
    name: 'Optimistic Updates',
    description: 'Enable optimistic UI updates for better perceived performance',
    defaultValue: true,
  },

  CACHING_LAYER: {
    name: 'Enhanced Caching',
    description: 'Enable enhanced caching layer for improved performance',
    defaultValue: true,
    environments: ['staging', 'production'],
  },

  // ============================================================================
  // Integration Features
  // ============================================================================

  TRADINGVIEW_INTEGRATION: {
    name: 'TradingView Integration',
    description: 'Enable TradingView charts and widgets integration',
    defaultValue: true,
  },

  WEBHOOK_SYSTEM: {
    name: 'Webhook System',
    description: 'Enable webhook system for external integrations',
    defaultValue: false,
    environments: ['staging', 'production'],
    enabledForAdmin: true,
  },

  API_KEY_MANAGEMENT: {
    name: 'API Key Management',
    description: 'Enable API key generation and management for users',
    defaultValue: true,
    requiredPermissions: ['epsx:api:manage'],
  },

  // ============================================================================
  // Experimental Features
  // ============================================================================

  AI_INSIGHTS: {
    name: 'AI-Powered Insights',
    description: 'Enable AI-powered market insights and recommendations',
    defaultValue: false,
    environments: ['development'],
    rolloutPercentage: 10,
  },

  VOICE_COMMANDS: {
    name: 'Voice Commands',
    description: 'Enable voice command interface for accessibility',
    defaultValue: false,
    environments: ['development'],
  },

  MACHINE_LEARNING_SIGNALS: {
    name: 'ML Market Signals',
    description: 'Enable machine learning-based market signals',
    defaultValue: false,
    environments: ['development'],
    requiredPermissions: ['epsx:analytics:advanced'],
  },
} as const;

// ============================================================================
// FEATURE FLAG EVALUATION
// ============================================================================

/**
 * Check if a feature flag is enabled for the current context
 */
export function isFeatureEnabled(
  flag: keyof typeof FEATURE_FLAGS,
  context: FeatureFlagContext = {}
): boolean {
  const config = FEATURE_FLAGS[flag];

  if (!checkStatus(config)) { return false; }
  if (!checkEnvironment(config, context.environment)) { return false; }
  if (!checkRoleRestrictions(config, context)) { return false; }
  if (!checkPermissions(config, context.userPermissions)) { return false; }
  if (!checkRollout(config, flag, context.userId)) { return false; }

  return checkOverrides(flag, config.defaultValue);
}

function checkStatus(config: FeatureFlagConfig): boolean {
  return !(config.removedAt !== undefined && Date.now() > new Date(config.removedAt).getTime());
}

function checkEnvironment(config: FeatureFlagConfig, env?: string): boolean {
  const currentEnv = (env ?? process.env.NODE_ENV) as 'development' | 'staging' | 'production';
  return config.environments === undefined || config.environments.includes(currentEnv);
}

function checkRoleRestrictions(config: FeatureFlagConfig, context: FeatureFlagContext): boolean {
  if (config.enabledForAdmin === true && context.isAdmin !== true) { return false; }
  if (config.enabledForUsers === true && context.isAdmin === true) { return false; }
  return true;
}

function checkPermissions(config: FeatureFlagConfig, currentPermissions?: string[]): boolean {
  if (config.requiredPermissions !== undefined && !hasRequiredPermissions(config.requiredPermissions, currentPermissions)) {
    return false;
  }
  return true;
}

function checkRollout(config: FeatureFlagConfig, flag: string, userId?: string): boolean {
  if (config.rolloutPercentage !== undefined) {
    const id = userId ?? 'anonymous';
    const hash = simpleHash(id + flag);
    const percentage = (hash % 100) + 1;
    return percentage <= config.rolloutPercentage;
  }
  return true;
}

function checkOverrides(flag: string, defaultValue: boolean): boolean {
  const envVar = `NEXT_PUBLIC_ENABLE_${flag.toUpperCase()}`;
  const envValue = process.env[envVar];
  if (envValue !== undefined && envValue !== '') {
    return envValue.toLowerCase() === 'true';
  }
  return defaultValue;
}

/**
 * Helper to check if context has required permissions
 */
function hasRequiredPermissions(required: string[], current?: string[]): boolean {
  if (current === undefined) { return false; }
  return required.some(permission =>
    current.includes(permission) ||
    current.includes('admin:*:*') ||
    current.includes('epsx:*:*')
  );
}

/**
 * Get all enabled features for a context
 */
export function getEnabledFeatures(context: FeatureFlagContext = {}): string[] {
  return Object.keys(FEATURE_FLAGS).filter(flag =>
    isFeatureEnabled(flag, context)
  );
}

/**
 * Get feature flag configuration
 */
export function getFeatureConfig(flag: keyof typeof FEATURE_FLAGS): FeatureFlagConfig | undefined {
  return FEATURE_FLAGS[flag];
}

/**
 * Get all feature flags (for admin interface)
 */
export function getAllFeatureFlags(): Record<string, FeatureFlagConfig> {
  return FEATURE_FLAGS;
}

/**
 * Check if user can toggle a feature flag (admin only)
 */
export function canToggleFeature(flag: keyof typeof FEATURE_FLAGS, context: FeatureFlagContext): boolean {
  if (context.isAdmin !== true) { return false; }

  return (context.userPermissions?.includes('admin:system:manage') ??
    context.userPermissions?.includes('admin:*:*')) ?? false;
}

/**
 * Validate feature flag context
 */
export function validateContext(context: FeatureFlagContext): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (context.userId !== undefined && typeof context.userId !== 'string') {
    errors.push('userId must be a string');
  }

  if (context.userPermissions !== undefined && !Array.isArray(context.userPermissions)) {
    errors.push('userPermissions must be an array');
  }

  if (context.isAdmin !== undefined && typeof context.isAdmin !== 'boolean') {
    errors.push('isAdmin must be a boolean');
  }

  if (context.environment !== undefined && !['development', 'staging', 'production'].includes(context.environment)) {
    errors.push('environment must be development, staging, or production');
  }

  return {
    valid: errors.length === 0,
    errors
  };
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Simple hash function for consistent rollout percentages
 */
function simpleHash(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash);
}

/**
 * Create feature flag context from user data
 */
export function createFeatureContext(
  userId?: string,
  userPermissions?: string[],
  isAdmin?: boolean
): FeatureFlagContext {
  return {
    userId,
    userPermissions,
    isAdmin,
    environment: (process.env.NODE_ENV as 'development' | 'staging' | 'production'),
    userAgent: typeof window !== 'undefined' ? window.navigator.userAgent : undefined
  };
}

/**
 * Feature flag React hook helper (for client-side components)
 */
export function createFeatureFlagHook() {
  return function useFeatureFlag(
    flag: keyof typeof FEATURE_FLAGS,
    context: FeatureFlagContext = {}
  ): boolean {
    return isFeatureEnabled(flag, context);
  };
}

// ============================================================================
// DEVELOPMENT UTILITIES
// ============================================================================

/**
 * List all deprecated features (for cleanup)
 */
export function getDeprecatedFeatures(): Array<{ flag: string; config: FeatureFlagConfig }> {
  return Object.entries(FEATURE_FLAGS)
    .filter(([_, config]) => config.deprecatedAt !== undefined)
    .map(([flag, config]) => ({ flag, config }));
}

/**
 * List features by rollout percentage (for monitoring)
 */
export function getFeaturesByRollout(): Array<{ flag: string; percentage: number; config: FeatureFlagConfig }> {
  return Object.entries(FEATURE_FLAGS)
    .filter((entry): entry is [string, FeatureFlagConfig & { rolloutPercentage: number }] => entry[1].rolloutPercentage !== undefined)
    .map(([flag, config]) => ({ flag, percentage: config.rolloutPercentage, config }));
}

/**
 * Get feature flags for specific environment
 */
export function getEnvironmentFeatures(environment: 'development' | 'staging' | 'production'): string[] {
  return Object.entries(FEATURE_FLAGS)
    .filter(([_, config]) => !config.environments || config.environments.includes(environment))
    .map(([flag]) => flag);
}

// ============================================================================
// LEGACY COMPATIBILITY
// ============================================================================

// Export individual feature checks for backward compatibility
export const featureFlags = {
  // UI Features
  isNewNavigationEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('NEW_NAVIGATION', ctx),
  isDarkModeEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('DARK_MODE', ctx),

  // Auth Features
  isWeb3AuthEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('WEB3_AUTHENTICATION', ctx),
  isMFAEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('MULTI_FACTOR_AUTH', ctx),

  // Admin Features
  isUnifiedUserManagementEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('UNIFIED_USER_MANAGEMENT', ctx),
  isBulkOperationsEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('BULK_USER_OPERATIONS', ctx),

  // User Features
  isRealTimeAnalyticsEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('REAL_TIME_ANALYTICS', ctx),
  isAdvancedChartingEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('ADVANCED_CHARTING', ctx),

  // Technical Features
  areServerComponentsEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('SERVER_COMPONENTS', ctx),
  isOptimisticUpdatesEnabled: (ctx?: FeatureFlagContext) => isFeatureEnabled('OPTIMISTIC_UPDATES', ctx),
} as const;