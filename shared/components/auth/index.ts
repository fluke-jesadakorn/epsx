/**
 * SHARED AUTH COMPONENTS INDEX
 * 
 * Consolidated exports for all shared authentication components.
 * Client-side permission validation has been removed in favor of backend enforcement.
 */

// NEW: Premium Auth Modal and Status components
export { AuthModal } from './AuthModal';
export type { AuthModalProps, AuthResult } from './AuthModal';
export { AuthStatus } from './AuthStatus';
export type { AuthStatusProps } from './AuthStatus';

// Unified progressive auth gate component
export { default as UnifiedProgressiveAuthGate } from './UnifiedProgressiveAuthGate';

export type {
  UnifiedProgressiveAuthGateProps
} from './UnifiedProgressiveAuthGate';

// Progressive auth gate convenience components
export {
  RequireFullAuth, RequireProgressiveAuth, RequireSignIn
} from './UnifiedProgressiveAuthGate';

// Higher-order component and hooks
export {
  useProgressiveAuthStatus, withProgressiveAuth
} from './UnifiedProgressiveAuthGate';
