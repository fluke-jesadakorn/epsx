/**
 * ADMIN FRONTEND PROGRESSIVE AUTH TYPES - MIGRATED TO SHARED
 * All progressive auth types moved to shared/types/progressive-auth with compatibility layer
 * This file now re-exports shared types for backward compatibility with admin context
 */

// Re-export everything from shared progressive auth types
export * from '../../../shared/types/progressive-auth';

// Import for local re-export with legacy names (maintaining compatibility)
import type {
  AuthLevel as SharedAuthLevel,
  AuthState as SharedAuthState,
  AdminAuthState as SharedAdminAuthState,
  ProgressiveAuthProps as SharedProgressiveAuthProps,
  AdminProgressiveAuthProps as SharedAdminProgressiveAuthProps,
  AuthGateProps as SharedAuthGateProps,
  ConnectedComponentProps as SharedConnectedComponentProps,
  AdminConnectedComponentProps as SharedAdminConnectedComponentProps,
  AuthenticatedComponentProps as SharedAuthenticatedComponentProps,
  AdminAuthenticatedComponentProps as SharedAdminAuthenticatedComponentProps
} from '../../../shared/types/progressive-auth';

// Re-export with exact same names for backward compatibility
export const AuthLevel = {
  PUBLIC: 'public' as const,
  CONNECTED: 'connected' as const,
  AUTHENTICATED: 'authenticated' as const,
};

export type AuthState = SharedAdminAuthState;
export type ProgressiveAuthProps = SharedAdminProgressiveAuthProps;
export type AuthGateProps = SharedAuthGateProps;

// Component props for different auth levels (admin-focused)
export type ConnectedAdminProps = SharedAdminConnectedComponentProps;
export type AuthenticatedAdminProps = SharedAdminAuthenticatedComponentProps;

// Legacy compatibility aliases
export type ConnectedComponentProps = SharedConnectedComponentProps;
export type AuthenticatedComponentProps = SharedAuthenticatedComponentProps;

// Admin-specific constants (re-exported from shared with admin context)
export const ADMIN_AUTH_MESSAGES = {
  CONNECT_WALLET: 'Connect your admin wallet to access administrative features',
  SIGN_IN_REQUIRED: 'Sign in with your admin wallet to perform this action',
  USER_MANAGEMENT_REQUIRED: 'Authentication required to manage users',
  SYSTEM_REQUIRED: 'Authentication required to access system settings',
  PERMISSIONS_REQUIRED: 'Authentication required to manage permissions',
  ANALYTICS_REQUIRED: 'Authentication required to view admin analytics',
  SECURITY_REQUIRED: 'Authentication required for security operations',
} as const;

export const ADMIN_ACTION_NAMES = {
  MANAGE_USERS: 'manage users',
  VIEW_ANALYTICS: 'view admin analytics',
  MODIFY_PERMISSIONS: 'modify permissions',
  ACCESS_SYSTEM: 'access system settings',
  SECURITY_OPERATIONS: 'perform security operations',
  MANAGE_CONTENT: 'manage content',
  VIEW_LOGS: 'view system logs',
} as const;

export const ADMIN_PERMISSION_LEVELS = {
  SUPER_ADMIN: ['admin:*:*'],
  USER_MANAGER: ['admin:users:*', 'admin:permissions:view'],
  CONTENT_MANAGER: ['admin:content:*', 'admin:analytics:view'],
  SYSTEM_VIEWER: ['admin:system:view', 'admin:analytics:view'],
  MODERATOR: ['admin:users:moderate', 'admin:content:moderate'],
} as const;