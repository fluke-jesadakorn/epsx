/**
 * FRONTEND PROGRESSIVE AUTH TYPES - MIGRATED TO SHARED
 * All progressive auth types moved to shared/types/progressive-auth with compatibility layer
 * This file now re-exports shared types for backward compatibility with user context
 */

// Re-export everything from shared progressive auth types
export * from '../../../shared/types/progressive-auth';

// Import for local re-export with legacy names (maintaining compatibility)
import type {
  AuthLevel as SharedAuthLevel,
  AuthState as SharedAuthState,
  UserAuthState as SharedUserAuthState,
  ProgressiveAuthProps as SharedProgressiveAuthProps,
  UserProgressiveAuthProps as SharedUserProgressiveAuthProps,
  AuthGateProps as SharedAuthGateProps,
  ConnectedComponentProps as SharedConnectedComponentProps,
  UserConnectedComponentProps as SharedUserConnectedComponentProps,
  AuthenticatedComponentProps as SharedAuthenticatedComponentProps,
  UserAuthenticatedComponentProps as SharedUserAuthenticatedComponentProps
} from '../../../shared/types/progressive-auth';

// Re-export with exact same names for backward compatibility
export const AuthLevel = {
  PUBLIC: 'public' as const,
  CONNECTED: 'connected' as const,
  AUTHENTICATED: 'authenticated' as const,
};

// Export type version for components that use AuthLevel as a type
export type AuthLevelType = typeof AuthLevel[keyof typeof AuthLevel];

export type AuthState = SharedUserAuthState;
export type ProgressiveAuthProps = SharedUserProgressiveAuthProps;
export type AuthGateProps = SharedAuthGateProps;

// Component props for different auth levels (user-focused)
export type ConnectedComponentProps = SharedUserConnectedComponentProps;
export type AuthenticatedComponentProps = SharedUserAuthenticatedComponentProps;

// User-specific constants (re-exported from shared with user context)
export const AUTH_MESSAGES = {
  CONNECT_WALLET: 'Connect your wallet to personalize your experience',
  SIGN_IN_REQUIRED: 'Sign in with your wallet to access this feature',
  PAYMENT_REQUIRED: 'Authentication required to process payments',
  SETTINGS_REQUIRED: 'Sign in to modify your account settings',
  API_REQUIRED: 'Authentication required to generate API keys',
  PREMIUM_REQUIRED: 'Sign in to access premium features',
} as const;

export const ACTION_NAMES = {
  VIEW_PREMIUM: 'view premium analytics',
  MAKE_PAYMENT: 'process payment',
  MODIFY_SETTINGS: 'modify settings',
  GENERATE_API_KEY: 'generate API key',
  ACCESS_DASHBOARD: 'access your dashboard',
  SAVE_PREFERENCES: 'save preferences',
} as const;