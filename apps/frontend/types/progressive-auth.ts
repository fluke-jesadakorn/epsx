/**
 * Progressive Authentication Types
 * Defines the three-tier authentication system for EPSX
 */

export enum AuthLevel {
  PUBLIC = 'public',           // No wallet needed - anyone can access
  CONNECTED = 'connected',     // Wallet connected, no signature - personalization
  AUTHENTICATED = 'authenticated' // Cryptographic signature required - sensitive actions
}

export interface AuthState {
  level: AuthLevel;
  walletAddress?: string;
  isAuthenticated: boolean;
  isWalletConnected: boolean;
}

export interface ProgressiveAuthProps {
  /**
   * Minimum authentication level required for this component
   */
  requiredLevel: AuthLevel;
  
  /**
   * Content to show when user meets the required auth level
   */
  children: React.ReactNode;
  
  /**
   * Optional fallback content for unauthorized users
   */
  fallback?: React.ReactNode;
  
  /**
   * Custom message explaining why authentication is needed
   */
  authMessage?: string;
  
  /**
   * Whether to show upgrade prompts
   */
  showUpgradePrompts?: boolean;
  
  /**
   * Action name for better UX messaging (e.g., "view premium data", "make payment")
   */
  actionName?: string;
}

export interface AuthGateProps extends ProgressiveAuthProps {
  /**
   * Loading component to show while determining auth state
   */
  loading?: React.ReactNode;
}

/**
 * Progressive authentication component props for different scenarios
 */
export interface ConnectedComponentProps {
  /**
   * Enhanced props when wallet is connected (personalization)
   */
  walletAddress?: string;
  isConnected: boolean;
}

export interface AuthenticatedComponentProps extends ConnectedComponentProps {
  /**
   * Full authentication state for sensitive operations
   */
  isAuthenticated: boolean;
}

/**
 * Standard auth messages for different scenarios
 */
export const AUTH_MESSAGES = {
  CONNECT_WALLET: 'Connect your wallet to personalize your experience',
  SIGN_IN_REQUIRED: 'Sign in with your wallet to access this feature',
  PAYMENT_REQUIRED: 'Authentication required to process payments',
  SETTINGS_REQUIRED: 'Sign in to modify your account settings',
  API_REQUIRED: 'Authentication required to generate API keys',
  PREMIUM_REQUIRED: 'Sign in to access premium features',
} as const;

/**
 * Action names for better UX
 */
export const ACTION_NAMES = {
  VIEW_PREMIUM: 'view premium analytics',
  MAKE_PAYMENT: 'process payment',
  MODIFY_SETTINGS: 'modify settings',
  GENERATE_API_KEY: 'generate API key',
  ACCESS_DASHBOARD: 'access your dashboard',
  SAVE_PREFERENCES: 'save preferences',
} as const;