/**
 * SHARED AUTH COMPONENTS INDEX
 *
 * Consolidated exports for all shared authentication components.
 * Client-side permission validation has been removed in favor of backend enforcement.
 */

// NEW: Premium Auth Modal and Status components
export { AuthModal } from './auth-modal';
export type { AuthModalProps, AuthResult } from './auth-modal';
export { AuthStatus } from './auth-status';
export type { AuthStatusProps } from './auth-status';

// Unified progressive auth gate component
export { default as UnifiedProgressiveAuthGate } from './progressive-auth-gate';

export type {
  UnifiedProgressiveAuthGateProps
} from './progressive-auth-gate';

// Progressive auth gate convenience components
export {
  RequireFullAuth, RequireProgressiveAuth, RequireSignIn
} from './progressive-auth-gate';

// Higher-order component and hooks
export {
  useProgressiveAuthStatus, withProgressiveAuth
} from './progressive-auth-gate';

// Auth provider and context
export { SharedOpenIDWeb3Provider, useSharedAuth, useAuth } from './provider';
export type { SharedAuthContextValue } from './provider';

