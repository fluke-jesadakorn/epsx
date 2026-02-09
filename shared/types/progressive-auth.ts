/**
 * CONSOLIDATED PROGRESSIVE AUTHENTICATION TYPES
 * Unified progressive auth system supporting both admin and user contexts
 * Three-tier authentication: PUBLIC -> CONNECTED -> AUTHENTICATED
 */

import type { ReactNode } from 'react';

// ============================================================================
// CORE PROGRESSIVE AUTH TYPES
// ============================================================================

export const AuthLevel = {
  PUBLIC: 'public' as const,           // No wallet needed - anyone can access
  CONNECTED: 'connected' as const,     // Wallet connected, no signature - limited access/personalization
  AUTHENTICATED: 'authenticated' as const // Cryptographic signature required - full privileges
};

export type AuthLevelType = typeof AuthLevel[keyof typeof AuthLevel];

export interface BaseAuthState {
  level: AuthLevelType;
  walletAddress?: string;
  isAuthenticated: boolean;
  isWalletConnected: boolean;
}

export interface UserAuthState extends BaseAuthState {
  // User-specific auth state (performance optimized)
  permissions?: string[];
  packageTier?: string;
  hasValidSubscription?: boolean;
}

export interface AdminAuthState extends BaseAuthState {
  // Admin-specific auth state (enhanced security)
  adminPermissions: string[];
  adminLevel?: 'super' | 'manager' | 'moderator';
  securityLevel?: 'standard' | 'elevated' | 'critical';
  mfaVerified?: boolean;
  deviceTrusted?: boolean;
}

export type AuthState = UserAuthState | AdminAuthState;

// ============================================================================
// PROGRESSIVE AUTH COMPONENT PROPS
// ============================================================================

export interface BaseProgressiveAuthProps {
  /**
   * Minimum authentication level required for this component
   */
  requiredLevel: AuthLevelType;

  /**
   * Content to show when user meets the required auth level
   */
  children: ReactNode;

  /**
   * Optional fallback content for unauthorized users
   */
  fallback?: ReactNode;

  /**
   * Custom message explaining why authentication is needed
   */
  authMessage?: string;

  /**
   * Whether to show upgrade prompts
   */
  showUpgradePrompts?: boolean;

  /**
   * Action name for better UX messaging
   */
  actionName?: string;
}

export interface UserProgressiveAuthProps extends BaseProgressiveAuthProps {
  /**
   * Context type for user authentication (optional for backward compatibility)
   */
  context?: 'user';

  /**
   * Required permissions for user context (e.g., ['epsx:analytics:view'])
   */
  requiredPermissions?: string[];

  /**
   * Minimum subscription tier required
   */
  requiredTier?: string;
}

export interface AdminProgressiveAuthProps extends BaseProgressiveAuthProps {
  /**
   * Context type for admin authentication (optional for backward compatibility)
   */
  context?: 'admin';

  /**
   * Required admin permissions (e.g., ['admin:users:manage', 'admin:system:view'])
   */
  requiredPermissions?: string[];

  /**
   * Required security level for admin operations
   */
  requiredSecurityLevel?: 'standard' | 'elevated' | 'critical';

  /**
   * Whether MFA is required for this operation
   */
  requireMFA?: boolean;
}

// For backward compatibility - make context optional in the main interface
export interface ProgressiveAuthProps extends BaseProgressiveAuthProps {
  /**
   * Optional context type for authentication (defaults to 'user' if not specified)
   */
  context?: 'user' | 'admin';

  /**
   * Required permissions (format depends on context)
   */
  requiredPermissions?: string[];

  /**
   * Required tier for user context
   */
  requiredTier?: string;

  /**
   * Required security level for admin context
   */
  requiredSecurityLevel?: 'standard' | 'elevated' | 'critical';

  /**
   * Whether MFA is required (admin context)
   */
  requireMFA?: boolean;
}

export interface AuthGateProps extends ProgressiveAuthProps {
  /**
   * Loading component to show while determining auth state
   */
  loading?: ReactNode;
}

// ============================================================================
// COMPONENT PROPS FOR DIFFERENT AUTH LEVELS
// ============================================================================

export interface ConnectedComponentProps {
  /**
   * Enhanced props when wallet is connected (limited access/personalization)
   */
  walletAddress?: string;
  isConnected: boolean;
}

export interface UserConnectedComponentProps extends ConnectedComponentProps {
  /**
   * User-specific connected state
   */
  permissions?: string[];
  packageTier?: string;
}

export interface AdminConnectedComponentProps extends ConnectedComponentProps {
  /**
   * Admin-specific connected state (limited admin access)
   */
  adminPermissions: string[];
  adminLevel?: string;
}

export interface AuthenticatedComponentProps extends ConnectedComponentProps {
  /**
   * Full authentication state for sensitive operations
   */
  isAuthenticated: boolean;
}

export interface UserAuthenticatedComponentProps extends UserConnectedComponentProps {
  /**
   * Full user authentication state
   */
  isAuthenticated: boolean;
  hasValidSubscription: boolean;
}

export interface AdminAuthenticatedComponentProps extends AdminConnectedComponentProps {
  /**
   * Full admin authentication state for sensitive operations
   */
  isAuthenticated: boolean;
  adminLevel: 'super' | 'manager' | 'moderator';
  securityLevel: 'standard' | 'elevated' | 'critical';
  mfaVerified: boolean;
  deviceTrusted: boolean;
}

// ============================================================================
// AUTH MESSAGES AND ACTION NAMES
// ============================================================================

export const AUTH_MESSAGES = {
  // User messages
  USER: {
    CONNECT_WALLET: 'Connect your wallet to personalize your experience',
    SIGN_IN_REQUIRED: 'Sign in with your wallet to access this feature',
    PAYMENT_REQUIRED: 'Authentication required to process payments',
    SETTINGS_REQUIRED: 'Sign in to modify your account settings',
    API_REQUIRED: 'Authentication required to generate API keys',
    PREMIUM_REQUIRED: 'Sign in to access premium features',
    SUBSCRIPTION_REQUIRED: 'Upgrade your subscription to access this feature',
  },

  // Admin messages
  ADMIN: {
    CONNECT_WALLET: 'Connect your admin wallet to access administrative features',
    SIGN_IN_REQUIRED: 'Sign in with your admin wallet to perform this action',
    USER_MANAGEMENT_REQUIRED: 'Authentication required to manage users',
    SYSTEM_REQUIRED: 'Authentication required to access system settings',
    PERMISSIONS_REQUIRED: 'Authentication required to manage permissions',
    ANALYTICS_REQUIRED: 'Authentication required to view admin analytics',
    SECURITY_REQUIRED: 'Authentication required for security operations',
    MFA_REQUIRED: 'Multi-factor authentication required for this operation',
    ELEVATED_ACCESS_REQUIRED: 'Elevated security clearance required',
  }
} as const;

export const ACTION_NAMES = {
  // User actions
  USER: {
    VIEW_PREMIUM: 'view premium analytics',
    MAKE_PAYMENT: 'process payment',
    MODIFY_SETTINGS: 'modify settings',
    GENERATE_API_KEY: 'generate API key',
    ACCESS_DASHBOARD: 'access your dashboard',
    SAVE_PREFERENCES: 'save preferences',
    EXPORT_DATA: 'export your data',
    VIEW_ANALYTICS_DATA: 'view analytics data',
  },

  // Admin actions
  ADMIN: {
    MANAGE_USERS: 'manage users',
    VIEW_ANALYTICS: 'view admin analytics',
    MODIFY_PERMISSIONS: 'modify permissions',
    ACCESS_SYSTEM: 'access system settings',
    SECURITY_OPERATIONS: 'perform security operations',
    MANAGE_CONTENT: 'manage content',
    VIEW_LOGS: 'view system logs',
    MODIFY_CONFIGURATION: 'modify system configuration',
  }
} as const;

// ============================================================================
// PERMISSION LEVELS AND REQUIREMENTS
// ============================================================================

export const PERMISSION_LEVELS = {
  // User permission levels
  USER: {
    FREE: ['epsx:analytics:basic'],
    BASIC: ['epsx:analytics:*', 'epsx:export:limited'],
    PRO: ['epsx:analytics:*', 'epsx:export:*', 'epsx:api:*'],
    ENTERPRISE: ['epsx:*:*'],
  },

  // Admin permission levels
  ADMIN: {
    SUPER_ADMIN: ['admin:*:*'],
    USER_MANAGER: ['admin:users:*', 'admin:permissions:view'],
    CONTENT_MANAGER: ['admin:content:*', 'admin:analytics:view'],
    SYSTEM_VIEWER: ['admin:system:view', 'admin:analytics:view'],
    MODERATOR: ['admin:users:moderate', 'admin:content:moderate'],
    SECURITY_OFFICER: ['admin:security:*', 'admin:audit:*'],
  }
} as const;

export const SECURITY_LEVELS = {
  STANDARD: 'standard',
  ELEVATED: 'elevated',
  CRITICAL: 'critical',
} as const;

// ============================================================================
// CONTEXT-SPECIFIC HELPER FUNCTIONS
// ============================================================================

export function getUserAuthMessage(actionName?: string, level?: AuthLevelType): string {
  if (actionName === undefined || actionName === '') {
    switch (level) {
      case AuthLevel.CONNECTED:
        return AUTH_MESSAGES.USER.CONNECT_WALLET;
      case AuthLevel.AUTHENTICATED:
        return AUTH_MESSAGES.USER.SIGN_IN_REQUIRED;
      default:
        return AUTH_MESSAGES.USER.SIGN_IN_REQUIRED;
    }
  }

  // Map action names to specific messages
  const messageMap: Record<string, string> = {
    [ACTION_NAMES.USER.MAKE_PAYMENT]: AUTH_MESSAGES.USER.PAYMENT_REQUIRED,
    [ACTION_NAMES.USER.MODIFY_SETTINGS]: AUTH_MESSAGES.USER.SETTINGS_REQUIRED,
    [ACTION_NAMES.USER.GENERATE_API_KEY]: AUTH_MESSAGES.USER.API_REQUIRED,
    [ACTION_NAMES.USER.VIEW_PREMIUM]: AUTH_MESSAGES.USER.PREMIUM_REQUIRED,
  };

  return messageMap[actionName] || AUTH_MESSAGES.USER.SIGN_IN_REQUIRED;
}

export function getAdminAuthMessage(actionName?: string, level?: AuthLevelType, requireMFA?: boolean): string {
  if (requireMFA === true) {
    return AUTH_MESSAGES.ADMIN.MFA_REQUIRED;
  }

  if (actionName === undefined || actionName === '') {
    switch (level) {
      case AuthLevel.CONNECTED:
        return AUTH_MESSAGES.ADMIN.CONNECT_WALLET;
      case AuthLevel.AUTHENTICATED:
        return AUTH_MESSAGES.ADMIN.SIGN_IN_REQUIRED;
      default:
        return AUTH_MESSAGES.ADMIN.SIGN_IN_REQUIRED;
    }
  }

  // Map action names to specific messages
  const messageMap: Record<string, string> = {
    [ACTION_NAMES.ADMIN.MANAGE_USERS]: AUTH_MESSAGES.ADMIN.USER_MANAGEMENT_REQUIRED,
    [ACTION_NAMES.ADMIN.ACCESS_SYSTEM]: AUTH_MESSAGES.ADMIN.SYSTEM_REQUIRED,
    [ACTION_NAMES.ADMIN.MODIFY_PERMISSIONS]: AUTH_MESSAGES.ADMIN.PERMISSIONS_REQUIRED,
    [ACTION_NAMES.ADMIN.VIEW_ANALYTICS]: AUTH_MESSAGES.ADMIN.ANALYTICS_REQUIRED,
    [ACTION_NAMES.ADMIN.SECURITY_OPERATIONS]: AUTH_MESSAGES.ADMIN.SECURITY_REQUIRED,
  };

  return messageMap[actionName] || AUTH_MESSAGES.ADMIN.SIGN_IN_REQUIRED;
}

// ============================================================================
// TYPE GUARDS AND UTILITIES
// ============================================================================

export function isUserAuthProps(props: ProgressiveAuthProps): props is UserProgressiveAuthProps {
  return props.context === 'user';
}

export function isAdminAuthProps(props: ProgressiveAuthProps): props is AdminProgressiveAuthProps {
  return props.context === 'admin';
}

export function isUserAuthState(state: AuthState): state is UserAuthState {
  return 'packageTier' in state || 'hasValidSubscription' in state;
}

export function isAdminAuthState(state: AuthState): state is AdminAuthState {
  return 'adminPermissions' in state;
}

export function hasRequiredPermissions(
  userPermissions: string[],
  requiredPermissions: string[]
): boolean {
  if (requiredPermissions.length === 0) { return true; }

  return requiredPermissions.every(required =>
    userPermissions.some(permission =>
      permission === required ||
      permission.includes('*') ||
      permission === 'admin:*:*' ||
      permission === 'epsx:*:*'
    )
  );
}

export function meetsSecurityLevel(
  userLevel: string,
  requiredLevel: 'standard' | 'elevated' | 'critical'
): boolean {
  const levelOrder = { 'standard': 0, 'elevated': 1, 'critical': 2 };
  return levelOrder[userLevel as keyof typeof levelOrder] >= levelOrder[requiredLevel];
}

export function getRequiredAuthLevel(permissions: string[]): AuthLevelType {
  // Determine minimum auth level based on permissions
  const sensitivePermissions = [
    'admin:security:', 'admin:system:', 'epsx:payment:', 'epsx:api:'
  ];

  const hasSensitivePermission = permissions.some(permission =>
    sensitivePermissions.some(sensitive => permission.includes(sensitive))
  );

  if (hasSensitivePermission) {
    return AuthLevel.AUTHENTICATED;
  }

  if (permissions.length > 0) {
    return AuthLevel.CONNECTED;
  }

  return AuthLevel.PUBLIC;
}

// ============================================================================
// PROGRESSIVE AUTH STATE MANAGEMENT
// ============================================================================

export interface ProgressiveAuthContextValue {
  authState: AuthState;
  updateAuthLevel: (level: AuthLevelType) => void;
  connectWallet: () => Promise<void>;
  authenticateUser: () => Promise<void>;
  disconnect: () => void;
  checkPermission: (permission: string) => boolean;
  checkSecurityLevel?: (level: 'standard' | 'elevated' | 'critical') => boolean;
}

export interface AuthUpgradePromptProps {
  currentLevel: AuthLevelType;
  requiredLevel: AuthLevelType;
  context: 'user' | 'admin';
  actionName?: string;
  onConnect?: () => void;
  onAuthenticate?: () => void;
  onCancel?: () => void;
}

// ============================================================================
// COMPATIBILITY EXPORTS
// ============================================================================

// Note: Main types are already exported above
// Legacy aliases removed to prevent TypeScript conflicts