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
export { default as UnifiedProgressiveAuthGate } from './unified-progressive-auth-gate';

export type {
  UnifiedProgressiveAuthGateProps
} from './unified-progressive-auth-gate';

// Progressive auth gate convenience components
export {
  RequireFullAuth, RequireProgressiveAuth, RequireSignIn
} from './unified-progressive-auth-gate';

// Higher-order component and hooks
export {
  useProgressiveAuthStatus, withProgressiveAuth
} from './unified-progressive-auth-gate';

