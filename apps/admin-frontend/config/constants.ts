/**
 * ADMIN FRONTEND - CONSTANTS COMPATIBILITY LAYER
 * Web3 wallet-first system with stub constants for compatibility
 * Provides admin-specific constants and backward compatibility
 */

// Stub types for compatibility
export type PermissionTemplateName = 'basic' | 'advanced' | 'enterprise';
export type AssetType = 'stock' | 'crypto' | 'commodity';
export type PaymentMethod = 'crypto' | 'fiat' | 'token';
export interface NetworkConfig {
  chainId: number;
  name: string;
  rpcUrl: string;
  nativeCurrency: { name: string; symbol: string; decimals: number };
}

// Stub constants for compatibility
export const PERMISSION_TEMPLATES = {
  basic: { name: 'Basic', permissions: ['read'] },
  advanced: { name: 'Advanced', permissions: ['read', 'write'] },
  enterprise: { name: 'Enterprise', permissions: ['read', 'write', 'admin'] }
} as const;

export const ASSET_DEFINITIONS = {
  stock: { type: 'stock', category: 'equity' },
  crypto: { type: 'crypto', category: 'digital' },
  commodity: { type: 'commodity', category: 'physical' }
} as const;

export const PAYMENT_CONFIGS = {
  crypto: { type: 'crypto', enabled: true },
  fiat: { type: 'fiat', enabled: false },
  token: { type: 'token', enabled: true }
} as const;

export const BLOCKCHAIN_NETWORKS = {
  bsc: { chainId: 56, name: 'BSC Mainnet', rpcUrl: 'https://bsc-dataseed.binance.org/', nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 } },
  bsc_testnet: { chainId: 97, name: 'BSC Testnet', rpcUrl: 'https://data-seed-prebsc-1-s1.binance.org:8545/', nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 } }
} as const;

export const UI_CONSTANTS = {
  BREAKPOINTS: { sm: 640, md: 768, lg: 1024, xl: 1280 },
  COLORS: { primary: '#0070f3', secondary: '#666', accent: '#ff0080' },
  SPACING: { xs: 4, sm: 8, md: 16, lg: 24, xl: 32 }
} as const;

// Stub utility functions for compatibility
/**
 *
 * @param name
 */
export function getPermissionTemplate(name: PermissionTemplateName) {
  return PERMISSION_TEMPLATES[name];
}

/**
 *
 * @param type
 */
export function getAssetDefinition(type: AssetType) {
  return ASSET_DEFINITIONS[type];
}

/**
 *
 * @param method
 */
export function getPaymentConfig(method: PaymentMethod) {
  return PAYMENT_CONFIGS[method];
}

/**
 *
 * @param network
 */
export function getNetworkConfig(network: keyof typeof BLOCKCHAIN_NETWORKS): NetworkConfig {
  return BLOCKCHAIN_NETWORKS[network];
}

/**
 *
 * @param template
 */
export function calculatePermissionPrice(template: PermissionTemplateName): number {
  const prices = { basic: 10, advanced: 50, enterprise: 200 };
  return prices[template] || 0;
}

/**
 *
 * @param type
 */
export function validateAssetType(type: string): type is AssetType {
  return ['stock', 'crypto', 'commodity'].includes(type);
}

/**
 *
 * @param method
 */
export function isValidPaymentMethod(method: string): method is PaymentMethod {
  return ['crypto', 'fiat', 'token'].includes(method);
}

/**
 *
 * @param amount
 * @param currency
 */
export function formatCurrency(amount: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency === 'BNB' ? 'USD' : currency
  }).format(amount);
}

/**
 * Admin-specific constants
 * Additional constants used only in admin frontend
 */
export const ADMIN_CONSTANTS = {
  // Admin-specific UI constants
  ADMIN_UI: {
    // Navigation
    SIDEBAR_WIDTH: '250px',
    SIDEBAR_COLLAPSED_WIDTH: '60px',
    HEADER_HEIGHT: '64px',

    // Tables
    DEFAULT_PAGE_SIZE: 25,
    MAX_PAGE_SIZE: 100,

    // Modals
    MODAL_SIZES: {
      SMALL: 'max-w-md',
      MEDIUM: 'max-w-lg',
      LARGE: 'max-w-4xl',
      EXTRA_LARGE: 'max-w-6xl',
    },

    // Animations (keeping as static values per zero-animation policy)
    TRANSITIONS: {
      NONE: 'transition-none',
      INSTANT: 'transition-none',
    },
  },

  // Admin operation limits
  LIMITS: {
    BULK_USER_CREATE: 100,
    BULK_USER_UPDATE: 500,
    BULK_PERMISSION_GRANT: 1000,
    MAX_SEARCH_RESULTS: 1000,
    AUDIT_LOG_PAGE_SIZE: 50,
  },

  // Admin notification types
  NOTIFICATION_TYPES: [
    'system_announcement',
    'security_alert',
    'maintenance_notice',
    'feature_release',
    'policy_update',
    'compliance_reminder',
    'billing_notification',
    'user_activity_alert'
  ] as const,

  // Admin roles and permission levels
  ADMIN_ROLES: {
    SUPER_ADMIN: 'super_admin',
    USER_MANAGER: 'user_manager',
    SYSTEM_ADMIN: 'system_admin',
    CONTENT_MANAGER: 'content_manager',
    COMPLIANCE_MANAGER: 'compliance_manager',
    GOVERNANCE_MANAGER: 'governance_manager',
    ENTERPRISE_MANAGER: 'enterprise_manager',
    DEVELOPER_ADMIN: 'developer_admin',
  } as const,

  // Admin dashboard metrics
  DASHBOARD_METRICS: {
    USER_STATS: [
      'total_users',
      'active_users_today',
      'active_users_week',
      'active_users_month',
      'new_users_today',
      'new_users_week',
      'new_users_month'
    ],

    PERMISSION_STATS: [
      'total_permissions',
      'active_permissions',
      'expired_permissions',
      'temporary_permissions'
    ],

    SYSTEM_STATS: [
      'api_requests_today',
      'error_rate',
      'avg_response_time',
      'active_sessions'
    ]
  } as const,

  // Bulk operation types
  BULK_OPERATIONS: [
    'create_users',
    'update_users',
    'delete_users',
    'grant_permissions',
    'revoke_permissions',
    'assign_roles',
    'remove_roles',
    'send_notifications'
  ] as const,

  // Admin audit actions
  AUDIT_ACTIONS: [
    'user_created',
    'user_updated',
    'user_deleted',
    'permission_granted',
    'permission_revoked',
    'role_assigned',
    'role_removed',
    'policy_created',
    'policy_updated',
    'policy_deleted',
    'bulk_operation_executed',
    'system_config_changed',
    'security_event',
    'compliance_check',
    'data_export'
  ] as const,
} as const;

/**
 * Admin-specific Z-index constants (migrated from z-index-constants.ts)
 */
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

  // Modal dialog dialogs and overlays: z-60 to z-70
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

/**
 * Admin-specific utility functions
 */

/**
 * Get admin role display name
 * @param role
 */
export function getAdminRoleDisplayName(role: keyof typeof ADMIN_CONSTANTS.ADMIN_ROLES): string {
  const roleDisplayNames: Record<string, string> = {
    SUPER_ADMIN: 'Super Administrator',
    USER_MANAGER: 'User Manager',
    SYSTEM_ADMIN: 'System Administrator',
    CONTENT_MANAGER: 'Content Manager',
    COMPLIANCE_MANAGER: 'Compliance Manager',
    GOVERNANCE_MANAGER: 'Governance Manager',
    ENTERPRISE_MANAGER: 'Enterprise Manager',
    DEVELOPER_ADMIN: 'Developer Administrator',
  };

  return roleDisplayNames[role] || role;
}

/**
 * Get notification type display name
 * @param type
 */
export function getNotificationTypeDisplayName(type: typeof ADMIN_CONSTANTS.NOTIFICATION_TYPES[number]): string {
  const typeDisplayNames: Record<string, string> = {
    system_announcement: 'System Announcement',
    security_alert: 'Security Alert',
    maintenance_notice: 'Maintenance Notice',
    feature_release: 'Feature Release',
    policy_update: 'Policy Update',
    compliance_reminder: 'Compliance Reminder',
    billing_notification: 'Billing Notification',
    user_activity_alert: 'User Activity Alert',
  };

  return typeDisplayNames[type] || type;
}

/**
 * Validate bulk operation type
 * @param operation
 */
export function isValidBulkOperation(operation: string): operation is typeof ADMIN_CONSTANTS.BULK_OPERATIONS[number] {
  return (ADMIN_CONSTANTS.BULK_OPERATIONS as readonly string[]).includes(operation);
}

/**
 * Get bulk operation limits
 * @param operation
 */
export function getBulkOperationLimit(operation: string): number {
  const limits: Record<string, number> = {
    create_users: ADMIN_CONSTANTS.LIMITS.BULK_USER_CREATE,
    update_users: ADMIN_CONSTANTS.LIMITS.BULK_USER_UPDATE,
    grant_permissions: ADMIN_CONSTANTS.LIMITS.BULK_PERMISSION_GRANT,
  };

  return limits[operation] || 100; // Default limit
}

/**
 * Admin-specific error messages
 */
export const ADMIN_ERROR_MESSAGES = {
  BULK_OPERATION_FAILED: 'Bulk operation failed to complete',
  INVALID_BULK_SIZE: 'Bulk operation exceeds maximum allowed size',
  INSUFFICIENT_ADMIN_PERMISSIONS: 'Insufficient administrative permissions',
  INVALID_ADMIN_CONTEXT: 'Action not allowed in admin context',
  USER_MANAGEMENT_ERROR: 'User management operation failed',
  PERMISSION_MANAGEMENT_ERROR: 'Permission management operation failed',
  SYSTEM_CONFIGURATION_ERROR: 'System configuration operation failed',
} as const;

// Export combined constants for backward compatibility
export const CONSTANTS = {
  // Shared constants
  UI_CONSTANTS,
  PERMISSION_TEMPLATES,
  ASSET_DEFINITIONS,
  PAYMENT_CONFIGS,
  BLOCKCHAIN_NETWORKS,

  // Admin-specific constants
  ...ADMIN_CONSTANTS,
  Z_INDEX_LAYERS,
  ADMIN_ERROR_MESSAGES,
} as const;

// Legacy compatibility - export constants as default
export default CONSTANTS;